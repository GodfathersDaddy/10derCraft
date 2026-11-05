"""Short independent solid-FEM/CB checks, not a full engineering qualification."""
from pathlib import Path
import json,sys,time
import numpy as np
from scipy import sparse,linalg
from scipy.sparse import linalg as spla
ROOT=Path(__file__).resolve().parents[1]
sys.path.insert(0,str(ROOT/'simulation'))
import fem_worker as fem
def job(n):
    faces=[[[[0,0,0],[0,100,0],[0,100,100]],[[0,0,0],[0,100,100],[0,0,100]]],[[[1000,0,0],[1000,100,100],[1000,100,0]],[[1000,0,0],[1000,0,100],[1000,100,100]]]]
    return dict(schema='solidcraft.fem-job/1',action='reduce',source={'reference':'analytic-box'},mesh=dict(shape=dict(kind='box',corner_mm=[0,0,0],size_mm=[1000,100,100]),divisions=n),material=dict(young_mpa=210000,poisson_ratio=.3,density_kg_m3=7850),interfaces=[dict(id=name,origin_mm=[x,50,50],triangles_mm=face) for name,x,face in zip(['root','tip'],[0,1000],faces)],loads=[dict(interface='tip',wrench_world_n_nm=[0,0,-100,0,0,0])],modes=6)
if __name__=='__main__':
    out=ROOT/'output/fem-rom';out.mkdir(parents=True,exist_ok=True)
    reports=[]
    for n in [[2,1,1],[4,1,1],[8,2,2]]:
        began=time.monotonic();r=fem.run(job(n))
        assert abs(r['mass_kg']-78.5)<1e-9
        assert r['static']['full_vs_rom_relative_displacement_error']<1e-6
        assert np.allclose(r['static']['root_reaction_world_n_nm'][:3],[0,0,100],atol=1e-6)
        entry=dict(divisions=n,nodes=len(r['nodes_mm']),displacement_mm=r['static']['max_displacement_mm'],static_rom_relative_error=r['static']['full_vs_rom_relative_displacement_error'],fixed_frequencies_hz=r['rom']['fixed_interface_frequencies_hz'],seconds=time.monotonic()-began)
        reports.append(entry);print(entry,flush=True)
    assert all(a['displacement_mm']<b['displacement_mm'] for a,b in zip(reports,reports[1:]))
    assert abs(reports[-1]['displacement_mm']/0.01904761904761905-1)<0.03
    j=job([2,1,1]);points,tets,boundary,ports=fem.prepare_mesh(j)
    K,M,_,_,volume,mass=fem.assemble(points,tets,j['material'])
    # A rigid translation has zero strain energy; consistent mass recovers the
    # physical mass in each Cartesian direction independently of the reduction.
    for axis in range(3):
        rigid=np.zeros(3*len(points));rigid[axis::3]=1
        assert abs(rigid@(M@rigid)-mass)<1e-9
        assert np.linalg.norm(K@rigid)<np.linalg.norm(K.data)*1e-12
    # Compare actual cantilever modes, not only the retained fixed-interface
    # eigenproblem. Both models use identical RBE2 surface assumptions.
    j=job([8,2,2]);points,tets,boundary,ports=fem.prepare_mesh(j);K,M,*_=fem.assemble(points,tets,j['material'])
    basis,Kr,Mr,_,_,inside,bd,R=fem.reduction(K,M,ports,6)
    br,bc=np.nonzero(R[:,6:]);C=sparse.coo_matrix((np.concatenate([np.ones(len(inside)),R[:,6:][br,bc]]),(np.concatenate([inside,bd[br]]),np.concatenate([np.arange(len(inside)),len(inside)+bc]))),shape=(K.shape[0],len(inside)+6)).tocsr()
    full=np.sort(spla.eigsh(C.T@K@C,k=3,M=C.T@M@C,sigma=0,which='LM',return_eigenvectors=False))
    reduced=linalg.eigh(Kr,Mr,subset_by_index=[0,2],eigvals_only=True)
    dynamic_error=(np.sqrt(reduced/full)-1)
    assert max(abs(dynamic_error))<.03
    print('Cantilever frequency relative errors',dynamic_error,flush=True)
    cyl=job([4,2,2]);cyl['mesh']['shape']=dict(kind='cylinder',base_mm=[0,0,0],axis=[1,0,0],radius_mm=50,length_mm=1000)
    for p in cyl['interfaces']:
        p['origin_mm'][1:]=[0,0];p['disk']=dict(center_mm=p['origin_mm'],normal=[1,0,0],radius_mm=50);p['area_mm2']=np.pi*50**2
    cr=fem.run(cyl)
    assert abs(cr['mass_kg']/(np.pi*.05**2*1*7850)-1)<.02
    assert cr['static']['full_vs_rom_relative_displacement_error']<1e-6
    print('Cylinder',len(cr['nodes_mm']),'nodes; mass',cr['mass_kg'],flush=True)
    for bad in ['fractional','overlap','disconnected']:
        j=job([2,1,1])
        if bad=='fractional':j['mesh']['divisions']=[2.5,1,1]
        elif bad=='overlap':j['interfaces'][1]['triangles_mm']=j['interfaces'][0]['triangles_mm']
        else:
            j['mesh']=dict(nodes_mm=[[0,0,0],[1,0,0],[0,1,0],[0,0,1],[10,0,0],[11,0,0],[10,1,0],[10,0,1]],tetrahedra=[[0,1,2,3],[4,5,6,7]])
            # Connectivity is an independent FEM condition, not an interface error.
            try:fem.assemble(np.array(j['mesh']['nodes_mm'])*.001,np.array(j['mesh']['tetrahedra']),j['material'])
            except ValueError as e:assert 'disconnected' in str(e)
            else:raise AssertionError('disconnected mesh accepted')
            continue
        try:fem.run(j)
        except ValueError:pass
        else:raise AssertionError('invalid FEM accepted: '+bad)
    j=job([6,2,2]);j['action']='flexible';j['motion']=dict(duration_s=.04,steps=200,samples=41,angle_deg=10,frequency_hz=2,ramp_s=.02)
    r=fem.run(j)
    assert len(r['flexible']['times_s'])==41 and r['flexible']['max_elastic_displacement_mm']>0
    (out/'reference-result.json').write_text(json.dumps(r),encoding='utf-8')
    (out/'verification.json').write_text(json.dumps(dict(passed=True,convergence=reports,full_fe_cantilever_frequencies_hz=(np.sqrt(full)/(2*np.pi)).tolist(),rom_frequency_relative_errors=dynamic_error.tolist(),analytic_euler_bernoulli_tip_mm=100*1000**3/(3*210000*(100*100**3/12)),flexible=r['flexible']['max_elastic_displacement_mm']),indent=2),encoding='utf-8')
    print('True FFRF flexible simulation passed',r['flexible']['max_elastic_displacement_mm'],flush=True)
