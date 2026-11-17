"""Export actual production and isolated transparent-NPC backgrounds for review."""
from pathlib import Path
import os, subprocess, tempfile, shutil, json, re, sys
ROOT=Path(__file__).resolve().parent
REPO=ROOT.parents[3]

def export(manifest, output, artifact_name="questmancer"):
    command=['cargo','build','--manifest-path',str(manifest),'--lib','--features','storybook','--offline','--message-format=json','--target-dir',str(REPO/'target')]
    if artifact_name=='questmancer':command.append('--locked')
    result=subprocess.run(command,cwd=REPO,check=True,capture_output=True,text=True,env={**os.environ,'CARGO_INCREMENTAL':'0'})
    print(result.stderr,end='')
    libraries={}
    for line in result.stdout.splitlines():
        item=json.loads(line)
        if item.get('reason')=='compiler-artifact' and item['target']['name'] in {artifact_name,'ratatui','serde_json'}:
            for name in item['filenames']:
                if name.endswith('.rlib'):libraries['questmancer' if item['target']['name']==artifact_name else item['target']['name']]=name
    assert len(libraries)==3
    with tempfile.TemporaryDirectory(prefix='questmancer-librarian-export-') as temporary:
        binary=Path(temporary)/'export'
        cmd=['rustc','--edition=2024','-D','warnings',str(ROOT/'export.rs'),'-L','dependency='+str(Path(libraries['serde_json']).parent),'-o',str(binary)]
        for name,path in libraries.items():cmd+=['--extern',name+'='+path]
        subprocess.run(cmd,cwd=REPO,check=True)
        subprocess.run([str(binary),str(ROOT),str(output)],cwd=REPO,check=True)

export(REPO/'Cargo.toml',ROOT/'current.json')
with tempfile.TemporaryDirectory(prefix='questmancer-librarian-background-') as temporary:
    scratch=Path(temporary)
    shutil.copytree(REPO/'src',scratch/'src')
    for filename in ['Cargo.toml','Cargo.lock','README.md']:
        shutil.copyfile(REPO/filename,scratch/filename)
    manifest=scratch/'Cargo.toml'
    manifest.write_text(manifest.read_text().replace('name = "questmancer"','name = "questmancer-librarian-background-study"',1))
    asset=scratch/'src/scene/assets/librarian.rs'
    text=asset.read_text()
    blank='const WORLD_ROWS: &[&str] = &[\n'+''.join('    "................",\n' for _ in range(24))+'];'
    text,count=re.subn(r'const WORLD_ROWS: &\[&str\] = &\[.*?\];',lambda _:blank,text,count=1,flags=re.S)
    assert count==1,'Transparent review-background substitution failed'
    asset.write_text(text)
    export(scratch/'Cargo.toml',ROOT/'background.json','questmancer_librarian_background_study')
current=json.loads((ROOT/'current.json').read_text())
background=json.loads((ROOT/'background.json').read_text())
assert sum(p is not None for p in current['frames']['current']['pixels'])>150, 'Current production Librarian is missing'
assert all(p is None for p in background['frames']['current']['pixels']), 'Background-only master was not removed'
subprocess.run([sys.executable,str(ROOT/'layout.py')],cwd=REPO,check=True)
