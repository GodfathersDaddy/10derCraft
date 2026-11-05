"""Construct a tube-section variant through normal native CAD commands.

This creates geometry and its calculation document. It does not run test suites
or certify the remaining robot joints and drives for manufacture.
"""
from pathlib import Path
import argparse
import json
import os
import re
import subprocess


def tube_source(source, stock=False):
    if stock:
        source = re.sub(r'^R_Oberarm\s*=.*$', 'R_Oberarm = 80mm', source, flags=re.M)
        source = re.sub(r'^R_Unterarm\s*=.*$', 'R_Unterarm = 70mm', source, flags=re.M)
    source = source.replace('Vollrundtraeger', 'Rohrquerschnitte')
    source = source.replace('Illustratives Modell, keine Herstellerdaten oder Festigkeitsfreigabe',
                            'Rohrprofil-Entwurf, Gelenke und Antriebe noch auszulegen')
    # Thicknesses precede every dependent expression, as in a normal worksheet.
    source = 't_Oberarm = 8mm\nt_Unterarm = 6mm\n' + source
    for section in ['Oberarm', 'Unterarm']:
        outer = f'R_{section}'
        inner = f'(R_{section}-t_{section})'
        formulas = {
            f'I_{section}': f'pi*({outer}^4-{inner}^4)/4',
            f'A_{section}_mm2': f'pi*(({outer}/1mm)^2-({inner}/1mm)^2)',
            f'I_{section}_mm4': f'pi*(({outer}/1mm)^4-({inner}/1mm)^4)/4',
        }
        for name, expression in formulas.items():
            source = re.sub(rf'^{name}\s*=.*$', f'{name} = {expression}', source, flags=re.M)
    source = re.sub(r'^F_TCP_N\s*=.*$', 'F_TCP_N = 8*9.81', source, flags=re.M)
    return source


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--cli', type=Path, required=True)
    parser.add_argument('--source', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=True)
    cli = str(args.cli.resolve())
    source = str(args.source.resolve())
    # Read the embedded calculation and every existing variant from the native
    # design, keeping its original source file and prior results separate.
    exported = output/'source-document.json'
    subprocess.run([cli, 'run', '--in', source, '--save', str(exported), '--quiet'], check=True)
    document = json.loads(exported.read_text(encoding='utf-8'))
    metadata = document['metadata']
    current = metadata['org.solidcraft.calcpad']['source']
    variants = metadata['org.solidcraft.calculation-variants']['items']
    commands = []

    def command(command_id, **params):
        commands.append(dict(command=command_id, params=params))

    # Upgrade all saved calculation alternatives so selecting an earlier row
    # still provides both wall-thickness references used by the new CAD cuts.
    for variant in variants:
        command('worksheet.load', document=dict(format='solidcraft-math/1', definitions=variant['definitions']))
        command('worksheet.calcpad.calculate', source=tube_source(variant['calcpad']['source']))
        command('worksheet.variant.save', name=variant['name'])
    command('worksheet.calcpad.calculate', source=tube_source(current, stock=True))
    for name in ['t_Oberarm', 't_Unterarm']:
        command('worksheet.reference', source=name, enabled=True)
    command('worksheet.apply')
    command('worksheet.variant.save', name='Metall-Rohrprofile 8kg - Entwurf')
    for component, body, base, axis, length in [
        ('upper', 'Oberarm', [0., 0., 405.], [0.46574643283262235, 0., 0.8849182223819825], 644.1),
        ('forearm', 'Unterarm', [320., 0., 972.], [0.9897905064830298, 0., -0.14252983293355628], 505.2),
    ]:
        command('component.activate', component=component)
        command('solid.cylinder', name=body+'_Innenbohrung',
                base=[x-u for x, u in zip(base, axis)], axis=axis,
                radius=f'ref_R_{body}-ref_t_{body}', height=length+2,
                operation='cut', targets=[body])
        command('material.assign', bodies=[body], material='Aluminum')
        command('component.metadata.set', component=component,
                properties=dict(FabricationStage='Tube-section concept', StructuralMaterial='Aluminium, grade pending',
                                ConnectionDesign='Joint housings, bearings and drive interfaces pending',
                                TubeOutsideDiameter_mm='160' if body=='Oberarm' else '140',
                                TubeWall_mm='8' if body=='Oberarm' else '6'))
    command('component.activate', component=0)
    command('view.isometric')
    command('view.fit')
    project = output/'Roboter-Rohrprofile-8kg-Entwurf.solidcraft'
    command('file.save', path=str(project))
    command('assembly.bom', path=str(output/'Roboter-Stueckliste.csv'))
    for body in ['Oberarm', 'Unterarm']:
        command('file.export', path=str(output/f'{body}-Rohr.step'), bodies=[body])
    recipe = output/'profile-recipe.json'
    recipe.write_text(json.dumps(dict(commands=commands), indent=2), encoding='utf-8')
    subprocess.run([cli, 'run', str(recipe), '--in', source, '--quiet'], env=os.environ, check=True)
    print(json.dumps(dict(project=str(project), state='manufacturing concept, joint design pending')))


if __name__ == '__main__':
    main()
