"""Mount the catalogue motor on the robot using native CAD and standard fasteners."""
from cad_test_client import Desktop, ROOT
import json

source=ROOT/'output/robot-standard-parts/Roboter-Parametrisch-Normteile.solidcraft'
with Desktop('robot-gripper-motor',port=7989) as d:
    d.call('ui.resize',width=1600,height=1000)
    d.cmd('doc.open',path=str(source));d.settled()
    d.click('project:cad')
    listing=d.cmd('component.list')['components']
    tool=next(c for c in listing if c['name']=='tool')['occurrences'][0]['id']
    def rigid(name,a,b,p):
        d.cmd('joint.create',name=name,type='rigid',a=dict(occurrence=a,point=p,z=[0,0,1]),
              b=dict(occurrence=b,point=p,z=[0,0,1]),flip=True)
    d.cmd('component.activate',component='tool')
    # Clearance for the motor shaft and for the heads of the upward-facing M3 bolts.
    d.cmd('solid.cylinder',name='Greifer_Motorwellenfreiraum',diameter=5.5,height=23,base=[1053,0,948],operation='cut',targets=['Greifer-Gehaeuse'])
    holes=[(1053+sx*15.5,sy*15.5) for sx in [-1,1] for sy in [-1,1]]
    for i,(x,y) in enumerate(holes):
        d.cmd('solid.cylinder',name=f'Greifer_M3_Kopftasche_{i}',diameter=6.4,height=4.5,base=[x,y,965.5],operation='cut',targets=['Greifer-Gehaeuse'])
    fixes=[(x,y) for x in [1012.5,1085] for y in [-43,43]]
    for i,(x,y) in enumerate(fixes):
        d.cmd('solid.cylinder',name=f'Greifer_M4_Kernloch_{i}',diameter=3.3,height=11,base=[x,y,959],operation='cut',targets=['Greifer-Gehaeuse'])
    d.cmd('component.activate',component=0)
    plate=d.cmd('component.create',name='Greifer_Motoradapter',activate=True)
    d.cmd('parameters.add',name='t_greifer_motorplatte',expression='4 mm')
    d.cmd('solid.box',name='Motoradapter_Rohteil',body_name='Motoradapter',corner=[1000.5,-56,969],length=105,width=112,height='t_greifer_motorplatte')
    d.cmd('solid.fillet',name='Motoradapter_Ecken',radius=3,edges=[[x,y,971] for x in [1000.5,1105.5] for y in [-56,56]])
    d.cmd('solid.cylinder',name='Motoradapter_Zentrierfreiraum',diameter=22.4,height=6,base=[1053,0,968],operation='cut',targets=['Motoradapter'])
    for i,(x,y) in enumerate(holes):
        d.cmd('solid.cylinder',name=f'Motoradapter_M3_Durchgang_{i}',diameter=3.4,height=6,base=[x,y,968],operation='cut',targets=['Motoradapter'])
    for i,(x,y) in enumerate(fixes):
        d.cmd('solid.cylinder',name=f'Motoradapter_M4_Durchgang_{i}',diameter=4.5,height=6,base=[x,y,968],operation='cut',targets=['Motoradapter'])
    # Keep existing guide rails and cover screw heads accessible.
    for i,y in enumerate([-51.5,20.5]):
        d.cmd('solid.box',name=f'Motoradapter_Fuehrungsfreiraum_{i}',corner=[1093.5,y,968.5],length=13,width=31,height=2.7,operation='cut',targets=['Motoradapter'])
    for x in [1028,1075]:
        for y in [-36,36]:
            d.cmd('solid.cylinder',name=f'Motoradapter_Deckelschraubenfreiraum_{x}_{y}',diameter=9,height=6,base=[x,y,968],operation='cut',targets=['Motoradapter'])
    d.cmd('material.assign',bodies=['Motoradapter'],material='Aluminum')
    d.cmd('component.activate',component=0)
    rigid('Motoradapter_am_Greifer',tool,plate['occurrence'],[1053,0,969])
    motor=d.cmd('parts.insert',family='stepper_motor',size='17HS19-2004S1',at=[1064.2,0,973],name='Greifer_17HS19_2004S1')
    print('Motor mounted',flush=True)
    hardware=[]
    for i,(x,y) in enumerate(holes):
        bolt=d.cmd('parts.insert',family='socket_head_cap_screw',size='M3',length=8,
                   point=[x,y,969],direction=[0,0,-1],name=f'Greifer_Motor_M3x8_{i}')
        rigid(f'Greifer_Motor_M3_Sitz_{i}',plate['occurrence'],bolt['occurrence'],[x,y,969]);hardware.append(bolt['component'])
    row=next(r for r in d.cmd('parts.library',family='washer')['sizes'] if r['size']=='M4')
    for i,(x,y) in enumerate(fixes):
        washer=d.cmd('parts.insert',family='washer',size='M4',point=[x,y,973],name=f'Greifer_Adapter_Scheibe_M4_{i}')
        rigid(f'Greifer_Adapter_Scheibe_Sitz_{i}',plate['occurrence'],washer['occurrence'],[x,y,973]);hardware.append(washer['component'])
        seat=[x,y,973+row['thickness']]
        bolt=d.cmd('parts.insert',family='socket_head_cap_screw',size='M4',length=12,point=seat,name=f'Greifer_Adapter_M4x12_{i}')
        rigid(f'Greifer_Adapter_M4_Sitz_{i}',washer['occurrence'],bolt['occurrence'],seat);hardware.append(bolt['component'])
    listing=d.cmd('component.list')['components']
    names=[name for c in listing if c['id'] in hardware for name in c['bodies']]
    motor_names=[name for c in listing if c['id']==motor['component'] for name in c['bodies']]
    d.cmd('material.assign',bodies=names,material='Steel')
    # Test non-threaded fitting surfaces. Thread engagement intentionally overlaps minor bores.
    old_names=['Greifer-Gehaeuse','Greifer-Fuehrung-L','Greifer-Fuehrung-R']
    check=d.cmd('inspect.clearance',bodies=['Motoradapter']+motor_names,against=old_names,clearance=.05)
    (d.out/'fit-clearance.json').write_text(json.dumps(check,indent=2),encoding='utf-8')
    assert not check['overlaps'] and not check['unchecked'],check
    check_plate=d.cmd('inspect.clearance',bodies=names,against=['Motoradapter'],clearance=.05)
    assert not check_plate['overlaps'] and not check_plate['unchecked'],check_plate
    joints=d.cmd('joint.list');assert not joints['conflicts'] and not joints['floating']
    project=d.out/'Roboter-Parameter-Motor-Normteile.solidcraft'
    d.cmd('file.save',path=str(project))
    settings=d.cmd('mks.job.state')['data']['job']['settings']
    result=d.cmd('mks.job.run',**settings)
    assert result['summary']['max_position_constraint_m']<1e-5
    d.cmd('file.save',path=str(project));d.cmd('view.isometric');d.cmd('view.fit');d.screenshot('01-Roboter-mit-Greifermotor.png')
    report=dict(passed=True,motor=motor,adapter=plate,hardware_components=len(hardware),summary=result['summary'],
                scope='Mounted gripper motor and fasteners, motor carried as rigid mass. Finger transmission and detailed-arm solid ROM remain open.')
    (d.out/'verification.json').write_text(json.dumps(report,indent=2),encoding='utf-8');print(json.dumps(report),flush=True)
