"""Exercise the actual linked plugin only inside a freshly owned Herdr server.

No outer terminal is controlled; this is runtime integration, not native visual
acceptance. All created resources are recorded and cleaned up even on failure.
"""
from pathlib import Path
import hashlib,json,os,shutil,signal,socket,subprocess,tempfile,time,tomllib
REPO=Path(__file__).resolve().parents[3]
HERDR=shutil.which('herdr')
PLUGIN='opsydyn.questmancer'
VERSION=tomllib.loads((REPO/'Cargo.toml').read_text())['package']['version']
CASE=Path(tempfile.mkdtemp(prefix='qm-rc-',dir='/tmp'))
CONFIG=CASE/'config/herdr';CONFIG.mkdir(parents=True)
(CONFIG/'config.toml').write_text('onboarding = false\n[server]\nheadless_cols = 180\nheadless_rows = 60\n[terminal]\ndefault_shell = "/bin/sh"\nshell_mode = "non_login"\n[update]\nversion_check = false\nmanifest_check = false\n[session]\nresume_agents_on_restore = false\n')
ENV={k:v for k,v in os.environ.items() if not k.startswith('HERDR_') and k not in {'ENV','BASH_ENV'}}
ENV.update(XDG_CONFIG_HOME=str(CASE/'config'),XDG_STATE_HOME=str(CASE/'state'),XDG_CACHE_HOME=str(CASE/'cache'),SHELL='/bin/sh',TERM='xterm-256color')
SOCKET=CONFIG/'herdr.sock'
log=(CASE/'server-launch.log').open('wb')
server=subprocess.Popen([HERDR,'server'],cwd=CASE,env=ENV,stdout=log,stderr=subprocess.STDOUT,start_new_session=True)
receipt={'case':str(CASE),'owned_server_pid':server.pid,'candidate_version':VERSION,'binary_sha256':hashlib.sha256((REPO/'target/release/questmancer').read_bytes()).hexdigest(),'panes':[],'actions':[],'native_visual_acceptance':'not exercised','cleanup_errors':[]}
linked=False;managed=None;original_focus=None;counter=0

def request(method,params=None):
 global counter
 counter+=1
 with socket.socket(socket.AF_UNIX,socket.SOCK_STREAM) as client:
  client.settimeout(4);client.connect(str(SOCKET));client.sendall((json.dumps({'id':str(counter),'method':method,'params':params or {}})+'\n').encode())
  response=json.loads(client.makefile('rb').readline())
  assert 'error' not in response,(method,response)
  return response['result']

def cli(*args):
 result=subprocess.run([HERDR,*args],cwd=CASE,env=ENV,capture_output=True,text=True,timeout=15)
 assert result.returncode==0,(args,result.stdout,result.stderr)
 return json.loads(result.stdout)

def wait_for(predicate,label,seconds=15):
 end=time.monotonic()+seconds
 while time.monotonic()<end:
  value=predicate()
  if value:return value
  assert server.poll() is None,'owned server exited'
  time.sleep(.1)
 raise AssertionError('Timed out: '+label)

def snapshot():return request('session.snapshot')['snapshot']
def runtimes():return list(CONFIG.rglob('runtime.json'))+list((CASE/'state').rglob('runtime.json'))
def action(name):
 result=cli('plugin','action','invoke',PLUGIN+'.'+name)
 receipt['actions'].append({'action':name,'response':result})
 return result

def condition(pane,expected):
 info=request('pane.get',{'pane_id':pane})['pane']
 return info if info.get('tokens',{}).get('quest_condition')==expected else None

try:
 wait_for(lambda:SOCKET.exists(),'owned server socket')
 assert request('ping')['version']=='0.9.0'
 before=snapshot();assert not before['panes'],before
 assert cli('plugin','list','--json')['result']['plugins']==[]
 for i,state in enumerate(['working','blocked','idle','unknown']):
  before_ids={p['pane_id'] for p in snapshot()['panes']}
  request('workspace.create',{'cwd':str(CASE),'label':'Review campaign '+str(i+1),'focus':i==0})
  added=[p for p in snapshot()['panes'] if p['pane_id'] not in before_ids];assert len(added)==1
  pane=added[0]['pane_id'];source='questmancer-rc-'+str(i)
  receipt['panes'].append({'pane_id':pane,'workspace_id':added[0]['workspace_id'],'source':source,'agent':'review-'+str(i),'state':state})
  request('pane.report_agent',{'pane_id':pane,'source':source,'agent':'review-'+str(i),'state':state,'seq':1})
 original_focus=next(p['pane_id'] for p in snapshot()['panes'] if p['focused'])
 cli('plugin','link',str(REPO));linked=True
 plain={p['pane_id'] for p in snapshot()['panes']}
 action('open')
 new=wait_for(lambda:[p for p in snapshot()['panes'] if p['pane_id'] not in plain],'managed pane')
 assert len(new)==1;managed=new[0]['pane_id'];receipt['managed_pane']=managed
 wait_for(lambda:runtimes(),'runtime registration')
 runtime_path=runtimes()[0]
 runtime=wait_for(lambda:(lambda r:r if r.get('pid',0)>0 else None)(json.loads(runtime_path.read_text())),'running plugin PID')
 assert runtime['pane_id']==managed
 command=subprocess.check_output(['ps','-p',str(runtime['pid']),'-o','command='],text=True).strip()
 assert str(REPO/'target/release/questmancer') in command,command
 receipt['runtime_command']=command
 action('open')
 assert {p['pane_id'] for p in snapshot()['panes']}==plain|{managed},'second open duplicated pane'
 expected=['Concentrating','Restrained','Resting','Blinded']
 for owned,wanted in zip(receipt['panes'],expected):
  info=wait_for(lambda:condition(owned['pane_id'],wanted),'production '+wanted)
  tokens=info['tokens'];assert tokens['quest_sigil'] and tokens['quest_role'] and tokens['quest_hoard']=='◈ empty'
  owned['projected_tokens']={k:v for k,v in tokens.items() if k.startswith('quest_')}
 assert not request('pane.get',{'pane_id':managed})['pane'].get('tokens',{}).get('quest_role'),'managed pane reported as adventurer'
 for name in ['guild','delve','guild']:action(name)
 assert {p['pane_id'] for p in snapshot()['panes']}==plain|{managed},'view switch duplicated pane'
 changed=receipt['panes'][0]
 request('pane.report_agent',{'pane_id':changed['pane_id'],'source':changed['source'],'agent':changed['agent'],'state':'blocked','seq':2})
 wait_for(lambda:condition(changed['pane_id'],'Restrained'),'working to counsel')
 request('pane.report_agent',{'pane_id':changed['pane_id'],'source':changed['source'],'agent':changed['agent'],'state':'working','seq':3})
 wait_for(lambda:condition(changed['pane_id'],'Concentrating'),'counsel to working')
 receipt['runtime_checks']='passed'
 # Action records alone establish invocation, not room appearance.
 receipt['plugin_logs']=cli('plugin','log','list','--plugin',PLUGIN,'--limit','30')
 (CASE/'pre-cleanup-snapshot.json').write_text(json.dumps(snapshot(),indent=2)+'\n')
except Exception as error:
 receipt['failure']=repr(error)
finally:
 if server.poll() is None:
  if managed is not None:
   try:
    action('close')
    wait_for(lambda:not any(p['pane_id']==managed for p in snapshot()['panes']),'managed pane closed')
    receipt['managed_pane_closed']=True
   except Exception as error:receipt['cleanup_errors'].append(repr(error))
  if original_focus is not None:
   try:request('pane.focus',{'pane_id':original_focus});receipt['owned_focus_restored']=True
   except Exception as error:receipt['cleanup_errors'].append(repr(error))
  for owned in receipt['panes']:
   try:
    request('pane.release_agent',{'pane_id':owned['pane_id'],'source':owned['source'],'agent':owned['agent'],'seq':100})
    assert not any(a['pane_id']==owned['pane_id'] for a in snapshot()['agents'])
    owned['identity_released']=True
    request('pane.close',{'pane_id':owned['pane_id']});owned['pane_closed']=True
   except Exception as error:receipt['cleanup_errors'].append(repr(error))
  if linked:
   try:cli('plugin','unlink',PLUGIN);receipt['test_link_removed']=True
   except Exception as error:receipt['cleanup_errors'].append(repr(error))
  try:request('server.stop')
  except Exception as error:receipt['stop_response']=repr(error)
  try:server.wait(timeout=8)
  except subprocess.TimeoutExpired:
   os.killpg(server.pid,signal.SIGTERM);server.wait(timeout=3);receipt['forced_cleanup']=True
 log.close()
 receipt['server_exit_code']=server.returncode;receipt['socket_removed']=not SOCKET.exists()
 (CASE/'receipt.json').write_text(json.dumps(receipt,indent=2)+'\n')
 print(json.dumps(receipt,indent=2))
assert receipt.get('runtime_checks')=='passed' and not receipt['cleanup_errors'] and receipt['server_exit_code']==0 and receipt['socket_removed'],receipt
