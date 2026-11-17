import os,socket,pathlib,json,subprocess,sys,time
ROOT=pathlib.Path(os.environ.get('QUESTMANCER_TRIAL_ROOT', str(pathlib.Path(__file__).resolve().parent)));SOCK=ROOT/'config/herdr/herdr.sock';HERDR='/Users/alancurrie/.local/bin/herdr'
def request(method,params=None):
 with socket.socket(socket.AF_UNIX) as s:
  s.settimeout(5);s.connect(str(SOCK));s.sendall((json.dumps({'id':'native-trial','method':method,'params':params or {}})+'\n').encode());r=json.loads(s.makefile('rb').readline());assert 'error' not in r,(method,r);return r['result']
def env():
 e={k:v for k,v in os.environ.items() if not k.startswith('HERDR_') and k not in ('ENV','BASH_ENV')};e.update(XDG_CONFIG_HOME=str(ROOT/'config'),XDG_STATE_HOME=str(ROOT/'state'),XDG_CACHE_HOME=str(ROOT/'cache'),HERDR_CONFIG_PATH=str(ROOT/'config/herdr/config.toml'));return e
if __name__=='__main__':
 mode=sys.argv[1] if len(sys.argv)>1 else 'attach'
 assert request('ping')['version']=='0.9.5'
 if mode=='attach':subprocess.run([HERDR],env=env(),check=True)
 elif mode=='cleanup':
  r=json.loads((ROOT/'receipt.json').read_text());assert SOCK.stat().st_ino==r['server_socket_inode'], 'Socket identity changed; re-baseline before cleanup';os.kill(r['owned_server_pid'],0);snap=request('session.snapshot')['snapshot'];allowed={a['pane_id'] for a in r['panes']}|{r['managed_pane'],r['capability_probe']['pane_id']};assert {p['pane_id'] for p in snap['panes']}<=allowed, 'Unexpected panes: preserve session and re-baseline';by_id={p['pane_id']:p for p in snap['panes']}
  # This server started empty and belongs exclusively to this trial.
  for a in r['panes']:
   if a['pane_id'] in by_id:request('pane.release_agent',{'pane_id':a['pane_id'],'source':a['source'],'agent':a['agent'],'seq':10000})
  request('server.stop');print('Stopped only this trial server; files retained:',ROOT)
 elif mode=='refresh':
  r=json.loads((ROOT/'receipt.json').read_text());pid=r['managed_pane'];snap=request('session.snapshot')['snapshot'];pane=next((p for p in snap['panes'] if p['pane_id']==pid),None)
  if pane:
   runtime_files=list((ROOT/'config').rglob('runtime.json'))+list((ROOT/'state').rglob('runtime.json'))
   assert any(json.loads(p.read_text()).get('pane_id')==pid for p in runtime_files), 'Owned managed-pane registration changed; stop and re-baseline'
   request('pane.close',{'pane_id':pid})
  subprocess.run([HERDR,'plugin','action','invoke','opsydyn.questmancer.guild'],env=env(),check=True)
  for _ in range(50):
   files=list((ROOT/'config').rglob('runtime.json'))+list((ROOT/'state').rglob('runtime.json'))
   if files:
    current=json.loads(files[0].read_text())
    if current.get('pane_id')!=pid and current.get('pid',0)>0:
     r['managed_pane']=current['pane_id'];(ROOT/'receipt.json').write_text(json.dumps(r,indent=2)+'\n');break
   time.sleep(.1)
  else:raise SystemExit('Native refresh not confirmed; inspect test session')
  print('Fresh own-pane startup; native appearance still needs your review.')
 elif mode in ('guild','delve','open'):
  subprocess.run([HERDR,'plugin','action','invoke','opsydyn.questmancer.'+mode],env=env(),check=True)
 else:raise SystemExit('Usage: control.py [attach|guild|delve|open|cleanup]')
