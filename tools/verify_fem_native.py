"""Command-level native CAD/FEM/ROM/flexible MKS round trip."""
from cad_test_client import Desktop,ROOT
import time,json
with Desktop('fem-rom-native',port=7969) as d:
    d.call('ui.resize',width=1600,height=1000);d.cmd('file.new',name='Flexarm - CAD FEM ROM MKS')
    arm=d.cmd('mks.library.body',name='Flexarm',mass_kg=78.5,size_mm=[1000,100,100],position_mm=[500,50,50])
    base=d.cmd('mks.library.body',name='Ground',mass_kg=10,size_mm=[100,100,100],position_mm=[-100,50,50],grounded=True)
    joint=d.cmd('mks.library.joint',name='Revolute',kind='revolute',a=base['occurrence'],b=arm['occurrence'],point_mm=[0,50,50],axis=[0,0,1])['joint']
    faces=d.cmd('model.faces',body=arm['body'])['faces']
    for x,name in [(0,'IF_Root'),(1000,'IF_Tip')]:
        face=next(f for f in faces if abs(f['centroid'][0]-x)<1e-6)
        d.cmd('select.set',items=[dict(type='face',body=arm['body'],index=face['index'],point=face['point'])]);d.cmd('mks.interface.capture',name=name,occurrence=arm['occurrence'],transfer='frame_wrench',surface=True)
    scopes=d.cmd('fem.model.read')['scopes'];scope=next(s for s in scopes if 'IF_Root' in s['interfaces'])
    began=time.monotonic()
    d.cmd('fem.model.create',id='Flexarm_ROM',block=scope['id'],interfaces=['IF_Root','IF_Tip'],material=dict(young_mpa=210000,poisson_ratio=.3,density_kg_m3=7850),divisions=[4,2,2],modes=6)
    d.cmd('fem.rom.reduce',id='Flexarm_ROM')
    d.cmd('fem.mks.bind',id='Flexarm_ROM',enabled=True)
    result=d.cmd('mks.job.run',duration_s=.1,steps=400,samples=101,drivers=[dict(joint=joint,displacement='20 deg',ramp_s=.1,profile='quintic')])
    assert len(result['flexible_states'])==1
    assert result['summary']['max_position_constraint_m']<1e-5
    d.click('project:mks');d.click('mks:graph:play');time.sleep(.3);d.click('mks:graph:pause')
    if not d.call('ui.handles',prefix='mks:scope:plot'):d.click('mks:scope:toggle')
    time.sleep(.5);d.screenshot('01-Flexible-MKS-Kraefte.png')
    d.cmd('mks.result.snapshot',sample=100,path=str(d.out/'02-Flexible-MKS-Netz.png'),width=1000,height=650,overwrite=True)
    d.cmd('mks.result.video',path=str(d.out/'Flexarm-Flexible-MKS.avi'),width=640,height=360,fps=30,overwrite=True)
    project=d.out/'Flexarm-FEM-ROM-MKS.solidcraft';d.cmd('file.save',path=str(project))
    d.cmd('fem.rom.export',id='Flexarm_ROM',path=str(d.out/'Flexarm-ROM.json'),overwrite=True)
    d.cmd('doc.open',path=str(project));d.settled()
    assert d.cmd('mks.job.state')['result_current']
    assert d.cmd('fem.model.read')['models'][0]['current']
    d.cmd('parameters.change',name=f"mbs_{arm['occurrence']}_L",expression='1100 mm')
    assert not d.cmd('mks.job.state')['result_current'] and not d.cmd('fem.model.read')['models'][0]['current']
    report=dict(passed=True,seconds=time.monotonic()-began,summary=result['summary'],project=str(project))
    (d.out/'verification.json').write_text(json.dumps(report,indent=2),encoding='utf-8');print(json.dumps(report),flush=True)
