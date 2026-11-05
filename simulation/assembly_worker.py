"""Optional local Exudyn adapter for CAD-derived rigid open chains. JSON only, no user code."""
from contextlib import redirect_stdout
from datetime import datetime, timezone
from hashlib import sha256
import argparse
import json
import math
from pathlib import Path
import sys


def run(job):
    import numpy as np
    import exudyn as exu
    from exudyn.utilities import RigidBodyInertia
    from exudyn.itemInterface import (ObjectGround, SensorBody, SensorObject,
        MarkerBodyRigid, LoadForceVector, LoadTorqueVector)
    if job.get('schema')!='solidcraft.assembly-job/1' or job.get('backend')!='exudyn_rigid_open_chain':
        raise ValueError('unsupported assembly job')
    settings=job['settings']; model=job['model']
    duration=float(settings['duration_s']);steps=int(settings['steps']);samples=int(settings['samples'])
    if not (.001<=duration<=10 and 10<=steps<=10000 and 2<=samples<=1001 and (steps%(samples-1)==0)):
        raise ValueError('invalid integration or sample settings')
    if not 1<=len(model['bodies'])<=8 or len(model['bodies'])*(samples-1)>8000:
        raise ValueError('body/sample limit exceeded')
    gravity=np.array(settings['gravity_m_s2'],dtype=float)
    if gravity.shape!=(3,) or not np.isfinite(gravity).all() or np.max(np.abs(gravity))>100:
        raise ValueError('invalid gravity')
    sc=exu.SystemContainer();mbs=sc.AddSystem()
    ground=mbs.AddObject(ObjectGround(referencePosition=[0.,0.,0.]))
    ids={model['ground']:ground}; initial={model['ground']:np.zeros(3)}
    sensors={}; inertias={}; masses={}
    for b in model['bodies']:
        bid=b['id'];mass=float(b['mass_kg']);com=np.array(b['com_world_m'],dtype=float)
        tensor=np.array(b['inertia_com_world_kg_m2'],dtype=float)
        if (bid in ids or not (0<mass<1e9) or com.shape!=(3,) or tensor.shape!=(3,3)
            or not np.isfinite(com).all() or not np.isfinite(tensor).all()
            or np.max(np.abs(tensor-tensor.T))>max(1e-14,np.max(np.abs(tensor))*1e-8)):
            raise ValueError('invalid body mass properties')
        eig=np.linalg.eigvalsh(tensor)
        if min(eig)<=0 or max(eig)>sum(eig)-max(eig)+max(eig)*1e-6:
            raise ValueError('non-physical inertia tensor')
        body=mbs.CreateRigidBody(name=bid,referencePosition=com.tolist(),referenceRotationMatrix=np.eye(3),
            inertia=RigidBodyInertia(mass=mass,inertiaTensor=tensor,inertiaTensorAtCOM=True),gravity=gravity.tolist())
        ids[bid]=body;initial[bid]=com;inertias[bid]=tensor;masses[bid]=mass
        sensors[bid]={}
        for name,ov in [('position',exu.OutputVariableType.Position),('rotation',exu.OutputVariableType.RotationMatrix),
                        ('velocity',exu.OutputVariableType.Velocity),('omega',exu.OutputVariableType.AngularVelocity)]:
            sensors[bid][name]=mbs.AddSensor(SensorBody(bodyNumber=body,outputVariableType=ov,storeInternal=True,writeToFile=False))
    joints=[]
    moment_sensors={}
    drivers={drive['joint']:drive for drive in model.get('drivers',[])}
    def offset_function(drive,kind):
        amplitude=float(drive['amplitude_si']);ramp=float(drive['ramp_s'])
        profile=drive.get('profile','cosine');frequency=float(drive.get('frequency_hz',1.))
        if profile=='half_cosine_displacement_then_hold':profile='cosine'
        if profile not in ('cosine','quintic','sine') or not math.isfinite(frequency) or not .01<=frequency<=20:
            raise ValueError('invalid motion source')
        if not math.isfinite(amplitude) or not 0<ramp<=duration:raise ValueError('invalid driver')
        def offset(mbs,t,itemNumber,parameters):
            phase=min(max(t/ramp,0.),1.)
            smooth=phase**3*(10+phase*(-15+6*phase))
            value=(.5*(1-math.cos(math.pi*phase)) if profile=='cosine' else smooth if profile=='quintic' else smooth*math.sin(2*math.pi*frequency*t))
            result=[0.]*6;result[5 if kind=='revolute' else 2]=amplitude*value
            return result
        return offset
    for j in model['joints']:
        a,b=j['a'],j['b'];pivot=np.array(j['pivot_world_m']);axis=np.array(j['axis_world'])
        if a not in ids or b not in ids or a==b or j['kind'] not in ('revolute','slider'):
            raise ValueError('invalid joint')
        z=axis/np.linalg.norm(axis)
        rotation=np.array(j['rotation_world'],dtype=float)
        if rotation.shape!=(3,3) or not np.allclose(rotation.T@rotation,np.eye(3),atol=1e-8) or np.linalg.det(rotation)<.999999:
            raise ValueError('invalid joint frame')
        drive=drivers.get(j['id'])
        constrained=[1]*6 if drive else ([1,1,1,1,1,0] if j['kind']=='revolute' else [1,1,0,1,1,1])
        obj=mbs.CreateGenericJoint(name=j['name'],bodyNumbers=[ids[a],ids[b]],position=pivot.tolist(),
            rotationMatrixAxes=rotation,constrainedAxes=constrained,offsetUserFunction=offset_function(drive,j['kind']) if drive else 0)
        sensor=mbs.AddSensor(SensorObject(objectNumber=obj,outputVariableType=exu.OutputVariableType.ForceLocal,
                                        storeInternal=True,writeToFile=False))
        moment_sensors[str(j['id'])]=mbs.AddSensor(SensorObject(objectNumber=obj,outputVariableType=exu.OutputVariableType.TorqueLocal,storeInternal=True,writeToFile=False))
        joints.append((j,pivot-initial[a],pivot-initial[b],z,sensor))
    for load in model['loads']:
        bid=load['block'];position=np.array(load['position_world_m'])-initial[bid]
        marker=mbs.AddMarker(MarkerBodyRigid(bodyNumber=ids[bid],localPosition=position.tolist()))
        mbs.AddLoad(LoadForceVector(markerNumber=marker,loadVector=load['force_initial_world_n'],bodyFixed=True))
        mbs.AddLoad(LoadTorqueVector(markerNumber=marker,loadVector=load['moment_initial_world_nm'],bodyFixed=True))
    mbs.Assemble()
    options=exu.SimulationSettings()
    options.timeIntegration.numberOfSteps=steps;options.timeIntegration.endTime=duration
    options.timeIntegration.verboseMode=0;options.timeIntegration.generalizedAlpha.spectralRadius=.6 if drivers else 1.
    options.solutionSettings.writeSolutionToFile=False;options.solutionSettings.sensorsWritePeriod=duration/(samples-1)
    mbs.SolveDynamic(options,solverType=exu.DynamicSolverType.GeneralizedAlpha)
    arrays={bid:{k:mbs.GetSensorStoredData(sn) for k,sn in ss.items()} for bid,ss in sensors.items()}
    times=next(iter(arrays.values()))['position'][:,0]
    if len(times)!=samples or abs(times[-1]-duration)>1e-8 or abs(times[0])>1e-8:
        raise ValueError('missing integration samples')
    for data in arrays.values():
        if any(not np.isfinite(a).all() or a.shape[0]!=samples or not np.allclose(a[:,0],times,atol=1e-10) for a in data.values()):
            raise ValueError('inconsistent sensor output')
    trajectories={bid:np.column_stack([data['position'][:,1:],data['rotation'][:,1:]]).tolist() for bid,data in arrays.items()}
    energy=[];max_pos=0.;max_axis=0.;coordinates={str(j['id']):[] for j,_,_,_,_ in joints}
    for k in range(samples):
        kinetic=0.;potential=0.
        for bid,data in arrays.items():
            velocity=data['velocity'][k,1:];omega=data['omega'][k,1:];rotation=data['rotation'][k,1:].reshape(3,3)
            kinetic+=.5*masses[bid]*np.dot(velocity,velocity)+.5*omega@(rotation@inertias[bid]@rotation.T)@omega
            potential-=masses[bid]*np.dot(gravity,data['position'][k,1:])
        energy.append([float(kinetic),float(potential),float(kinetic+potential)])
        def pose(bid):
            if bid==model['ground']:return np.zeros(3),np.eye(3)
            d=arrays[bid];return d['position'][k,1:],d['rotation'][k,1:].reshape(3,3)
        def speed(bid):
            if bid==model['ground']:return np.zeros(3),np.zeros(3)
            d=arrays[bid];return d['velocity'][k,1:],d['omega'][k,1:]
        for j,pa,pb,z,_ in joints:
            ca,ra=pose(j['a']);cb,rb=pose(j['b']);delta=cb+rb@pb-ca-ra@pa
            za=ra@z;zb=rb@z
            va,wa=speed(j['a']);vb,wb=speed(j['b'])
            # mks.model exposes expressed angular coordinates in radians, distances in mm.
            q0=float(j['native_values_at_start'][0])*(1. if j['kind']=='revolute' else .001)
            if j['kind']=='revolute':
                basis=np.asarray(j['rotation_world']);relative=basis.T@ra.T@rb@basis
                q=math.atan2(float(relative[1,0]),float(relative[0,0]))+q0
                velocity=float(np.dot(wb-wa,za))
            else:
                q=float(np.dot(delta,za))+q0
                velocity=float(np.dot(vb+np.cross(wb,rb@pb)-va-np.cross(wa,ra@pa),za)+np.dot(delta,np.cross(wa,za)))
            coordinates[str(j['id'])].append([q,velocity])
            if j['kind']=='slider':delta=delta-za*np.dot(delta,za)
            max_pos=max(max_pos,float(np.linalg.norm(delta)))
            max_axis=max(max_axis,float(np.linalg.norm(np.cross(za,zb))))
            if j['kind']=='slider':max_axis=max(max_axis,float(np.linalg.norm(ra-rb)))
    if max_pos>1e-5 or max_axis>1e-5:raise ValueError('joint residual exceeds adapter tolerance')
    for j,_,_,_,_ in joints:
        if j['kind']=='revolute':
            values=np.asarray(coordinates[str(j['id'])]);values[:,0]=np.unwrap(values[:,0]);coordinates[str(j['id'])]=values.tolist()
    reactions={str(j['id']):mbs.GetSensorStoredData(sensor)[:,1:].tolist() for j,_,_,_,sensor in joints}
    moments={}
    for joint_id,sensor in moment_sensors.items():
        data=mbs.GetSensorStoredData(sensor)
        if data.shape!=(samples,4) or not np.isfinite(data).all() or not np.allclose(data[:,0],times,atol=1e-10):
            raise ValueError('inconsistent joint torque output')
        moments[joint_id]=data[:,1:].tolist()
    energy_change=max(abs(e[2]-energy[0][2]) for e in energy)
    return dict(schema='solidcraft.assembly-result/1',status='success',job=job,backend_version=exu.__version__,
        times_s=times.tolist(),trajectories=trajectories,energy_j=energy,joint_forces_local_n=reactions,joint_torques_local_nm=moments,joint_coordinates_si=coordinates,
        numerics=dict(generalized_alpha_spectral_radius=.6 if drivers else 1.,force_initial_sample_valid=False),
        summary=dict(max_position_constraint_m=max_pos,max_axis_alignment_error=max_axis,max_energy_change_j=energy_change),
        provenance=dict(worker_sha256=sha256(Path(__file__).read_bytes()).hexdigest(),
            job_sha256=sha256(json.dumps(job,sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()).hexdigest(),
            completed_utc=datetime.now(timezone.utc).isoformat()),
        assumptions=['Zero initial velocities; current CAD pose is the reference pose.',
            'Generalized-alpha spectral radius: 0.6 for prescribed drivers to damp unresolved constraint-force modes, 1.0 for free motion. No output filtering. Cosine-source acceleration jumps need time-step checks.',
            'Joint forces use the Exudyn generic-joint local frame; body loads rotate with their block.',
            'Joint q and velocity are measured from relative body poses and velocities, including the native initial joint value; radians/metres and SI time derivatives. Rotation is unwrapped at output times; adequate sampling is required.',
            'Energy contains kinetic and gravitational potential energy; port-load and actuator work is not subtracted.'])


def main():
    parser=argparse.ArgumentParser();parser.add_argument('job',nargs='?',type=Path);parser.add_argument('--out',type=Path)
    args=parser.parse_args()
    try:
        raw=args.job.read_text(encoding='utf-8-sig') if args.job else sys.stdin.read(2000001)
        if len(raw)>2000000:raise ValueError('job exceeds 2 MB')
        job=json.loads(raw,parse_constant=lambda x: (_ for _ in ()).throw(ValueError('non-finite number')))
        with redirect_stdout(sys.stderr):result=run(job)
        text=json.dumps(result,ensure_ascii=False,allow_nan=False,separators=(',',':'))
        if args.out:
            with args.out.open('x',encoding='utf-8') as stream:stream.write(text)
        else:sys.stdout.write(text)
        return 0
    except Exception as exc:
        sys.stdout.write(json.dumps(dict(status='failed',error=str(exc)),ensure_ascii=False));return 1


if __name__=='__main__':raise SystemExit(main())
