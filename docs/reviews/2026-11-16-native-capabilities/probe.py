"""Owned terminal probe; no clipboard reads or application state changes."""
import os,sys,tty,termios,select,time,base64,json,pathlib,re,datetime
ROOT=pathlib.Path(__file__).resolve().parent
IDS=(611601,611602)
def run(label):
 result={'at':datetime.datetime.now(datetime.UTC).isoformat(),'context':label,'responses':{},'visible_appearance':'requires human observation','clipboard':'not queried'}
 fd=sys.stdin.fileno();saved=termios.tcgetattr(fd)
 def emit(s):os.write(sys.stdout.fileno(),s.encode())
 def collect(seconds):
  end=time.monotonic()+seconds;data=b''
  while time.monotonic()<end:
   if select.select([fd],[],[],min(.05,max(0,end-time.monotonic())))[0]:data+=os.read(fd,4096)
  # Retain protocol replies only, never literal typed input.
  return re.findall(rb'\x1b_G[^\x1b]*\x1b\\|\x1b\[[0-9;?$]*[a-zA-Z~]',data)
 def query(name,seq,seconds=.5):
  emit(seq);replies=collect(seconds);result['responses'][name]=[r.decode('ascii',errors='replace') for r in replies];return b''.join(replies)
 def graphics(control,pixels=b''):emit('\x1b_G'+control+(';' + base64.b64encode(pixels).decode() if pixels else '')+'\x1b\\')
 visibility_enabled=False
 try:
  tty.setraw(fd)
  query('cell_geometry','\x1b[16t')
  mode=query('visibility_mode','\x1b[?2033$p')
  if b'?2033;2$y' in mode:
   visibility_enabled=True;emit('\x1b[?2033h');query('visibility_report','\x1b[?998n')
  basic=query('kitty_basic','\x1b_Ga=q,i=611600,t=d,f=32,s=1,v=1;/////w==\x1b\\')
  query('animation_feature','\x1b_Ga=q,i=611603,e=f,r=1,X=1\x1b\\')
  query('relative_feature','\x1b_Ga=q,i=611604,e=p,P=1,Q=1,H=1,V=1\x1b\\')
  if b';OK' in basic:
   # 16x16 two-colour diagnostics, bounded to three seconds, deleted by exact IDs.
   emit('\r\nBounded image demo: parent changes colour; child should sit to its right.\r\n')
   graphics('a=T,i=611601,p=1,t=d,f=32,s=16,v=16,C=1',bytes([40,190,90,255])*256)
   query('parent_upload','',.2)
   graphics('a=f,i=611601,t=d,f=32,s=16,v=16,z=500',bytes([170,70,210,255])*256)
   query('animation_frame','',.2)
   graphics('a=a,i=611601,r=1,z=500');graphics('a=a,i=611601,s=3,v=3')
   graphics('a=t,i=611602,t=d,f=32,s=8,v=8',bytes([240,190,40,255])*64)
   graphics('a=p,i=611602,p=1,P=611601,Q=1,H=3,V=0,C=1')
   query('demo_responses','',3)
   result['demo']='sent bounded commands; protocol acknowledgements do not prove visible animation or relative placement'
  else:result['demo']='not sent: no positive basic Kitty reply'
 finally:
  if visibility_enabled:emit('\x1b[?2033l')
  for image_id in IDS:graphics(f'a=d,d=I,i={image_id},q=2')
  collect(.15)
  termios.tcsetattr(fd,termios.TCSANOW,saved)
 with (ROOT/'capability-results.jsonl').open('a') as f:f.write(json.dumps(result)+'\n')
 print(json.dumps(result,indent=2),flush=True)
 print('Review replies as acknowledgements only. Missing feature-query fields mean unknown, not supported.',flush=True)
if '--baseline' in sys.argv:run('headless baseline; no Ghostty client attached by the trial')
while True:
 try:input('\nAttach this session from Ghostty, select this probe pane, then Return to run/repeat; Ctrl-C exits: ')
 except (EOFError,KeyboardInterrupt):break
 run('user-triggered; attached terminal presence and appearance require human confirmation')
