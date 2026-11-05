"""Focused acceptance through native library and MKS UI entry points."""
from cad_test_client import Desktop, ROOT, SOURCE
import json, math, sys, time

sys.path.insert(0, str(SOURCE/'tools/10dercraft'))
from ui_helpers import wait_handle, handle

with Desktop('robot-rom-library',port=7986) as d:
    d.call('ui.resize',width=1600,height=1000)
    library=d.cmd('parts.library')
    assert 'stepper_motor' in [f['family'] for f in library['families']]
    motor=d.cmd('parts.insert',family='stepper_motor',size='17HS19-2004S1',name='Motor_17HS19')
    measure=d.cmd('inspect.measure')['bodies']
    assert len(measure)==1 and measure[0]['closed']
    assert abs(measure[0]['mass_g']-390)<1e-5
    bbox=measure[0]['bbox']
    assert all(abs(a-b)<1e-6 for a,b in zip(bbox['min'],[-21,-21,-48]))
    assert all(abs(a-b)<1e-6 for a,b in zip(bbox['max'],[21,21,24]))
    radius=2.5;offset=2
    segment=radius**2*math.acos(offset/radius)-offset*math.sqrt(radius**2-offset**2)
    expected=42*42*48-4*math.pi*1.25**2*4.5+math.pi*11**2*2+math.pi*2.5**2*22-segment*15
    assert abs(measure[0]['volume_mm3']/expected-1)<1e-4
    # Same existing standard library used by people through Insert Part.
    for family,size,x in [('socket_head_cap_screw','M3',65),('washer','M3',85),('hex_nut','M3',100),('bearing','608',125)]:
        args=dict(family=family,size=size,point=[x,0,0])
        if family=='socket_head_cap_screw':args['length']=8
        d.cmd('parts.insert',**args)
    d.cmd('view.fit');d.screenshot('01-Motor-und-Normteile.png')
    d.cmd('file.save',path=str(d.out/'Motor-und-Normteile.solidcraft'))
    d.cmd('doc.open',path=str(SOURCE/'examples/projects/Kran-Zwei-Flexible-Arme.solidcraft'));d.settled()
    models=d.cmd('fem.model.read')['models']
    assert len(models)==2 and all(m['enabled'] and m['current'] for m in models)
    d.click('project:mks');time.sleep(.4)
    entry=handle(d,'mks:library:rom');target=handle(d,'mks:body:'+models[1]['block'])
    d.call('ui.drag',x0=entry['x'],y0=entry['y'],x1=target['x'],y1=target['y'],button='left',steps=15)
    wait_handle(d,'mks:rom:results:'+models[1]['id'])
    wait_handle(d,'mks:library:add:rom')
    d.click('mks:library:add:rom')
    model=models[0]
    wait_handle(d,'mks:rom:results:'+model['id'])
    d.screenshot('02-ROM-Block-Einstellungen.png')
    d.click('mks:rom:results:'+model['id']);wait_handle(d,'fem:deformation:ready')
    d.screenshot('03-ROM-Bauteil-Verformung.png')
    d.click('fem:rom:motion');d.text('fem:rom:time','1.25');d.call('ui.key',key='Enter')
    d.text('fem:deformation:scale','500');d.call('ui.key',key='Enter');d.click('fem:deformation:fit')
    wait_handle(d,'fem:deformation:ready');time.sleep(.5)
    d.screenshot('05-Bauteil-Aus-MKS-Ergebnis.png')
    d.click('project:mks');d.click('mks:library:add:rom')
    d.click('mks:rom:prepare:'+model['block'])
    wait_handle(d,'fem:rom:interface:Arm1_Root')
    d.screenshot('04-ROM-Anlegen-Aus-MKS.png')
    report=dict(passed=True,motor_volume_mm3=measure[0]['volume_mm3'],analytic_volume_mm3=expected,
                motor_mass_g=measure[0]['mass_g'],normal_library=True,rom_library_entry=True,rom_drag_to_second_body=True,
                rom_block_results_navigation=True,mks_history_component_view=True,component_setup_from_mks=True)
    (d.out/'verification.json').write_text(json.dumps(report,indent=2),encoding='utf-8')
    print(json.dumps(report),flush=True)
