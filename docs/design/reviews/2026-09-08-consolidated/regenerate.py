"""Re-export current production reviews, retaining unchanged historical files."""
from pathlib import Path
import hashlib,json,os,shutil,subprocess,tempfile
ROOT=Path(__file__).resolve().parent
REPO=ROOT.parents[3]
PYTHON=os.environ.get('REVIEW_PYTHON','python3')
with tempfile.TemporaryDirectory(prefix='questmancer-consolidated-') as tmp:
 work=Path(tmp)
 with (work/'build.jsonl').open('w') as output:
  subprocess.run(['cargo','build','--lib','--features','storybook','--message-format=json'],cwd=REPO,stdout=output,check=True)
 paths={}
 for line in (work/'build.jsonl').read_text().splitlines():
  item=json.loads(line)
  if item.get('reason')=='compiler-artifact' and item['target']['name'] in {'questmancer','ratatui','serde_json'}:
   for filename in item['filenames']:
    if filename.endswith('.rlib'):paths[item['target']['name']]=filename
 assert len(paths)==3
 receipt={'source_version':'0.1.9','native_visual_acceptance':'pending','exports':[]}
 for name in ['librarian','party-pilot','card-fallbacks']:
  source=ROOT.parent/('2026-09-05-'+name);out=work/name;out.mkdir()
  for filename in ['layout.py','baseline.json']:
   if (source/filename).exists():shutil.copyfile(source/filename,out/filename)
  layout=out/'layout.py'
  layout.write_text(layout.read_text().replace('ROOT.parents[3]', 'Path('+repr(str(REPO))+')'))
  command=['rustc','--edition=2024','-D','warnings',str(source/'export.rs'),'-L','dependency='+str(Path(paths['serde_json']).parent),'-o',str(out/'export')]
  for package,path in paths.items():command+=['--extern',package+'='+path]
  subprocess.run(command,cwd=REPO,check=True)
  subprocess.run([str(out/'export'),str(out),str(out/'rendered.json')],cwd=REPO,check=True)
  subprocess.run([PYTHON,str(out/'layout.py'),str(out/'rendered.json')],cwd=REPO,check=True)
  for asset in sorted(out.iterdir()):
   if asset.suffix not in {'.png','.gif'}:continue
   previous=source/asset.name
   matches=previous.exists() and previous.read_bytes()==asset.read_bytes()
   if matches:target=previous
   else:
    target=ROOT/(name+'-'+asset.name);shutil.copyfile(asset,target)
   receipt['exports'].append({'group':name,'file':os.path.relpath(target,ROOT),'matches_previous_export':matches,'sha256':hashlib.sha256(asset.read_bytes()).hexdigest()})
 (ROOT/'exports.json').write_text(json.dumps(receipt,indent=2)+'\n')
 print(json.dumps(receipt,indent=2))
