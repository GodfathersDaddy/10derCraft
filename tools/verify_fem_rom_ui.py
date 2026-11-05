"""Actual desktop acceptance: CAD surfaces, solid mesh, ROM, stale data, save/reopen."""
from cad_test_client import Desktop,ROOT
import json,time,os
def wait(d,handle):
    for _ in range(150):
        found=[h for h in d.call('ui.handles',prefix=handle) if h['name']==handle]
        if found:return found[0]
        time.sleep(.15)
    raise AssertionError('handle missing: '+handle)
with Desktop(os.environ.get('FEM_UI_OUTPUT','fem-rom-ui'),port=7968) as d:
    d.call('ui.resize',width=1600,height=1000)
    d.cmd('file.new',name='Volumen FEM und Komponenten ROM')
    arm=d.cmd('mks.library.body',name='Flexarm',mass_kg=78.5,size_mm=[1000,100,100],position_mm=[500,50,50])
    base=d.cmd('mks.library.body',name='Ground',mass_kg=10,size_mm=[100,100,100],position_mm=[-100,50,50],grounded=True)
    joint=d.cmd('mks.library.joint',name='Revolute',kind='revolute',a=base['occurrence'],b=arm['occurrence'],point_mm=[0,50,50],axis=[0,0,1])['joint']
    faces=d.cmd('model.faces',body=arm['body'])['faces']
    for x,name in [(0,'IF_Root'),(1000,'IF_Tip')]:
        face=next(f for f in faces if abs(f['centroid'][0]-x)<1e-6)
        d.cmd('select.set',items=[dict(type='face',body=arm['body'],index=face['index'],point=face['point'])])
        d.cmd('mks.interface.capture',name=name,occurrence=arm['occurrence'],transfer='frame_wrench',surface=True)
    d.click('project:fem');d.click('fem:interfaces:toggle')
    wait(d,'fem:deformation:ready');time.sleep(.5)
    d.screenshot('01-CAD-Kraftuebertragungsflaechen.png')
    d.click('fem:interfaces:toggle');d.click('fem:rom:open')
    wait(d,'fem:rom:interface:IF_Root');d.click('fem:rom:interface:IF_Root');d.click('fem:rom:interface:IF_Tip')
    d.screenshot('02-Volumen-FEM-ROM-Einstellungen.png')
    # Centre of the actual shared card in this 1600x1000 fixture.
    d.call('ui.scroll',x=800,y=440,delta=-500);time.sleep(.3)
    began=time.monotonic();d.click('fem:rom:flexible')
    for tick in range(100):
        d.call('ui.move',x=1500+(tick%2),y=160)
        state=d.cmd('fem.model.read')
        if any(m['has_flexible'] for m in state['models']):break
        diagnostic=d.call('ui.inspect').get('component_fem',{})
        if diagnostic.get('error'):
            (d.out/'async-error.json').write_text(json.dumps(diagnostic,indent=2),encoding='utf-8')
            raise AssertionError(diagnostic)
        time.sleep(.15)
    else:
        d.call('ui.scroll',x=800,y=440,delta=-1200);time.sleep(.5)
        (d.out/'last-error-state.json').write_text(json.dumps(dict(state=state,ui=d.call('ui.inspect'),handles=d.call('ui.handles',prefix='fem:')),indent=2),encoding='utf-8')
        try:d.screenshot('error-'+str(time.time_ns())+'.png')
        except RuntimeError:pass
        raise AssertionError('FEM result missing; see current error screenshot')
    elapsed=time.monotonic()-began
    wait(d,'fem:deformation:ready');d.click('fem:deformation:grid');wait(d,'fem:deformation:ready');time.sleep(.5)
    d.text('fem:deformation:scale','1000');d.call('ui.key',key='Enter');time.sleep(.8)
    d.screenshot('03-FE-Netz-und-Verformung.png')
    d.text('fem:deformation:scale','1');d.call('ui.key',key='Enter')
    d.click('fem:rom:motion');d.text('fem:rom:time','0.4');d.call('ui.key',key='Enter');time.sleep(1)
    d.screenshot('04-Flexible-Komponente.png')
    model=state['models'][0];assert model['static_rom_error']<1e-6 and abs(model['mass_kg']-78.5)<1e-8
    d.click('fem:rom:open');d.call('ui.scroll',x=800,y=440,delta=-700);time.sleep(.3)
    wait(d,'fem:rom:bind:'+model['id']);d.click('fem:rom:bind:'+model['id']);d.call('ui.key',key='Escape')
    assert d.cmd('fem.model.read')['models'][0]['enabled']
    result=d.cmd('mks.job.run',duration_s=1.,steps=4000,samples=1001,drivers=[dict(joint=joint,displacement='40 deg',ramp_s=1.,profile='quintic')])
    assert len(result['flexible_states'])==1
    assert result['summary']['max_position_constraint_m']<1e-5
    d.click('project:mks');d.click('mks:graph:run');time.sleep(1.2)
    if not d.call('ui.handles',prefix='mks:scope:plot'):d.click('mks:scope:toggle')
    time.sleep(.5)
    d.screenshot('05-Flexible-MKS-Gelenkkraefte.png')
    d.cmd('mks.result.snapshot',sample=1000,path=str(d.out/'06-Flexible-MKS-Netz.png'),width=1000,height=650,overwrite=True)
    d.cmd('mks.result.video',path=str(d.out/'Flexarm-Flexible-MKS.avi'),width=640,height=360,fps=30,overwrite=True)
    assert d.cmd('mks.result.pose',time_s=.4)['interpolation']=='reference_slerp_modal_linear'
    target=d.out/'Flexarm-FEM-ROM.solidcraft';d.cmd('file.save',path=str(target))
    exported=d.out/'Flexarm-ROM.json';d.cmd('fem.rom.export',id=model['id'],path=str(exported),overwrite=True)
    d.cmd('doc.open',path=str(target));d.settled()
    assert d.cmd('fem.model.read')['models'][0]['current']
    model=d.cmd('fem.model.read')['models'][0]
    assert d.cmd('mks.job.state')['result_current']
    d.cmd('parameters.change',name=f"mbs_{arm['occurrence']}_L",expression='1100 mm')
    assert not d.cmd('fem.model.read')['models'][0]['current']
    assert not d.cmd('mks.job.state')['result_current']
    try:d.cmd('fem.rom.reduce',id=model['id'])
    except RuntimeError:pass
    else:raise AssertionError('stale FEM reduced without rebuilding')
    (d.out/'verification.json').write_text(json.dumps(dict(passed=True,seconds=elapsed,model=model,project=str(target),export=str(exported)),indent=2),encoding='utf-8')
    print(json.dumps(dict(passed=True,seconds=elapsed,model=model)),flush=True)
