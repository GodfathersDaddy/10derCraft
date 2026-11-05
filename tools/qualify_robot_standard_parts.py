"""Assign explicit steel to the foundation hardware and requalify native dynamics."""
from cad_test_client import Desktop, ROOT
import json
folder=ROOT/'output/robot-standard-parts'
with Desktop('robot-standard-parts-qualified',port=7988) as d:
    d.call('ui.resize',width=1600,height=1000)
    d.cmd('doc.open',path=str(folder/'Roboter-Parametrisch-Normteile.solidcraft'));d.settled()
    components=d.cmd('component.list')['components']
    hardware=[c for c in components if c['name'].startswith(('ISO7089_M16_','ISO4017_M16x80_'))]
    assert len(hardware)==8
    names=[body for c in hardware for body in c['bodies']]
    d.cmd('material.assign',bodies=names,material='Steel')
    settings=d.cmd('mks.job.state')['data']['job']['settings']
    result=d.cmd('mks.job.run',**settings)
    assert result['summary']['max_position_constraint_m']<1e-5
    mass=d.cmd('inspect.measure',bodies=names)
    assert all(abs(b['density_g_cm3']-7.85)<1e-8 for b in mass['bodies'])
    d.cmd('file.save',path=str(folder/'Roboter-Parametrisch-Normteile.solidcraft'))
    d.cmd('view.isometric');d.cmd('view.fit');d.screenshot('01-Roboter-Normteile.png')
    report=dict(passed=True,steel_hardware_bodies=len(names),hardware_mass_g=mass['total']['mass_g'],summary=result['summary'])
    (d.out/'verification.json').write_text(json.dumps(report,indent=2),encoding='utf-8')
    print(json.dumps(report),flush=True)
