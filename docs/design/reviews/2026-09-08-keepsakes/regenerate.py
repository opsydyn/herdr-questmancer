"""Run with a Python containing Pillow to regenerate the production review."""
from pathlib import Path
import subprocess
import sys
import tempfile

HERE = Path(__file__).resolve().parent
REPO = HERE.parents[3]
subprocess.run(['cargo', 'build', '--lib', '--features', 'storybook'], cwd=REPO, check=True)
deps = REPO / 'target/debug/deps'
with tempfile.TemporaryDirectory(prefix='questmancer-keepsakes-') as temporary:
    output = Path(temporary)
    command = ['rustc', '--edition=2024', '-Dwarnings', str(HERE / 'export.rs'),
               '-L', 'dependency=' + str(deps), '--extern',
               'questmancer=' + str(REPO / 'target/debug/libquestmancer.rlib')]
    for library in ['ratatui', 'serde_json']:
        artifact = max(deps.glob('lib' + library + '-*.rlib'), key=lambda p: p.stat().st_mtime)
        command.extend(['--extern', library + '=' + str(artifact)])
    command.extend(['-o', str(output / 'export')])
    subprocess.run(command, cwd=REPO, check=True)
    subprocess.run([str(output / 'export'), str(output / 'data.json')], check=True)
    subprocess.run([sys.executable, str(HERE / 'layout.py'), str(output / 'data.json')], check=True)
