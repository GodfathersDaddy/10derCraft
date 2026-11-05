"""Real FFRF chain acceptance; driven/free joints and deformable interfaces."""
from verify_fem_rom import job,ROOT
import sys,json,numpy as np
sys.path.insert(0,str(ROOT/'simulation'))
import fem_worker as fe, flexible_assembly_worker as worker
def envelope(two=False,driven=True):
    cfg=job([4,1,1]);r=fe.run(cfg);components={'arm1':dict(job=cfg,result=r)}
    bodies=[dict(id='arm1',mass_kg=78.5,com_world_m=[.5,.05,.05],inertia_com_world_kg_m2=np.diag([78.5*.02/12,78.5*1.01/12,78.5*1.01/12]).tolist(),flexible_component='arm1')]
    def joint(i,a,b,x):return dict(id=i,name='J'+str(i),kind='revolute',a=a,b=b,pivot_world_m=[x,.05,.05],axis_world=[0,0,1],rotation_world=np.eye(3).tolist(),native_values_at_start=[0])
    joints=[joint(1,'ground','arm1',0)]
    if two:
        cfg2=job([4,1,1]);cfg2['mesh']['shape']['corner_mm'][0]=1000
        for p in cfg2['interfaces']:
            p['origin_mm'][0]+=1000
            for t in p['triangles_mm']:
                for point in t:point[0]+=1000
        components['arm2']=dict(job=cfg2,result=fe.run(cfg2));bodies.append(dict(bodies[0],id='arm2',com_world_m=[1.5,.05,.05],flexible_component='arm2'));joints.append(joint(2,'arm1','arm2',1))
    original=dict(schema='solidcraft.assembly-job/1',backend='exudyn_ffrf_open_chain',source={'test':'flex-chain'},settings=dict(duration_s=.05,steps=200,samples=51,gravity_m_s2=[0,0,-9.81]),model=dict(ground='ground',bodies=bodies,joints=joints,loads=[dict(port='tip',block=bodies[-1]['id'],position_world_m=[2 if two else 1,.05,.05],force_initial_world_n=[0,0,-100],moment_initial_world_nm=[0,0,0])],drivers=[dict(joint=1,amplitude_si=.1,ramp_s=.05,profile='quintic',frequency_hz=1)] if driven else []))
    return dict(schema='solidcraft.flex-assembly-job/1',job=original,components=components)
if __name__=='__main__':
    reports=[]
    for two,driven in [(False,True),(False,False),(True,True)]:
        e=envelope(two,driven);r=worker.run(e)
        assert len(r['flexible_states'])==(2 if two else 1)
        assert max(abs(x) for state in r['flexible_states'].values() for q in state['modal_coordinates'] for x in q)>0
        assert r['summary']['max_position_constraint_m']<1e-5
        reports.append(dict(two=two,driven=driven,summary=r['summary']))
        print(reports[-1],flush=True)
    folder=ROOT/'output/fem-rom';folder.mkdir(exist_ok=True,parents=True)
    (folder/'flexible-chain-result.json').write_text(json.dumps(r),encoding='utf-8')
    (folder/'flexible-chain-verification.json').write_text(json.dumps(dict(passed=True,cases=reports),indent=2),encoding='utf-8')
