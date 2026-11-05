from cad_test_client import Desktop,ROOT,SOURCE
import json,time,hashlib
def wait(d,name):
    for _ in range(150):
        if any(h['name']==name for h in d.call('ui.handles',prefix=name)):return
        time.sleep(.1)
    raise RuntimeError(name)
project=SOURCE/'examples/projects/Flexarm-FEM-ROM-MKS.solidcraft'
before=hashlib.sha256(project.read_bytes()).hexdigest()
with Desktop('fem-rom-showcase',port=7972) as d:
    d.call('ui.resize',width=1600,height=1000);d.cmd('doc.open',path=str(project));d.settled()
    d.click('project:fem');d.click('fem:workspace:results');wait(d,'fem:deformation:ready')
    d.click('fem:deformation:grid');d.text('fem:deformation:scale','5000');d.call('ui.key',key='Enter');d.click('fem:deformation:fit');wait(d,'fem:deformation:ready');time.sleep(.5)
    d.screenshot('02-Volumen-FEM-Netz-Verformung.png')
    d.click('fem:interfaces:toggle');wait(d,'fem:deformation:ready');time.sleep(.3);d.screenshot('01-CAD-Transferflaechen.png')
    d.cmd('fem.mks.bind',id='Component_1',enabled=False)
    job=d.cmd('mks.job.create',duration_s=.02,steps=20,samples=21,drivers=[],port_loads=[])
    assert job['backend']=='exudyn_rigid_open_chain'
    result=d.cmd('mks.job.run',duration_s=.02,steps=20,samples=21,drivers=[],port_loads=[])
    assert not result.get('flexible_states')
    d.cmd('fem.mks.bind',id='Component_1',enabled=True)
    assert not d.cmd('mks.job.state')['result_current']
    report=dict(passed=True,rigid_fallback=True,rebind_invalidates_old_result=True,source_project_unchanged=before==hashlib.sha256(project.read_bytes()).hexdigest())
    assert report['source_project_unchanged']
    (d.out/'verification.json').write_text(json.dumps(report,indent=2),encoding='utf-8');print(json.dumps(report))
