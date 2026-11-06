"""Local mixed rigid / FFRF open-chain solver. No geometry substitutions."""
from contextlib import redirect_stdout
from hashlib import sha256
from pathlib import Path
from datetime import datetime,timezone
from time import perf_counter
import json,sys,math
import numpy as np
import fem_worker as fe
from motion_profiles import joint_motion, SOURCE_SHA256

def same_part(part, owner):
    return all(part[key] == owner[key] for key in ['source', 'component', 'path'])

def aggregate_parts(parts):
    if not parts or len(parts) > 256:
        raise ValueError('a rigid attachment group needs 1..256 physical CAD parts')
    masses = np.array([float(p['mass_kg']) for p in parts])
    positions = np.array([fe.finite(p['com_world_m'], (3,)) for p in parts])
    if not np.isfinite(masses).all() or min(masses) <= 0:
        raise ValueError('invalid attachment mass')
    mass = sum(masses)
    com = np.average(positions, axis=0, weights=masses)
    tensor = np.zeros((3, 3))
    for part, m, position in zip(parts, masses, positions):
        inertia = fe.finite(part['inertia_com_world_kg_m2'], (3, 3))
        d = position - com
        tensor += inertia + m * ((d @ d) * np.eye(3) - np.outer(d, d))
    if not np.allclose(tensor, tensor.T, rtol=0, atol=1e-10) or min(np.linalg.eigvalsh(tensor)) <= 0:
        raise ValueError('invalid attachment inertia tensor')
    return float(mass), com, tensor

def run(envelope):
    started = perf_counter()
    import exudyn as exu
    from exudyn.FEM import FEMinterface,ObjectFFRFreducedOrderInterface
    from exudyn.utilities import RigidBodyInertia
    from exudyn.itemInterface import ObjectGround,ObjectRigidBody,NodeRigidBodyEP,MarkerBodyRigid,MarkerSuperElementRigid,GenericJoint,SensorMarker,SensorNode,SensorBody,SensorObject,LoadForceVector,LoadTorqueVector
    if envelope.get('schema')!='solidcraft.flex-assembly-job/1':raise ValueError('unsupported flexible assembly job')
    job=envelope['job'];settings=job['settings'];model=job['model']
    if job['backend']!='exudyn_ffrf_open_chain':raise ValueError('incorrect flexible backend')
    duration=float(settings['duration_s']);steps=settings['steps'];samples=settings['samples'];gravity=fe.finite(settings['gravity_m_s2'],(3,))
    if not(.001<=duration<=10 and isinstance(steps,int) and 10<=steps<=10000 and isinstance(samples,int) and 2<=samples<=1001 and steps%(samples-1)==0 and 1<=len(model['bodies'])<=8 and len(model['bodies'])*(samples-1)<=8000 and max(abs(gravity))<=100):raise ValueError('integration limits exceeded')
    initial_equilibrium = settings.get('initial_equilibrium', False)
    if not isinstance(initial_equilibrium, bool):
        raise ValueError('initial_equilibrium must be boolean')
    static_phase = False
    sc=exu.SystemContainer();mbs=sc.AddSystem();ground=mbs.AddObject(ObjectGround())
    ids={model['ground']:ground};initial={model['ground']:np.zeros(3)};sensors={};flex={};physical={}
    attachment_sensors = {}
    attachment_constraints = []
    ovs=dict(position=exu.OutputVariableType.Position,rotation=exu.OutputVariableType.RotationMatrix,velocity=exu.OutputVariableType.Velocity,omega=exu.OutputVariableType.AngularVelocity)
    for b in model['bodies']:
        bid=b['id'];mass=float(b['mass_kg']);com=fe.finite(b['com_world_m'],(3,));tensor=fe.finite(b['inertia_com_world_kg_m2'],(3,3))
        if bid in ids or not 0<mass<1e9 or min(np.linalg.eigvalsh(tensor))<=0:raise ValueError('invalid body mass/inertia')
        initial[bid]=com;physical[bid]=(mass,tensor);sensors[bid]={}
        if 'flexible_component' in b:
            record=envelope['components'][b['flexible_component']];r=record['result'];config=record['job']
            points,tets,boundary,ports=fe.prepare_mesh(config);K,M,_,_,_,fe_mass=fe.assemble(points,tets,config['material'])
            attachments = record.get('rigid_attachments', [])
            parts = b.get('parts', [])
            if attachments:
                covered = list(record['owners']) + [a['owner'] for a in attachments]
                if len(parts) != len(covered) or any(sum(same_part(p, owner) for owner in covered) != 1 for p in parts):
                    raise ValueError('FE owners and rigid attachments must cover every massive CAD part exactly once')
                total, center, inertia = aggregate_parts(parts)
                if not np.isclose(total, mass, rtol=1e-8) or not np.allclose(center, com, rtol=0, atol=1e-9) or not np.allclose(inertia, tensor, rtol=1e-7, atol=1e-10):
                    raise ValueError('attachment mass accounting differs from the native CAD block')
                elastic_mass = sum(float(p['mass_kg']) for p in parts if any(same_part(p, owner) for owner in record['owners']))
            else:
                elastic_mass = mass
            if elastic_mass <= 0 or abs(fe_mass-elastic_mass)/elastic_mass>.02:raise ValueError(f'{bid}: CAD and FE mass differ by more than 2%; use the same material and elastic geometry')
            if not np.allclose(points*1000,fe.finite(r['nodes_mm'],points.shape),rtol=0,atol=1e-8) or not np.array_equal(tets,np.array(r['tetrahedra'])):raise ValueError('ROM mesh does not match prepared FEM')
            basis=fe.finite(r['rom']['basis']);freq=fe.finite(r['rom']['fixed_interface_frequencies_hz']);values=(freq*2*math.pi)**2
            if basis.ndim!=2 or basis.shape[0]!=3*len(points) or basis.shape[1]>72:raise ValueError('invalid ROM basis')
            if not np.allclose(basis.T@(K@basis),r['rom']['stiffness_reduced'],rtol=1e-7,atol=1e-7) or not np.allclose(basis.T@(M@basis),r['rom']['mass_reduced'],rtol=1e-7,atol=1e-10):raise ValueError('ROM projection does not match full FE matrices')
            origin=ports[0]['origin'];local=points-origin;fem=FEMinterface();fem.nodes={'Position':local};fem.elements=[{'Tet4' if tets.shape[1]==4 else 'Tet10':tets}];fem.surface=[{'Trigs':boundary}];fem.massMatrix=M;fem.stiffnessMatrix=K;fem.modeBasis={'matrix':basis,'type':'HCBmodes'};fem.eigenValues=values
            obj=ObjectFFRFreducedOrderInterface(fem).AddObjectFFRFreducedOrder(mbs,positionRef=origin.tolist(),gravity=gravity.tolist(),stiffnessProportionalDamping=2*.01/math.sqrt(min(values)))
            ids[bid]=obj['oFFRFreducedOrder'];flex[bid]=dict(component_id=b['flexible_component'],origin=origin,local=local,ports=ports,basis=basis,M=M,K=K,Kr=fe.finite(r['rom']['stiffness_reduced']),mass=fe_mass,record=record,parts=parts,attachments={},couplers={})
            for name,ov in ovs.items():sensors[bid][name]=mbs.AddSensor(SensorNode(nodeNumber=obj['nRigidBody'],outputVariableType=ov,storeInternal=True,writeToFile=False))
            for name,ov in [('modal',exu.OutputVariableType.Coordinates),('modal_velocity',exu.OutputVariableType.Coordinates_t)]:sensors[bid][name]=mbs.AddSensor(SensorNode(nodeNumber=obj['nGenericODE2'],outputVariableType=ov,storeInternal=True,writeToFile=False))
        else:
            ids[bid]=mbs.CreateRigidBody(name=bid,referencePosition=com.tolist(),inertia=RigidBodyInertia(mass=mass,inertiaTensor=tensor,inertiaTensorAtCOM=True),gravity=gravity.tolist())
            for name,ov in ovs.items():sensors[bid][name]=mbs.AddSensor(SensorBody(bodyNumber=ids[bid],outputVariableType=ov,storeInternal=True,writeToFile=False))
    def surface_marker(bid, pivot, port_id=None):
        f=flex[bid];ports=[p for p in f['ports'] if (p['id']==port_id if port_id else np.linalg.norm(p['origin']-pivot)<1e-7)]
        if len(ports)!=1:raise ValueError(f'{bid}: joint/load must resolve to exactly one reduced CAD surface; define its face-centred joint interface')
        p=ports[0]
        if np.linalg.norm(p['origin']-pivot)>1e-7:raise ValueError('load/joint origin differs from ROM surface frame')
        offset=p['origin']-f['origin']-np.average(f['local'][p['nodes']],axis=0,weights=p['weights'])
        return mbs.AddMarker(MarkerSuperElementRigid(bodyNumber=ids[bid],meshNodeNumbers=p['nodes'].tolist(),weightingFactors=p['weights'].tolist(),offset=offset.tolist()))
    for bid, f in flex.items():
        groups = {}
        for attachment in f['record'].get('rigid_attachments', []):
            matches = [p for p in f['parts'] if same_part(p, attachment['owner'])]
            if len(matches) != 1:
                raise ValueError('rigid attachment physical part missing or ambiguous')
            groups.setdefault(attachment['interface'], []).append(matches[0])
        if len(groups) > 8:
            raise ValueError('at most eight rigid attachment groups per flexible block')
        # An explicit surface-to-joint mapping is an ideal Rigid Transform.
        # A zero-inertia, fully constrained frame transports the actual surface
        # rotation to an offset joint without inventing a massive CAD part.
        for assignment in f['record'].get('joint_interfaces', []):
            groups.setdefault(assignment['interface'], [])
        for port_id, parts in groups.items():
            ports = [p for p in f['ports'] if p['id'] == port_id]
            if len(ports) != 1:
                raise ValueError('rigid attachment needs a reduced CAD surface')
            pivot = ports[0]['origin']
            if parts:
                mass, com, tensor = aggregate_parts(parts)
                obj = mbs.CreateRigidBody(name=bid + ':' + port_id, referencePosition=com.tolist(),
                                          inertia=RigidBodyInertia(mass=mass, inertiaTensor=tensor, inertiaTensorAtCOM=True),
                                          gravity=gravity.tolist())
            else:
                mass, com, tensor = 0., pivot, np.zeros((3, 3))
                node = mbs.AddNode(NodeRigidBodyEP(referenceCoordinates=com.tolist() + [1., 0, 0, 0]))
                obj = mbs.AddObject(ObjectRigidBody(name=bid + ':' + port_id + ':frame', nodeNumber=node, mass=0., inertia=[0.]*6))
            markers = [surface_marker(bid, pivot, port_id),
                       mbs.AddMarker(MarkerBodyRigid(bodyNumber=obj, localPosition=(pivot-com).tolist()))]
            mbs.AddObject(GenericJoint(name=bid + ':' + port_id + ':bond', markerNumbers=markers, constrainedAxes=[1]*6))
            attachment_constraints.append([{name:mbs.AddSensor(SensorMarker(markerNumber=m, outputVariableType=ov, storeInternal=True, writeToFile=False))
                                            for name, ov in ovs.items()} for m in markers])
            f['couplers'][port_id] = dict(object=obj, com=com)
            if parts:
                f['attachments'][port_id] = dict(object=obj, mass=mass, com=com, inertia=tensor)
                attachment_sensors[bid, port_id] = {name:mbs.AddSensor(SensorBody(bodyNumber=obj, outputVariableType=ov, storeInternal=True, writeToFile=False))
                                                   for name, ov in ovs.items()}
    def marker(bid, pivot, port_id=None):
        if bid not in flex:
            return mbs.AddMarker(MarkerBodyRigid(bodyNumber=ids[bid], localPosition=(pivot-initial[bid]).tolist()))
        f = flex[bid]
        port_id = f['record'].get('port_attachments', {}).get(port_id, port_id)
        if port_id in f['couplers']:
            attachment = f['couplers'][port_id]
            return mbs.AddMarker(MarkerBodyRigid(bodyNumber=attachment['object'], localPosition=(pivot-attachment['com']).tolist()))
        return surface_marker(bid, pivot, port_id)
    def joint_marker(bid, joint, pivot):
        assignments = flex[bid]['record'].get('joint_interfaces', []) if bid in flex else []
        matches = [a['interface'] for a in assignments if a['joint'] == joint]
        if len(matches) > 1:
            raise ValueError('ambiguous joint ROM interface')
        return marker(bid, pivot, matches[0] if matches else None)
    drivers={d['joint']:d for d in model.get('drivers',[])}
    if initial_equilibrium and (len(drivers) != len(model['joints']) or any(j['id'] not in drivers for j in model['joints'])):
        raise ValueError('static initial equilibrium needs prescribed motion on every joint')
    joints=[]
    moment_sensors={}
    for j in model['joints']:
        a,b=j['a'],j['b'];pivot=fe.finite(j['pivot_world_m'],(3,));rotation=fe.finite(j['rotation_world'],(3,3))
        if a not in ids or b not in ids or a==b or j['kind'] not in ['revolute','slider'] or not np.allclose(rotation.T@rotation,np.eye(3),atol=1e-8):raise ValueError('invalid joint')
        markers=[joint_marker(a,j['id'],pivot),joint_marker(b,j['id'],pivot)];drive=drivers.get(j['id']);axes=[1]*6 if drive else ([1,1,1,1,1,0] if j['kind']=='revolute' else [1,1,0,1,1,1])
        offset, velocity = joint_motion(drive, j['kind'], duration, frozen=lambda:static_phase) if drive else (0, 0)
        obj=mbs.AddObject(GenericJoint(name=j['name'],markerNumbers=markers,rotationMarker0=rotation.tolist(),rotationMarker1=rotation.tolist(),constrainedAxes=axes,offsetUserFunction=offset,offsetUserFunction_t=velocity))
        reaction=mbs.AddSensor(SensorObject(objectNumber=obj,outputVariableType=exu.OutputVariableType.ForceLocal,storeInternal=True,writeToFile=False))
        frames=[{name:mbs.AddSensor(SensorMarker(markerNumber=m,outputVariableType=ov,storeInternal=True,writeToFile=False)) for name,ov in ovs.items()} for m in markers]
        moment_sensors[str(j['id'])]=mbs.AddSensor(SensorObject(objectNumber=obj,outputVariableType=exu.OutputVariableType.TorqueLocal,storeInternal=True,writeToFile=False))
        joints.append((j,reaction,frames))
    for load in model['loads']:
        m=marker(load['block'],fe.finite(load['position_world_m'],(3,)),load['port'])
        mbs.AddLoad(LoadForceVector(markerNumber=m,loadVector=fe.finite(load['force_initial_world_n'],(3,)).tolist(),bodyFixed=True));mbs.AddLoad(LoadTorqueVector(markerNumber=m,loadVector=fe.finite(load['moment_initial_world_nm'],(3,)).tolist(),bodyFixed=True))
    mbs.Assemble()
    setup_finished = perf_counter()
    initialization = dict(method='undeformed', static_converged=False, held_joints=0)
    if initial_equilibrium:
        static_options = exu.SimulationSettings()
        static_options.solution.file.write = False
        static_options.solution.sensors.active = False
        static_options.staticSolver.verboseMode = 0
        static_options.staticSolver.numberOfLoadSteps = 8
        static_options.staticSolver.newton.relativeTolerance = 1e-8
        static_options.staticSolver.newton.absoluteTolerance = 1e-10
        static_phase = True
        try:
            converged = mbs.SolveStatic(static_options, updateInitialValues=True)
            if not converged:
                raise ValueError('static preload did not converge')
            initialization = dict(
                method='static_equilibrium', static_converged=True, held_joints=len(joints),
                load_steps=8, relative_tolerance=1e-8, absolute_tolerance=1e-10,
                joint_forces_local_n={str(j['id']):fe.finite(mbs.GetSensorValues(sensor), (3,)).tolist() for j,sensor,_ in joints},
                joint_torques_local_nm={jid:fe.finite(mbs.GetSensorValues(sensor), (3,)).tolist() for jid,sensor in moment_sensors.items()})
        finally:
            static_phase = False
        coordinates = mbs.systemData.GetODE2Coordinates(exu.ConfigurationType.Initial)
        mbs.systemData.SetODE2Coordinates_t(np.zeros_like(coordinates), exu.ConfigurationType.Initial)
        mbs.systemData.SetTime(0., exu.ConfigurationType.Initial)
    static_finished = perf_counter()
    options=exu.SimulationSettings();options.timeIntegration.endTime=duration;options.timeIntegration.numberOfSteps=steps;options.timeIntegration.verboseMode=0;options.timeIntegration.generalizedAlpha.spectralRadius=.8;options.solution.file.write=False;options.solution.sensors.writePeriod=duration/(samples-1)
    mbs.SolveDynamic(options,solverType=exu.DynamicSolverType.GeneralizedAlpha)
    integration_finished = perf_counter()
    def data(sensor):
        a=mbs.GetSensorStoredData(sensor)
        if a.shape[0]!=samples or not np.isfinite(a).all() or not np.allclose(a[:,0], np.linspace(0, duration, samples), rtol=0, atol=1e-8):raise ValueError('incomplete/non-finite or misaligned sensor output')
        return a[:,1:]
    arrays={bid:{name:data(sensor) for name,sensor in ss.items()} for bid,ss in sensors.items()};times=np.linspace(0,duration,samples);trajectories={};states={};energy=np.zeros((samples,3))
    for bid,a in arrays.items():
        rotation=a['rotation'].reshape(-1,3,3);pos=a['position'];mass,tensor=physical[bid]
        if bid in flex:
            f=flex[bid];q=a['modal'];qd=a['modal_velocity'];states[bid]=dict(component_id=f['component_id'],modal_coordinates=q.tolist(),reference_poses=np.column_stack([pos,a['rotation']]).tolist())
            states[bid]['max_elastic_displacement_mm'] = float(max(np.max(np.linalg.norm((f['basis'] @ row).reshape(-1,3), axis=1)) for row in q) * 1000)
            states[bid]['initial_elastic_displacement_mm'] = float(np.max(np.linalg.norm((f['basis'] @ q[0]).reshape(-1,3), axis=1)) * 1000)
            trajectories[bid]=np.column_stack([pos+np.einsum('kij,j->ki',rotation,initial[bid]-f['origin']),a['rotation']]).tolist()
            for k in range(samples):
                u=(f['basis']@q[k]).reshape(-1,3);ud=(f['basis']@qd[k]).reshape(-1,3);w=rotation[k].T@a['omega'][k];v=rotation[k].T@a['velocity'][k]+np.cross(w,f['local']+u)+ud
                energy[k,0]+=.5*v.ravel()@(f['M']@v.ravel());mass_nodes=np.asarray(f['M'].sum(axis=1)).ravel().reshape(-1,3)[:,0]
                nodal_world=pos[k]+(f['local']+u)@rotation[k].T
                energy[k,1]+=.5*q[k]@(f['Kr']@q[k])-np.sum(mass_nodes*(nodal_world@gravity))
        else:
            trajectories[bid]=np.column_stack([pos,a['rotation']]).tolist()
            for k in range(samples):energy[k,0]+=.5*mass*(a['velocity'][k]@a['velocity'][k])+.5*a['omega'][k]@(rotation[k]@tensor@rotation[k].T)@a['omega'][k];energy[k,1]-=mass*(gravity@pos[k])
    coordinates={};reactions={};max_pos=0.;max_axis=0.
    for (bid, port_id), ss in attachment_sensors.items():
        a = {name:data(sensor) for name,sensor in ss.items()}
        attachment = flex[bid]['attachments'][port_id]
        rotation = a['rotation'].reshape(-1, 3, 3)
        states[bid].setdefault('attachment_states', {})[port_id] = dict(
            com_world_m=attachment['com'].tolist(), poses=np.column_stack([a['position'], a['rotation']]).tolist())
        for k in range(samples):
            energy[k, 0] += .5*attachment['mass']*(a['velocity'][k]@a['velocity'][k]) + .5*a['omega'][k]@(rotation[k]@attachment['inertia']@rotation[k].T)@a['omega'][k]
            energy[k, 1] -= attachment['mass']*(gravity@a['position'][k])
    for frames in attachment_constraints:
        a, b = [{name:data(sensor) for name,sensor in frame.items()} for frame in frames]
        max_pos = max(max_pos, float(np.max(np.linalg.norm(a['position']-b['position'], axis=1))))
        max_axis = max(max_axis, float(np.max(np.linalg.norm(a['rotation']-b['rotation'], axis=1))))
    for j,sensor,ss in joints:
        a,b=[{name:data(sensor) for name,sensor in s.items()} for s in ss];basis=fe.finite(j['rotation_world'],(3,3));values=[]
        for k in range(samples):
            ra=a['rotation'][k].reshape(3,3)@basis;rb=b['rotation'][k].reshape(3,3)@basis;z=ra[:,2];delta=b['position'][k]-a['position'][k];q0=float(j['native_values_at_start'][0])*(1 if j['kind']=='revolute' else .001)
            if j['kind']=='revolute':relative=ra.T@rb;q=math.atan2(relative[1,0],relative[0,0])+q0;v=(b['omega'][k]-a['omega'][k])@z
            else:q=delta@z+q0;v=(b['velocity'][k]-a['velocity'][k])@z+delta@np.cross(a['omega'][k],z);delta-=z*(delta@z)
            values.append([float(q),float(v)]);max_pos=max(max_pos,float(np.linalg.norm(delta)));max_axis=max(max_axis,float(np.linalg.norm(np.cross(z,rb[:,2]))))
            if j['kind']=='slider':max_axis=max(max_axis,float(np.linalg.norm(ra-rb)))
        values=np.array(values)
        if j['kind']=='revolute':values[:,0]=np.unwrap(values[:,0])
        coordinates[str(j['id'])]=values.tolist();reactions[str(j['id'])]=data(sensor).tolist()
    if max_pos>1e-5 or max_axis>1e-5:raise ValueError('deformed interface joint residual exceeds tolerance')
    energy[:,2]=energy[:,0]+energy[:,1]
    initial_note = ('Static gravity/port-load equilibrium at held CAD joint coordinates; zero initial velocity.'
                    if initial_equilibrium else 'Zero initial elastic displacement/velocity; no static prestress initialization.')
    return dict(
        schema='solidcraft.assembly-result/1', status='success', job=job, backend_version=exu.__version__,
        initialization=initialization, times_s=times.tolist(), trajectories=trajectories,
        flexible_states=states, energy_j=energy.tolist(), joint_forces_local_n=reactions,
        joint_torques_local_nm={joint_id:data(sensor).tolist() for joint_id,sensor in moment_sensors.items()},
        joint_coordinates_si=coordinates,
        numerics=dict(generalized_alpha_spectral_radius=.8, force_initial_sample_valid=False),
        timings_s=dict(model_setup=setup_finished-started, static_initialization=static_finished-setup_finished,
                       integration=integration_finished-static_finished, postprocessing=perf_counter()-integration_finished,
                       total=perf_counter()-started),
        summary=dict(max_position_constraint_m=max_pos, max_axis_alignment_error=max_axis,
                     max_energy_change_j=float(max(abs(energy[:,2]-energy[0,2])))),
        provenance=dict(motion_profiles_sha256=SOURCE_SHA256,
                        worker_sha256=sha256(Path(__file__).read_bytes()).hexdigest(),
                        job_sha256=sha256(json.dumps(job,sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()).hexdigest(),
                        completed_utc=datetime.now(timezone.utc).isoformat()),
        assumptions=[
            'Mixed rigid/FFRF open chain; stiffness-proportional damping calibrated to 1% at the lowest fixed-interface frequency.',
            'Flexible joints and loads act through reduced CAD surface node sets. Explicit surface-to-joint mappings use ideal massless rigid-transform frames; these add no hardware mass or support flexibility.',
            'Small elastic deformation, RBE2 rigid surfaces; no contact/plasticity.', initial_note,
            'CAD attachments retain their mass and centroidal inertia, bonded rigidly to their explicitly selected surface. Their motion and energy are included separately. No bolt/contact compliance is inferred.',
            'Energy includes full FE nodal kinetic energy, attachment rigid kinetic energy, elastic strain and gravitational potential; actuator/load work is not subtracted.'])


if __name__=='__main__':
    try:
        raw=sys.stdin.read(6*1024*1024+1)
        if len(raw)>6*1024*1024:raise ValueError('flexible assembly input exceeds 6 MiB')
        job=json.loads(raw,parse_constant=lambda v:(_ for _ in ()).throw(ValueError('non-finite JSON')))
        with redirect_stdout(sys.stderr):result=run(job)
        text=json.dumps(result,separators=(',',':'),allow_nan=False)
        if len(text)>12*1024*1024:raise ValueError('flexible output exceeds 12 MiB')
        sys.stdout.write(text)
    except Exception as exc:sys.stdout.write(json.dumps(dict(status='failed',error=str(exc))));raise SystemExit(1)
