"""Replace the robot foundation's decorative primitives with native standard parts."""
from cad_test_client import Desktop, ROOT, SOURCE
import json
with Desktop('robot-standard-parts',port=7987) as d:
    d.call('ui.resize',width=1600,height=1000)
    d.cmd('doc.open',path=str(SOURCE/'examples/projects/Roboter-Parametrisch-Grosse-Bewegung.solidcraft'));d.settled()
    for i in range(4):
        for prefix in ['Fundament-Ankerschraube-','Fundament-Unterlegscheibe-']:
            d.cmd('timeline.delete',feature=prefix+str(i))
    for i,(x,y) in enumerate([(-175,-145),(-175,145),(175,-145),(175,145)]):
        d.cmd('component.activate',component='base')
        d.cmd('solid.cylinder',name=f'Fundament_M16_Durchgang_{i}',base=[x,y,-1],diameter=17.5,height=38,
              operation='cut',targets=['Fundament'])
        d.cmd('component.activate',component=0)
        washer=d.cmd('parts.insert',family='washer',size='M16',at=[x+8.75,y,36],name=f'ISO7089_M16_{i}')
        # Washer thickness from the same native library table used by the UI.
        row=next(r for r in d.cmd('parts.library',family='washer')['sizes'] if r['size']=='M16')
        seat=[x,y,36+row['thickness']]
        bolt=d.cmd('parts.insert',family='hex_bolt',size='M16',length=80,
              point=seat,name=f'ISO4017_M16x80_{i}')
        d.cmd('joint.create',name=f'M16_Schraubensitz_{i}',type='rigid',
              a=dict(occurrence=washer['occurrence'],point=seat,z=[0,0,1]),
              b=dict(occurrence=bolt['occurrence'],point=seat,z=[0,0,1]),flip=True)
        print('M16 mount',i,flush=True)
    joints=d.cmd('joint.list');assert not joints['conflicts'] and not joints['floating']
    state=d.cmd('mks.job.state')
    settings=state['data']['job']['settings']
    # The saved independent graph sources retain their own ramp/frequency settings.
    # Keep their full eight-second study rather than truncating a managed source.
    settings.update(duration_s=8,steps=4000,samples=501)
    d.cmd('file.save',path=str(d.out/'Roboter-Parametrisch-Normteile.solidcraft'))
    result=d.cmd('mks.job.run',**settings)
    assert result['summary']['max_position_constraint_m']<1e-5
    d.cmd('view.isometric');d.cmd('view.fit');d.screenshot('01-Roboter-Normteile.png')
    d.cmd('file.save',path=str(d.out/'Roboter-Parametrisch-Normteile.solidcraft'))
    health=d.cmd('document.inspect',measure=False)
    errors=[f for f in health['timeline'] if f.get('error')]
    assert not errors,errors
    report=dict(passed=True,native_fastener_sets=4,short_rigid_mks=result['summary'],
                scope='Original six-axis parameterised robot, foundation washers/bolts replaced, native drilled clearances; solid ROM for detailed assembly pending')
    (d.out/'verification.json').write_text(json.dumps(report,indent=2),encoding='utf-8')
    print(json.dumps(report),flush=True)
