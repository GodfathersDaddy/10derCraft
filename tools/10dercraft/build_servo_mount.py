"""Rebuild the motor-side subassembly through normal native CAD commands."""
from pathlib import Path
import argparse
import json
import math
import subprocess


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--cli', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    repo = Path(__file__).resolve().parents[2]
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=True)
    cli = str(args.cli.resolve())
    recipe = json.loads((repo/'examples/servo-motor-mount.json').read_text(encoding='utf-8'))
    recipe['commands'].append(dict(command='component.list', params={}))
    base = output/'base-recipe.json'
    base.write_text(json.dumps(recipe, indent=2), encoding='utf-8')
    results = output/'base-results.json'
    subprocess.run([cli, 'run', str(base), '--results', '--json-out', str(results)], cwd=output, check=True)
    components = json.loads(results.read_text(encoding='utf-8'))[-1]['components']
    hardware = [body for component in components
                if component['name'].startswith(('A2_ISO7089', 'A2_ISO4762'))
                for body in component['bodies']]
    commands = []

    def add(command_id, **params):
        commands.append(dict(command=command_id, params=params))

    add('material.assign', bodies=hardware, material='Steel')
    add('component.metadata.set', component='A2_Panasonic_MSMF202L1H8', bom_type='purchased',
        properties={'Representation': 'Mounting envelope, catalogue mass 6.6 kg; no rotor/electrical model'})
    add('component.activate', component=0)
    add('component.create', name='A2_Servomotor_Montage', parent=0, activate=False, grounded=True)
    add('occurrence.ground', occurrence='A2_Motoradapter:1', grounded=False)
    for component in components:
        for occurrence in component['occurrences']:
            add('occurrence.reparent', occurrence=occurrence['id'], parent='A2_Servomotor_Montage')
    add('joint.as_built', name='Adapter_im_Motormodul', type='rigid',
        a='A2_Servomotor_Montage', b='A2_Motoradapter',
        at=dict(occurrence='A2_Servomotor_Montage', point=[0, 0, 0], z=[0, 0, 1]))
    add('component.activate', component='A2_Motoradapter')
    for i in range(4):
        angle = math.pi/4+i*math.pi/2
        add('solid.thread', face=[57.5*math.cos(angle)+3.4, 57.5*math.sin(angle), 7.5],
            designation='M8x1.25', length=15, modeled=False)
    for i in range(6):
        angle = i*60
        add('timeline.edit', feature=f'Gehaeuse_M8_Durchgang_{i}',
            set={'base': [f'housing_PCD/2*cos({angle} deg)', f'housing_PCD/2*sin({angle} deg)', -1]})
    add('component.activate', component=0)
    add('component.metadata.set', component='A2_Servomotor_Montage',
        part_number='RB-A2-MOTORMOUNT-001',
        description='Motor-side servo mounting module; gear and output bearing connection pending')
    add('file.save', path=str(output/'A2-Servomotor-Montage.solidcraft'))
    add('file.export', path=str(output/'A2-Motoradapter.step'), bodies=['A2_Adapterplatte'])
    add('assembly.bom', path=str(output/'A2-Montage-Stueckliste.csv'))
    final = output/'finish-recipe.json'
    final.write_text(json.dumps(dict(commands=commands), indent=2), encoding='utf-8')
    subprocess.run([cli, 'run', str(final), '--in', str(output/'A2-Servomotor-Montage.solidcraft'), '--quiet'],
                   cwd=output, check=True)
    print(output/'A2-Servomotor-Montage.solidcraft')


if __name__ == '__main__':
    main()
