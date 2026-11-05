"""Package an existing local build and its separate scientific runtimes.

Does not build, run tests, execute the packaged application, or access credentials.
"""
from pathlib import Path
import argparse
import hashlib
import json
import shutil
import subprocess
import sys
import zipfile


def copy_tree(source, target):
    shutil.copytree(source, target, ignore=shutil.ignore_patterns('.venv', '__pycache__', '*.pyc'), dirs_exist_ok=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--build-dir', type=Path, required=True)
    parser.add_argument('--calcpad-dir', type=Path, required=True)
    parser.add_argument('--python-env', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--dotnet-notices', type=Path, required=True)
    args = parser.parse_args()
    repo = Path(__file__).resolve().parents[2]
    commit = subprocess.check_output(['git', '-C', str(repo), 'rev-parse', 'HEAD'], text=True).strip()
    folder = args.output.resolve() / f'10derCraft-windows-x64-dev-{commit[:8]}'
    if folder.exists():
        raise FileExistsError(f'Choose a new output folder, already exists: {folder}')
    # Fail before staging partial runtimes when a required input is absent.
    for source in [args.build_dir/'solidcraft.exe', args.build_dir/'solidcraft-cli.exe',
                   args.calcpad_dir/'CalcpadLocal.exe', args.calcpad_dir/'CalcpadCE-LICENSE.txt',
                   args.python_env/'Lib/site-packages/exudyn', args.dotnet_notices]:
        if not source.exists():
            raise FileNotFoundError(source)
    python_base = Path(sys.base_prefix)
    if sys.platform != 'win32' or not (python_base/'python.exe').is_file():
        raise RuntimeError('Run with the Windows scientific Python environment')
    folder.mkdir(parents=True)
    (folder/'bin').mkdir()
    for name in ['solidcraft.exe', 'solidcraft-cli.exe']:
        shutil.copy2(args.build_dir/name, folder/'bin'/name)
    for name in ['Start-10derCraft.cmd', '10derCraft.examples.json', 'LICENSE-MIT',
                 'LICENSE-APACHE', 'THIRD-PARTY-LICENSES.txt', 'ATTRIBUTION.md', 'README.md']:
        shutil.copy2(repo/name, folder/name)
    copy_tree(repo/'examples', folder/'examples')
    copy_tree(repo/'docs', folder/'docs')
    copy_tree(repo/'attribution', folder/'attribution')
    copy_tree(repo/'simulation', folder/'simulation')
    # Scientific environments are built separately. Keep the library and licence
    # files, and avoid copying their machine-specific venv launcher/configuration.
    python = folder/'python'
    python.mkdir()
    for source in python_base.iterdir():
        if source.is_file() and (source.suffix.lower() in ['.dll', '.exe'] or source.name == 'LICENSE.txt'):
            shutil.copy2(source, python/source.name)
    copy_tree(python_base/'DLLs', python/'DLLs')
    shutil.copytree(python_base/'Lib', python/'Lib',
                    ignore=shutil.ignore_patterns('site-packages', '__pycache__', '*.pyc'))
    copy_tree(args.python_env/'Lib/site-packages', python/'Lib/site-packages')
    copy_tree(args.calcpad_dir, folder/'calcpad')
    shutil.copy2(args.dotnet_notices, folder/'calcpad/DOTNET-ThirdPartyNotices.txt')
    info = dict(commit=commit, platform='windows-x64', status='development snapshot',
                tests='Broad tests paused at owner request. Packaging does not run acceptance tests.',
                python=sys.version, scientific_packages='See python/Lib/site-packages/*.dist-info',
                components=['10derCraft GUI', 'CLI/MCP', 'example projects', 'local Calcpad helper',
                            'Python runtime', 'Exudyn', 'NumPy', 'SciPy'])
    (folder/'BUILD_INFO.json').write_text(json.dumps(info, indent=2)+'\n', encoding='utf-8')
    files = sorted(p for p in folder.rglob('*') if p.is_file())
    manifest = ''.join(f'{hashlib.sha256(p.read_bytes()).hexdigest()}  {p.relative_to(folder).as_posix()}\n' for p in files)
    (folder/'SHA256SUMS.txt').write_text(manifest, encoding='utf-8')
    archive = folder.with_suffix('.zip')
    with zipfile.ZipFile(archive, 'x', compression=zipfile.ZIP_DEFLATED, compresslevel=6) as output:
        for source in sorted(p for p in folder.rglob('*') if p.is_file()):
            output.write(source, Path(folder.name)/source.relative_to(folder))
    digest = hashlib.sha256(archive.read_bytes()).hexdigest()
    archive.with_suffix('.zip.sha256').write_text(f'{digest}  {archive.name}\n', encoding='ascii')
    print(json.dumps(dict(archive=str(archive), bytes=archive.stat().st_size, sha256=digest)))


if __name__ == '__main__':
    main()
