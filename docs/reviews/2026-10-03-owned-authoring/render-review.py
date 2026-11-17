"""Reproduce production cell buffers and PNGs with Rust and Pillow on macOS."""
import os,pathlib,subprocess,sys,tempfile
pack=pathlib.Path(__file__).resolve().parent
root=pack.parents[2]
os.chdir(root)
env=dict(os.environ,CARGO_INCREMENTAL='0')
subprocess.run(['cargo','build','--lib','--features','storybook'],env=env,check=True)
deps=root/'target/debug/deps'
with tempfile.TemporaryDirectory(prefix='questmancer-review-') as temp:
 binary=pathlib.Path(temp)/'render-review'
 command=['rustc','--edition=2024',str(pack/'render-review.rs'),'-L','dependency='+str(deps),'-o',str(binary),'--extern','questmancer='+str(root/'target/debug/libquestmancer.rlib')]
 for name in ('ratatui','serde_json'):
  library=max(deps.glob('lib'+name+'-*.rlib'),key=lambda path:path.stat().st_mtime)
  command+=['--extern',name+'='+str(library)]
 subprocess.run(command,check=True)
 subprocess.run([str(binary),str(pack/'visuals')],check=True)
 subprocess.run([sys.executable,str(pack/'render-review-images.py')],check=True)
