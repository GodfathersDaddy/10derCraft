"""Build a native two-ROM crane using the same public commands as the library UI.

This demonstrates transport with an already attached pendulating payload,
not gripping/contact. No private CAD or simulation geometry is substituted.
"""
from cad_test_client import Desktop, ROOT
import json, time

with Desktop('crane-flexible', port=7974) as d:
    d.call('ui.resize', width=1600, height=1000)
    d.cmd('file.new', name='Kran - zwei flexible Arme und Pendellast')
    ground = d.cmd('mks.library.body', name='Fundament', mass_kg=100,
                   size_mm=[200,200,200], position_mm=[-150,50,50], grounded=True)
    arms = [d.cmd('mks.library.body', name=f'Ausleger_{i+1}', mass_kg=78.5,
                  size_mm=[1000,100,100], position_mm=[500+1000*i,50,50]) for i in range(2)]
    payload = d.cmd('mks.library.body', name='Last_20kg', mass_kg=20,
                    size_mm=[200,200,200], position_mm=[2000,50,-150])
    joints = []
    previous = ground
    configs = []
    for i, arm in enumerate(arms):
        joints.append(d.cmd('mks.library.joint', name=f'Antrieb_{i+1}', kind='revolute',
                           a=previous['occurrence'], b=arm['occurrence'],
                           point_mm=[i*1000,50,50], axis=[0,0,1])['joint'])
        previous = arm
    d.cmd('mks.library.joint', name='Lastpendel', kind='revolute', a=arms[-1]['occurrence'],
          b=payload['occurrence'], point_mm=[2000,50,50], axis=[0,1,0])
    for i, arm in enumerate(arms):
        faces = d.cmd('model.faces', body=arm['body'])['faces']
        names = [f'Arm{i+1}_Root', f'Arm{i+1}_Tip']
        for x, name in zip([1000*i,1000*(i+1)], names):
            face = next(f for f in faces if abs(f['centroid'][0]-x)<1e-6)
            d.cmd('select.set', items=[dict(type='face',body=arm['body'],index=face['index'],point=face['point'])])
            d.cmd('mks.interface.capture', name=name, occurrence=arm['occurrence'], transfer='frame_wrench',surface=True)
        scope = next(s for s in d.cmd('fem.model.read')['scopes'] if names[0] in s['interfaces'])
        model_id = f'ROM_Arm{i+1}'
        configs.append(dict(id=model_id,block=scope['id'],interfaces=names,
              material=dict(young_mpa=210000,poisson_ratio=.3,density_kg_m3=7850),divisions=[4,1,1],modes=6))
    # Capture every CAD interface before solving: later CAD edits invalidate ROMs.
    for i in range(2):
        model_id = f'ROM_Arm{i+1}'
        d.cmd('fem.model.create',**configs[i])
        d.cmd('fem.rom.reduce',id=model_id)
        d.cmd('fem.mks.bind',id=model_id,enabled=True)
        print(f'{model_id} ready',flush=True)
    result=d.cmd('mks.job.run',duration_s=2,steps=4000,samples=1001,
                 drivers=[dict(joint=j,displacement=a,ramp_s=.5,profile='sine',frequency_hz=f)
                          for j,a,f in zip(joints,['45 deg','-30 deg'],[.5,.75])])
    assert len(result['flexible_states'])==2
    assert result['summary']['max_position_constraint_m']<1e-5
    d.click('project:mks');d.click('mks:graph:run');time.sleep(1)
    if not d.call('ui.handles',prefix='mks:scope:plot'):d.click('mks:scope:toggle')
    d.screenshot('Kran-Flexible-MKS.png')
    d.cmd('mks.result.video',path=str(d.out/'Kran-Flexible-MKS.avi'),width=960,height=540,fps=30,overwrite=True)
    project=d.out/'Kran-Zwei-Flexible-Arme.solidcraft'
    d.cmd('file.save',path=str(project))
    report=dict(passed=True,project=str(project),summary=result['summary'],
                scope='two flexible native beams, two prescribed drives, 20kg pendulating payload; no grip/contact')
    (d.out/'verification.json').write_text(json.dumps(report,indent=2),encoding='utf-8')
    print(json.dumps(report),flush=True)
