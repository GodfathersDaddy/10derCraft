"""Bounded local solid FEM and component-mode synthesis. SI internally; JSON only.

Original implementation. TET4/TET10 elasticity with consistent mass. Interface
nodes follow rigid surface frames (RBE2); this is NOT distributing/RBE3 coupling.
The first interface is the floating reference, excluded from the elastic basis.
"""
from contextlib import redirect_stdout
from hashlib import sha256
from pathlib import Path
import json,math,sys
import numpy as np
from motion_profiles import joint_motion, SOURCE_SHA256
from scipy import linalg,sparse
from scipy.sparse import linalg as spla
EDGES=[(0,1),(1,2),(0,2),(0,3),(1,3),(2,3)]

def quadratic_mesh(points,tets,boundary):
    nodes=list(points);edges={};connectivity=[]
    for tet in tets:
        mids=[]
        for a,b in EDGES:
            key=tuple(sorted([int(tet[a]),int(tet[b])]))
            if key not in edges:edges[key]=len(nodes);nodes.append((points[key[0]]+points[key[1]])/2)
            mids.append(edges[key])
        connectivity.append([*map(int,tet),*mids])
    surface=[]
    for a,b,c in boundary:
        ab=edges[tuple(sorted([int(a),int(b)]))];bc=edges[tuple(sorted([int(b),int(c)]))];ca=edges[tuple(sorted([int(c),int(a)]))]
        surface.extend([[a,ab,ca],[ab,b,bc],[ca,bc,c],[ab,bc,ca]])
    if len(nodes)>1500:raise ValueError('quadratic mesh exceeds 1500-node limit; use a smaller mesh')
    return np.asarray(nodes),np.asarray(connectivity,dtype=int),np.asarray(surface,dtype=int)

def mass_coefficients(order):
    if order==1:return (np.ones((4,4))+np.eye(4))/20
    polynomials=[]
    for i in range(4):
        a=[0]*4;a[i]=2;b=[0]*4;b[i]=1;polynomials.append([(2,a),(-1,b)])
    for i,j in EDGES:
        a=[0]*4;a[i]=1;a[j]=1;polynomials.append([(4,a)])
    result=np.zeros((10,10))
    for i,p in enumerate(polynomials):
        for j,q in enumerate(polynomials):
            for ci,ai in p:
                for cj,aj in q:
                    power=np.asarray(ai)+aj
                    result[i,j]+=ci*cj*6*math.prod(math.factorial(int(e)) for e in power)/math.factorial(3+sum(map(int,power)))
    return result

def strain_matrix(grad):
    B=np.zeros((6,3*len(grad)))
    for i,(x,y,z) in enumerate(grad):B[:,3*i:3*i+3]=[[x,0,0],[0,y,0],[0,0,z],[y,x,0],[0,z,y],[z,0,x]]
    return B

def finite(value,shape=None):
    a=np.asarray(value,dtype=float)
    if not np.isfinite(a).all() or (shape is not None and a.shape!=shape):raise ValueError('invalid finite array or shape')
    return a

def box_mesh(shape,divisions):
    a=finite(shape['corner_mm'],(3,));size=finite(shape['size_mm'],(3,))
    raw=np.asarray(divisions)
    if raw.shape!=(3,) or not np.issubdtype(raw.dtype,np.integer):raise ValueError('integer mesh divisions required')
    n=raw.astype(int)
    if n.shape!=(3,) or min(n)<1 or max(n)>16 or min(size)<=0:raise ValueError('invalid box mesh')
    nodes=np.array([a+size*np.array([i/n[0],j/n[1],k/n[2]]) for i in range(n[0]+1) for j in range(n[1]+1) for k in range(n[2]+1)])
    index=lambda i,j,k:(i*(n[1]+1)+j)*(n[2]+1)+k
    tets=[]
    for i in range(n[0]):
        for j in range(n[1]):
            for k in range(n[2]):
                v=[index(i+x,j+y,k+z) for x,y,z in [(0,0,0),(1,0,0),(1,1,0),(0,1,0),(0,0,1),(1,0,1),(1,1,1),(0,1,1)]]
                tets.extend([[v[a],v[b],v[c],v[d]] for a,b,c,d in [(0,1,2,6),(0,2,3,6),(0,3,7,6),(0,7,4,6),(0,4,5,6),(0,5,1,6)]])
    if 'world' in shape:
        transform=finite(shape['world'],(4,4)).T
        nodes=nodes@transform[:3,:3].T+transform[:3,3]
    return nodes,np.asarray(tets,dtype=int)

def cylinder_mesh(shape,divisions):
    from scipy.spatial import Delaunay
    origin=finite(shape['base_mm'],(3,));axis=finite(shape['axis'],(3,));axis/=np.linalg.norm(axis)
    if len(divisions)!=3 or any(isinstance(n,bool) or not isinstance(n,int) for n in divisions):raise ValueError('integer mesh divisions required')
    r=float(shape['radius_mm']);length=float(shape['length_mm']);layers=int(divisions[0]);segments=24
    if not (r>0 and length>0 and 1<=layers<=8):raise ValueError('invalid cylinder mesh')
    seed=np.array([1.,0,0]) if abs(axis[0])<.8 else np.array([0.,1,0])
    y=seed-axis*(seed@axis);y/=np.linalg.norm(y);z=np.cross(axis,y)
    cross=np.array([[0,0]]+[[radius*math.cos(2*math.pi*i/n),radius*math.sin(2*math.pi*i/n)] for radius,n in [(r,segments),(r/2,segments//2)] for i in range(n)])
    triangles=Delaunay(cross).simplices;n=len(cross)
    nodes=np.array([origin+axis*(length*k/layers)+y*p[0]+z*p[1] for k in range(layers+1) for p in cross])
    cells=[]
    for k in range(layers):
        for triangle in triangles:
            a,b,c=sorted(map(int,triangle));a0,b0,c0=np.array([a,b,c])+k*n;a1,b1,c1=np.array([a,b,c])+(k+1)*n
            cells.extend([[a0,b0,c0,c1],[a0,b0,b1,c1],[a0,a1,b1,c1]])
    if 'world' in shape:
        transform=finite(shape['world'],(4,4)).T;nodes=nodes@transform[:3,:3].T+transform[:3,3]
    return nodes,np.asarray(cells,dtype=int)

def tube_mesh(shape, divisions):
    origin = finite(shape['base_mm'], (3,))
    axis = finite(shape['axis'], (3,))
    norm = np.linalg.norm(axis)
    if norm < 1e-12:
        raise ValueError('tube axis is zero')
    axis = axis / norm
    if len(divisions) != 3 or any(isinstance(n, bool) or not isinstance(n, int) for n in divisions):
        raise ValueError('integer mesh divisions required')
    outer = float(shape['radius_mm'])
    inner = float(shape['inner_radius_mm'])
    length = float(shape['length_mm'])
    layers = divisions[0]
    segments = 24
    if not 0 < inner < outer or not length > 0 or not 1 <= layers <= 8:
        raise ValueError('invalid annular tube dimensions or layers')
    seed = np.array([1., 0, 0]) if abs(axis[0]) < .8 else np.array([0., 1, 0])
    y = seed - axis * (seed @ axis)
    y /= np.linalg.norm(y)
    z = np.cross(axis, y)
    cross = np.array([radius * (y * math.cos(2 * math.pi * i / segments)
                               + z * math.sin(2 * math.pi * i / segments))
                      for radius in [inner, outer] for i in range(segments)])
    # A triangulated annulus is extruded with a consistent sorted-node prism
    # split. No cells span the bore, and adjacent prisms share their diagonals.
    triangles = []
    for i in range(segments):
        j = (i + 1) % segments
        triangles.extend([[i, segments + i, segments + j], [i, segments + j, j]])
    count = len(cross)
    nodes = np.array([origin + axis * length * k / layers + p
                      for k in range(layers + 1) for p in cross])
    cells = []
    for k in range(layers):
        for triangle in triangles:
            a, b, c = sorted(triangle)
            a0, b0, c0 = np.array([a, b, c]) + k * count
            a1, b1, c1 = np.array([a, b, c]) + (k + 1) * count
            cells.extend([[a0, b0, c0, c1], [a0, b0, b1, c1], [a0, a1, b1, c1]])
    if 'world' in shape:
        transform = finite(shape['world'], (4, 4)).T
        nodes = nodes @ transform[:3, :3].T + transform[:3, 3]
    return nodes, np.asarray(cells, dtype=int)

def skin(tets):
    faces={}
    for tet in tets:
        for f in [(tet[0],tet[2],tet[1]),(tet[0],tet[1],tet[3]),(tet[1],tet[2],tet[3]),(tet[2],tet[0],tet[3])]:
            key=tuple(sorted(f));faces.setdefault(key,[]).append(list(map(int,f)))
    if any(len(v)>2 for v in faces.values()):raise ValueError('non-manifold tetrahedral mesh')
    return np.array([v[0] for v in faces.values() if len(v)==1],dtype=int)

def on_surface(points,triangles,tolerance):
    found=np.zeros(len(points),dtype=bool)
    for a,b,c in triangles:
        ab=b-a;ac=c-a;n=np.cross(ab,ac);nn=np.linalg.norm(n)
        if nn<1e-18:continue
        ap=points-a
        dist=np.abs(ap@n)/nn
        aa=ab@ab;bb=ab@ac;cc=ac@ac;den=aa*cc-bb*bb
        if den<1e-30:continue
        u=(cc*(ap@ab)-bb*(ap@ac))/den;v=(aa*(ap@ac)-bb*(ap@ab))/den
        found|=(dist<=tolerance)&(u>=-1e-8)&(v>=-1e-8)&(u+v<=1+1e-8)
    return np.where(found)[0]

def prepare_mesh(job):
    mesh=job['mesh']
    if 'shape' in mesh:
        kind=mesh['shape'].get('kind')
        meshers = {'box': box_mesh, 'cylinder': cylinder_mesh, 'tube': tube_mesh}
        if kind not in meshers:raise ValueError('automatic meshing supports verified boxes/cylinders/tubes only; import a conforming TET4 mesh for other geometry')
        points,tets=meshers[kind](mesh['shape'],mesh.get('divisions',[4,2,2]))
    else:
        points=finite(mesh['nodes_mm']);raw=np.asarray(mesh['tetrahedra'])
        if not np.issubdtype(raw.dtype,np.integer):raise ValueError('tetrahedron indices must be integers')
        tets=raw.astype(int)
    if points.ndim!=2 or points.shape[1]!=3 or not 4<=len(points)<=512 or np.max(np.abs(points))>1e7:raise ValueError('mesh requires 4..512 finite CAD nodes')
    if tets.ndim!=2 or tets.shape[1]!=4 or not 1<=len(tets)<=3000 or min(tets.flat)<0 or max(tets.flat)>=len(points):raise ValueError('invalid TET4 connectivity')
    if len(set(map(tuple,np.sort(tets,axis=1))))!=len(tets) or any(len(set(t))!=4 for t in tets):raise ValueError('duplicate or collapsed tetrahedron')
    if set(tets.flat)!=set(range(len(points))):raise ValueError('unused mesh nodes')
    points=points*.001
    for tet in tets:
        p=points[tet];det=np.linalg.det((p[1:]-p[0]).T)
        scale=max(np.linalg.norm(p[i]-p[j]) for i in range(4) for j in range(i))
        if abs(det)<max(1e-24,scale**3*1e-9):raise ValueError('degenerate tetrahedron')
        if det<0:tet[1],tet[2]=tet[2],tet[1]
    boundary=skin(tets)
    order=mesh.get('element_order',2)
    if order not in (1,2) or isinstance(order,bool):raise ValueError('element_order must be 1 or 2')
    if order==2:points,tets,boundary=quadratic_mesh(points,tets,boundary)
    ports=[];used=set()
    specs=job['interfaces']
    if not 1<=len(specs)<=8 or len({p['id'] for p in specs})!=len(specs):raise ValueError('one to eight distinct surface interfaces required')
    for item in specs:
        triangles=finite(item['triangles_mm'])*.001
        if triangles.ndim!=3 or triangles.shape[1:]!=(3,3) or len(triangles)>2048:raise ValueError('invalid CAD face triangles')
        if 'disk' in item:
            disk=item['disk'];center=finite(disk['center_mm'],(3,))*.001;normal=finite(disk['normal'],(3,));normal/=np.linalg.norm(normal)
            distance=(points-center)@normal;radial=points-center-distance[:,None]*normal
            radii = np.linalg.norm(radial, axis=1)
            # Linear geometry chords lie slightly inside the analytic inner
            # circle. Include them using the chord bound of the 24-sided mesh.
            inner = float(disk.get('inner_radius_mm', 0)) * .001 * math.cos(math.pi / 24)
            nodes=np.where((np.abs(distance)<1e-8)&(radii<=float(disk['radius_mm'])*.001*(1+1e-7))&(radii>=inner-1e-10))[0]
        else:nodes=on_surface(points,triangles,1e-8)
        if len(nodes)<3:raise ValueError(f"{item['id']}: fewer than three FE nodes on the actual CAD face; refine/import a matching mesh")
        if used.intersection(map(int,nodes)):raise ValueError('overlapping interface node sets')
        used.update(map(int,nodes))
        origin=finite(item['origin_mm'],(3,))*.001
        r=points[nodes]-origin
        R=np.zeros((3*len(nodes),6))
        for i,(x,y,z) in enumerate(r):
            R[3*i:3*i+3,:3]=np.eye(3)
            R[3*i:3*i+3,3:]=[[0,z,-y],[-z,0,x],[y,-x,0]]
        if np.linalg.matrix_rank(R,tol=1e-11)!=6:raise ValueError('interface nodes cannot carry a full force/moment wrench')
        node_set=set(map(int,nodes));facets=[t for t in boundary if set(map(int,t)).issubset(node_set)]
        area=sum(np.linalg.norm(np.cross(points[t[1]]-points[t[0]],points[t[2]]-points[t[0]]))/2 for t in facets)
        cad_area=float(item['area_mm2'])*1e-6 if 'area_mm2' in item else sum(np.linalg.norm(np.cross(t[1]-t[0],t[2]-t[0]))/2 for t in triangles)
        coverage=area/cad_area if cad_area>0 else 0
        if not .95<=coverage<=1.05:raise ValueError(f"{item['id']}: FE/CAD interface area coverage {coverage:.4g}, required 0.95..1.05")
        weights=np.zeros(len(nodes));lookup={int(n):i for i,n in enumerate(nodes)}
        for t in facets:
            a=np.linalg.norm(np.cross(points[t[1]]-points[t[0]],points[t[2]]-points[t[0]]))/6
            for n in t:weights[lookup[int(n)]]+=a
        weights/=sum(weights)
        ports.append(dict(id=item['id'],nodes=nodes,origin=origin,R=R,weights=weights,coverage=coverage))
    return points,tets,boundary,ports

def assemble(points,tets,material):
    E=float(material['young_mpa'])*1e6;nu=float(material['poisson_ratio']);rho=float(material['density_kg_m3'])
    if not (1e3<=E<=1e13 and -.99<nu<.499 and 1<=rho<=1e6):raise ValueError('invalid isotropic material')
    lam=E*nu/((1+nu)*(1-2*nu));mu=E/(2*(1+nu))
    D=np.zeros((6,6));D[:3,:3]=lam;np.fill_diagonal(D[:3,:3],lam+2*mu);D[3:,3:]=np.eye(3)*mu
    rows=[];cols=[];kv=[];mv=[];elements=[];volume=0.
    coefficients=mass_coefficients(1 if tets.shape[1]==4 else 2)
    for tet in tets:
        p=points[tet[:4]];A=np.column_stack([np.ones(4),p]);V=abs(np.linalg.det(p[1:]-p[0]))/6
        grad=np.linalg.inv(A)[1:,:].T
        if len(tet)==4:matrices=[strain_matrix(grad)]
        else:
            matrices=[]
            for k in range(4):
                L=np.full(4,(5-math.sqrt(5))/20);L[k]=(5+3*math.sqrt(5))/20
                g=np.vstack([(4*L-1)[:,None]*grad,[4*(L[i]*grad[j]+L[j]*grad[i]) for i,j in EDGES]])
                matrices.append(strain_matrix(g))
        K=V*sum(B.T@D@B for B in matrices)/len(matrices);M=rho*V*np.kron(coefficients,np.eye(3))
        dofs=(tet[:,None]*3+np.arange(3)).ravel()
        rows.extend(np.repeat(dofs,len(dofs)));cols.extend(np.tile(dofs,len(dofs)));kv.extend(K.ravel());mv.extend(M.ravel())
        elements.append((dofs,matrices));volume+=V
    n=3*len(points)
    K=sparse.coo_matrix((kv,(rows,cols)),shape=(n,n)).tocsr();M=sparse.coo_matrix((mv,(rows,cols)),shape=(n,n)).tocsr()
    adjacency=K.copy();adjacency.data=np.ones_like(adjacency.data)
    # Three displacement directions need not be coupled for special Poisson
    # ratios; connectivity is checked on the tetrahedra's NODE graph instead.
    edges=np.array([(int(a),int(b)) for tet in tets for a in tet for b in tet])
    graph=sparse.coo_matrix((np.ones(len(edges)),(edges[:,0],edges[:,1])),shape=(len(points),len(points)))
    if sparse.csgraph.connected_components(graph,directed=False)[0]!=1:raise ValueError('disconnected FE assembly; define conforming shared nodes/bonded connections before reduction')
    return K,M,elements,D,volume,volume*rho

def reduction(K,M,ports,modes):
    nd=K.shape[0];bd=np.concatenate([(p['nodes'][:,None]*3+np.arange(3)).ravel() for p in ports]);inside=np.setdiff1d(np.arange(nd),bd)
    if len(inside)<2 or isinstance(modes,bool) or not isinstance(modes,int) or not 1<=modes<=min(24,len(inside)-1):raise ValueError('one to 24 interior modes required; refine the volume mesh')
    R=linalg.block_diag(*(p['R'] for p in ports));Ki=K[inside][:,inside].tocsc();Mi=M[inside][:,inside].tocsc()
    factor=spla.splu(Ki)
    if len(inside)<=180:
        values,Phi=linalg.eigh(Ki.toarray(),Mi.toarray(),subset_by_index=[0,modes-1])
    else:
        values,Phi=spla.eigsh(Ki,k=modes,M=Mi,sigma=0,which='LM',tol=1e-10)
        order=np.argsort(values);values=values[order];Phi=Phi[:,order]
    if min(values)<=0 or not np.isfinite(values).all():raise ValueError('nonpositive fixed-interface eigenvalue; check mesh and supports')
    modal_residual=np.linalg.norm(Ki@Phi-(Mi@Phi)*values)/max(np.linalg.norm(Ki@Phi),1e-30)
    if modal_residual>1e-6 or not np.allclose(Phi.T@(Mi@Phi),np.eye(modes),atol=1e-6):raise ValueError('modal residual/normalization failed')
    Psi=factor.solve(-np.asarray(K[inside][:,bd]@R))
    columns=6*(len(ports)-1)+modes;basis=np.zeros((nd,columns))
    basis[np.ix_(inside,np.arange(6*(len(ports)-1)))]=Psi[:,6:]
    basis[np.ix_(bd,np.arange(6*(len(ports)-1)))]=R[:,6:]
    basis[np.ix_(inside,np.arange(6*(len(ports)-1),columns))]=Phi
    Kr=basis.T@(K@basis);Mr=basis.T@(M@basis)
    linalg.cholesky(Mr);linalg.cholesky(Kr)
    return basis,Kr,Mr,values,modal_residual,inside,bd,R

def csr(matrix):
    a=matrix.tocsr();return dict(data=a.data.tolist(),indices=a.indices.tolist(),indptr=a.indptr.tolist(),shape=list(a.shape))

def static_result(job,points,ports,K,M,elements,D,reduced):
    basis,Kr,Mr,values,residual,inside,bd,R=reduced
    force=np.zeros(len(points)*3)
    seen=set()
    for load in job.get('loads',[]):
        if load['interface'] in seen:raise ValueError('duplicate interface load')
        seen.add(load['interface']);port=next((p for p in ports if p['id']==load['interface']),None)
        if port is None:raise ValueError('unknown load interface')
        wrench=finite(load['wrench_world_n_nm'],(6,))
        if np.max(np.abs(wrench))>1e7:raise ValueError('wrench outside limits')
        nodal=port['R']@np.linalg.solve(port['R'].T@port['R'],wrench)
        force[(port['nodes'][:,None]*3+np.arange(3)).ravel()]+=nodal
    br,bc=np.nonzero(R[:,6:])
    Cp=sparse.coo_matrix((np.concatenate([np.ones(len(inside)),R[:,6:][br,bc]]),(np.concatenate([inside,bd[br]]),np.concatenate([np.arange(len(inside)),len(inside)+bc]))),shape=(K.shape[0],len(inside)+R.shape[1]-6)).tocsr()
    exact=Cp@spla.spsolve((Cp.T@K@Cp).tocsc(),Cp.T@force)
    q=np.linalg.solve(Kr,basis.T@force);u=basis@q
    error=float(np.linalg.norm(u-exact)/max(np.linalg.norm(exact),1e-20))
    if error>1e-6:raise ValueError('Craig-Bampton static comparison failed')
    stress=[]
    for dofs,matrices in elements:
        values=[]
        for B in matrices:
            sx,sy,sz,txy,tyz,txz=D@(B@u[dofs])
            values.append(float(math.sqrt(.5*((sx-sy)**2+(sy-sz)**2+(sz-sx)**2)+3*(txy*txy+tyz*tyz+txz*txz))*1e-6))
        stress.append(max(values))
    root=ports[0];reaction=root['R'].T@(K@u-force)[(root['nodes'][:,None]*3+np.arange(3)).ravel()]
    return dict(node_displacements_mm=(u.reshape(-1,3)*1000).tolist(),element_von_mises_mpa=stress,max_displacement_mm=float(max(np.linalg.norm(u.reshape(-1,3),axis=1))*1000),max_von_mises_mpa=max(stress),root_reaction_world_n_nm=reaction.tolist(),strain_energy_j=float(.5*u@(K@u)),full_vs_rom_relative_displacement_error=error)

def flexible_result(job,points,tets,boundary,ports,K,M,basis,values):
    import exudyn as exu
    from exudyn.FEM import FEMinterface,ObjectFFRFreducedOrderInterface
    from exudyn.itemInterface import ObjectGround,MarkerBodyRigid,MarkerSuperElementRigid,GenericJoint,LoadForceVector,LoadTorqueVector,SensorNode,SensorObject
    settings=job.get('motion',{});duration=float(settings.get('duration_s',1));steps=settings.get('steps',1000);samples=settings.get('samples',101)
    amplitude=float(settings.get('angle_deg',0))*math.pi/180;frequency=float(settings.get('frequency_hz',1));ramp=float(settings.get('ramp_s',.2))
    # Only modal coordinates and reference poses are stored, not one full nodal
    # array per sample. Nodes and modes are bounded separately by the FEM job.
    if not (.01<=duration<=5 and isinstance(steps,int) and 20<=steps<=10000 and isinstance(samples,int) and 2<=samples<=201 and steps%(samples-1)==0 and abs(amplitude)<=math.pi and .05<=frequency<=5 and 0<ramp<=duration):raise ValueError('invalid flexible motion limits')
    fem=FEMinterface();origin=ports[0]['origin'];local=points-origin
    fem.nodes={'Position':local};fem.elements=[{'Tet4' if tets.shape[1]==4 else 'Tet10':tets}];fem.surface=[{'Trigs':boundary}]
    fem.massMatrix=M;fem.stiffnessMatrix=K;fem.modeBasis={'matrix':basis,'type':'HCBmodes'};fem.eigenValues=values
    cms=ObjectFFRFreducedOrderInterface(fem)
    sc=exu.SystemContainer();mbs=sc.AddSystem();ground=mbs.AddObject(ObjectGround(referencePosition=origin.tolist()))
    obj=cms.AddObjectFFRFreducedOrder(mbs,positionRef=origin.tolist(),gravity=[0,0,-9.81],stiffnessProportionalDamping=2*.01/math.sqrt(min(values)))
    body=obj['oFFRFreducedOrder'];root=ports[0]
    def marker(port):
        offset=port['origin']-origin-np.average(local[port['nodes']],axis=0,weights=port['weights'])
        return mbs.AddMarker(MarkerSuperElementRigid(bodyNumber=body,meshNodeNumbers=port['nodes'].tolist(),weightingFactors=port['weights'].tolist(),offset=offset.tolist()))
    mg=mbs.AddMarker(MarkerBodyRigid(bodyNumber=ground));mr=marker(root)
    motion, velocity = joint_motion(dict(amplitude_si=amplitude, ramp_s=ramp, frequency_hz=frequency, profile='sine'), 'revolute', duration)
    joint=mbs.AddObject(GenericJoint(markerNumbers=[mg,mr],constrainedAxes=[1]*6,offsetUserFunction=motion,offsetUserFunction_t=velocity))
    for load in job.get('loads',[]):
        port=next(p for p in ports if p['id']==load['interface']);m=marker(port);w=finite(load['wrench_world_n_nm'],(6,))
        mbs.AddLoad(LoadForceVector(markerNumber=m,loadVector=w[:3].tolist(),bodyFixed=False))
        mbs.AddLoad(LoadTorqueVector(markerNumber=m,loadVector=w[3:].tolist(),bodyFixed=False))
    modal=mbs.AddSensor(SensorNode(nodeNumber=obj['nGenericODE2'],outputVariableType=exu.OutputVariableType.Coordinates,storeInternal=True,writeToFile=False))
    pos=mbs.AddSensor(SensorNode(nodeNumber=obj['nRigidBody'],outputVariableType=exu.OutputVariableType.Position,storeInternal=True,writeToFile=False))
    rot=mbs.AddSensor(SensorNode(nodeNumber=obj['nRigidBody'],outputVariableType=exu.OutputVariableType.RotationMatrix,storeInternal=True,writeToFile=False))
    sensor=mbs.AddSensor(SensorObject(objectNumber=joint,outputVariableType=exu.OutputVariableType.ForceLocal,storeInternal=True,writeToFile=False))
    mbs.Assemble();options=exu.SimulationSettings();options.timeIntegration.endTime=duration;options.timeIntegration.numberOfSteps=steps
    options.timeIntegration.verboseMode=0;options.timeIntegration.generalizedAlpha.spectralRadius=.8
    options.solutionSettings.writeSolutionToFile=False;options.solutionSettings.sensorsWritePeriod=duration/(samples-1)
    mbs.SolveDynamic(options,solverType=exu.DynamicSolverType.GeneralizedAlpha)
    q=mbs.GetSensorStoredData(modal);p=mbs.GetSensorStoredData(pos);r=mbs.GetSensorStoredData(rot);f=mbs.GetSensorStoredData(sensor)
    if q.shape!=(samples,basis.shape[1]+1) or not all(np.isfinite(v).all() and v.shape[0]==samples for v in [q,p,r,f]):raise ValueError('invalid flexible result sensors')
    return dict(backend='Exudyn FFRFreducedOrder',backend_version=exu.__version__,times_s=q[:,0].tolist(),modal_coordinates=q[:,1:].tolist(),reference_poses=np.column_stack([p[:,1:],r[:,1:]]).tolist(),joint_forces_local_n=f[:,1:].tolist(),max_elastic_displacement_mm=float(max(np.linalg.norm((basis@row).reshape(-1,3),axis=1).max() for row in q[:,1:])*1000),coupling='FFRF inertial coupling and elastic joint reaction; first surface attached to prescribed moving reference',force_initial_sample_valid=False)

def run(job):
    if job.get('schema')!='solidcraft.fem-job/1' or job.get('action') not in ['reduce','flexible']:raise ValueError('unsupported FEM job')
    points,tets,boundary,ports=prepare_mesh(job)
    K,M,elements,D,volume,mass=assemble(points,tets,job['material'])
    reduced=reduction(K,M,ports,job.get('modes',6));basis,Kr,Mr,values,residual,*_=reduced
    result=dict(schema='solidcraft.fem-rom/1',status='success',source=job['source'],nodes_mm=(points*1000).tolist(),tetrahedra=tets.tolist(),element_type='TET4' if tets.shape[1]==4 else 'TET10',surface_triangles=boundary.tolist(),material=job['material'],mass_kg=mass,volume_mm3=volume*1e9,
        interfaces=[dict(id=p['id'],nodes=p['nodes'].tolist(),origin_mm=(p['origin']*1000).tolist(),weights=p['weights'].tolist(),area_coverage=p['coverage'],coupling='RBE2',reference=i==0) for i,p in enumerate(ports)],
        rom=dict(method='Hurty-Craig-Bampton / RBE2',reference_interface=ports[0]['id'],basis=basis.tolist(),stiffness_reduced=Kr.tolist(),mass_reduced=Mr.tolist(),fixed_interface_frequencies_hz=(np.sqrt(values)/(2*math.pi)).tolist(),modal_residual=float(residual),coordinates='interface translations [m], rotations [rad] except first reference, followed by mass-normalized fixed-interface modes',full_dofs=K.shape[0],reduced_dofs=basis.shape[1]),
        static=static_result(job,points,ports,K,M,elements,D,reduced),
        limitations=['Linear isotropic elasticity: convergence and element-quality studies required; TET4 bending may be too stiff. TET10 has quadratic displacement and exact consistent mass; linear geometry.','RBE2 stiffens each selected surface; it is not pressure/distributing (RBE3) coupling.','Disconnected assembly meshes are refused; no automatic contact, bolt or adhesive model.','Small elastic deformation about a floating frame; no yielding, buckling or contact.','Flexible runs start with zero elastic displacement/velocity; gravity and loads excite startup vibration. Static prestress equilibrium is not initialized.'])
    if job['action']=='flexible':result['flexible']=flexible_result(job,points,tets,boundary,ports,K,M,basis,values)
    import scipy
    result['provenance']=dict(motion_profiles_sha256=SOURCE_SHA256,worker_sha256=sha256(Path(__file__).read_bytes()).hexdigest(),job_sha256=sha256(json.dumps(job,sort_keys=True,separators=(',',':'),allow_nan=False).encode()).hexdigest(),numpy_version=np.__version__,scipy_version=scipy.__version__)
    return result

def main():
    try:
        raw=sys.stdin.read(6*1024*1024+1)
        if len(raw)>6*1024*1024:raise ValueError('FEM input exceeds 6 MiB')
        job=json.loads(raw,parse_constant=lambda v:(_ for _ in ()).throw(ValueError('nonfinite JSON')))
        with redirect_stdout(sys.stderr):result=run(job)
        text=json.dumps(result,separators=(',',':'),allow_nan=False)
        if len(text)>12*1024*1024:raise ValueError('FEM output exceeds 12 MiB')
        sys.stdout.write(text);return 0
    except Exception as exc:
        sys.stdout.write(json.dumps(dict(status='failed',error=str(exc))));return 1

if __name__=='__main__':raise SystemExit(main())
