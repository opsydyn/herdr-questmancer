"""Exercise the actual linked plugin only inside a freshly owned Herdr server.

No outer terminal is controlled; this is runtime integration, not native visual
acceptance. All created resources are recorded and cleaned up even on failure.
"""
from pathlib import Path
import hashlib,json,os,shutil,signal,socket,subprocess,tempfile,time,tomllib,uuid
REPO=Path(__file__).resolve().parents[3]
HERDR=shutil.which('herdr')
PLUGIN='opsydyn.questmancer'
VERSION=tomllib.loads((REPO/'Cargo.toml').read_text())['package']['version']
CASE=Path(tempfile.mkdtemp(prefix='qm-c3-',dir='/tmp'))
CONFIG=CASE/'config/herdr';CONFIG.mkdir(parents=True)
(CONFIG/'config.toml').write_text('onboarding = false\n[server]\nheadless_cols = 180\nheadless_rows = 60\n[terminal]\ndefault_shell = "/bin/sh"\nshell_mode = "non_login"\n[update]\nversion_check = false\nmanifest_check = false\n[session]\nresume_agents_on_restore = false\n')
ENV={k:v for k,v in os.environ.items() if not k.startswith('HERDR_') and k not in {'ENV','BASH_ENV'}}
ENV.update(XDG_CONFIG_HOME=str(CASE/'config'),XDG_STATE_HOME=str(CASE/'state'),XDG_CACHE_HOME=str(CASE/'cache'),SHELL='/bin/sh',TERM='xterm-256color')
# A test-owned sleeper supplies process identity only; no model or AI session runs.
(CASE/'pi.rs').write_text('fn main() { std::thread::sleep(std::time::Duration::from_secs(300)); }\n')
subprocess.run(['rustc',str(CASE/'pi.rs'),'-o',str(CASE/'pi')],check=True)
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
  pane=added[0]['pane_id'];source='herdr:pi';session=str(uuid.uuid4())
  receipt['panes'].append({'pane_id':pane,'workspace_id':added[0]['workspace_id'],'source':source,'agent':'pi','session':session,'state':state})
  request('pane.send_text',{'pane_id':pane,'text':'exec '+str(CASE/'pi')})
  request('pane.send_keys',{'pane_id':pane,'keys':['enter']})
  wait_for(lambda:any(a['pane_id']==pane and a['agent']=='pi' for a in snapshot()['agents']),'dummy Pi process detected')
  request('pane.report_agent_session',{'pane_id':pane,'source':source,'agent':'pi','agent_session_id':session,'session_start_source':'startup','seq':0})
  request('pane.report_agent',{'pane_id':pane,'source':source,'agent':'pi','state':state,'seq':1,'agent_session_id':session})
 initial=snapshot();(CASE/'initial-snapshot.json').write_text(json.dumps(initial,indent=2)+'\n')
 assert len(initial['agents'])==4 and all(a.get('agent_session') for a in initial['agents']), 'synthetic identity not qualified'
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
 receipt['runtime_command']=command;receipt['runtime_pid']=runtime['pid']
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
 request('pane.report_agent',{'pane_id':changed['pane_id'],'source':changed['source'],'agent':changed['agent'],'agent_session_id':changed['session'],'state':'blocked','seq':2})
 wait_for(lambda:condition(changed['pane_id'],'Restrained'),'working to counsel')
 request('pane.report_agent',{'pane_id':changed['pane_id'],'source':changed['source'],'agent':changed['agent'],'agent_session_id':changed['session'],'state':'working','seq':3})
 wait_for(lambda:condition(changed['pane_id'],'Concentrating'),'counsel to working')
 # Validate actual writer bytes and status source; never call these snapshot observations.
 history_path=runtime_path.parent/'chronicle.jsonl'
 def records():
  return [json.loads(line) for line in history_path.read_text().splitlines()] if history_path.exists() else []
 first=wait_for(lambda:records(),'captured history')
 assert len(first)==2,first
 for seq,state,wanted in [(4,'idle','Resting'),(5,'unknown','Blinded'),(6,'working','Concentrating')]:
  request('pane.report_agent',{'pane_id':changed['pane_id'],'source':changed['source'],'agent':changed['agent'],'agent_session_id':changed['session'],'state':state,'seq':seq})
  wait_for(lambda:condition(changed['pane_id'],wanted),'state '+state)
  wait_for(lambda:len(records())==seq-1,'record '+state)
 captured=records()
 assert len(captured)==5,captured
 assert all(r['record_version']==2 and r['observation']['evidence']['source']=='pane_metadata' for r in captured),captured
 assert len({r['id'] for r in captured})==5
 assert [r['observation']['evidence']['presence'] for r in captured]==['blocked','working','idle','unknown','working']
 receipt['captured_records']=captured
 # A matching metadata report must not add history.
 request('pane.report_agent',{'pane_id':changed['pane_id'],'source':changed['source'],'agent':changed['agent'],'agent_session_id':changed['session'],'state':'working','seq':7})
 time.sleep(.5)
 assert records()==captured
 # Select a subject with captured records using only the owned plugin's UI.
 request('pane.send_keys',{'pane_id':managed,'keys':['c']})
 for _ in receipt['panes']:
  time.sleep(.15)
  read=request('pane.read',{'pane_id':managed,'source':'visible','format':'text','strip_ansi':True})
  if 'observed ' in read['read']['text']:break
  request('pane.send_keys',{'pane_id':managed,'keys':['esc','j','c']})
 assert 'CHRONICLE OF THIS ADVENTURER' in read['read']['text'] and 'observed ' in read['read']['text']
 (CASE/'records-pane-read.json').write_text(json.dumps(read,indent=2)+'\n')
 request('pane.send_keys',{'pane_id':managed,'keys':['tab']})
 time.sleep(.2)
 read=request('pane.read',{'pane_id':managed,'source':'visible','format':'text','strip_ansi':True})
 assert 'CHRONICLE / LAST HOUR' in read['read']['text'] and 'pane metadata observation' in read['read']['text']
 (CASE/'chapter-pane-read.json').write_text(json.dumps(read,indent=2)+'\n')
 # Closing/reopening uses the latest binary with the same owned state directory.
 before_restart=history_path.read_bytes()
 action('close')
 wait_for(lambda:not any(p['pane_id']==managed for p in snapshot()['panes']),'first managed pane closed')
 managed=None
 plain={p['pane_id'] for p in snapshot()['panes']}
 action('open')
 new=wait_for(lambda:[p for p in snapshot()['panes'] if p['pane_id'] not in plain],'restarted managed pane')
 assert len(new)==1;managed=new[0]['pane_id'];receipt['restarted_managed_pane']=managed
 wait_for(lambda:condition(changed['pane_id'],'Concentrating'),'restored baseline')
 time.sleep(.5)
 assert history_path.read_bytes()==before_restart,'restart backfilled history'
 receipt['restart_history_unchanged']=True
 state=json.loads((history_path.parent/'state.json').read_text());assert state['experience']==0
 receipt['persisted_experience']=state['experience']
 (CASE/'captured-chronicle.jsonl').write_bytes(before_restart)
 receipt['runtime_checks']='passed'
 # Action records alone establish invocation, not room appearance.
 receipt['plugin_logs']=cli('plugin','log','list','--plugin',PLUGIN,'--limit','30')
 (CASE/'pre-cleanup-snapshot.json').write_text(json.dumps(snapshot(),indent=2)+'\n')
except Exception as error:
 receipt['failure']=repr(error)
 try:(CASE/'failure-snapshot.json').write_text(json.dumps(snapshot(),indent=2)+'\n')
 except Exception:pass
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
    if not any(p['pane_id']==owned['pane_id'] for p in snapshot()['panes']):
     owned['already_exited_and_closed']=True
     continue
    owned['release_acknowledgement']=request('pane.release_agent',{'pane_id':owned['pane_id'],'source':owned['source'],'agent':owned['agent'],'seq':100})
    owned['after_release']=request('pane.get',{'pane_id':owned['pane_id']})['pane']
    request('pane.close',{'pane_id':owned['pane_id']})
    wait_for(lambda:not any(p['pane_id']==owned['pane_id'] for p in snapshot()['panes']),'owned pane closed')
    owned['pane_closed']=True
   except Exception as error:receipt['cleanup_errors'].append(repr(error))
  if linked:
   try:cli('plugin','unlink',PLUGIN);receipt['test_link_removed']=True
   except Exception as error:receipt['cleanup_errors'].append(repr(error))
  try:
   receipt['final_snapshot']=snapshot()
   assert not receipt['final_snapshot']['panes']
   receipt['final_plugins']=cli('plugin','list','--json')
   receipt['plugin_logs_after_cleanup']=cli('plugin','log','list','--plugin',PLUGIN,'--limit','30')
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
