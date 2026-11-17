import os,pty,tty,subprocess,fcntl,termios,struct,select,time,json,tempfile,pathlib,re
root=pathlib.Path.cwd(); receipt={'scope':'automated macOS PTY input and restart; no native visual approval or Herdr server','steps':[]}
with tempfile.TemporaryDirectory(prefix='questmancer-l1b-pty-',dir=os.path.realpath('/tmp')) as temporary:
 state=pathlib.Path(temporary); env=dict(os.environ)
 for key in list(env):
  if key.startswith('HERDR_'):env.pop(key)
 env.update(HERDR_PLUGIN_STATE_DIR=str(state),TERM='xterm-256color')
 def start():
  master,slave=pty.openpty();fcntl.ioctl(slave,termios.TIOCSWINSZ,struct.pack('HHHH',36,120,720,960))
  process=subprocess.Popen([str(root/'target/release/questmancer'),'ui'],stdin=slave,stdout=slave,stderr=slave,env=env,preexec_fn=lambda: (os.setsid(),fcntl.ioctl(slave,termios.TIOCSCTTY,0)));os.close(slave)
  return master,process,bytearray(),set()
 def read(ms=150):
  deadline=time.monotonic()+ms/1000
  while time.monotonic()<deadline:
   if select.select([master],[],[],min(.02,max(0,deadline-time.monotonic())))[0]:
    try:data=os.read(master,65536)
    except OSError:return
    if not data:return
    output.extend(data)
    probes=[(b'\x1b_Gi=31',b'\x1b_Gi=31;ENOTSUP\x1b\\'),(b'\x1b[16t',b'\x1b[6;20;10t'),(b'\x1b[6n',b'\x1b[1;1R'),(b'\x1b[c',b'\x1b[?62;22c'),(b'\x1b[5n',b'\x1b[0n')]
    for query,reply in probes:
     if query in output and query not in replied:
      os.write(master,reply);replied.add(query)
 def wait_ready():
  deadline=time.monotonic()+15
  while time.monotonic()<deadline:
   read(100)
   if b'offline: launch from Herdr' in output:return
   if process.poll() is not None:raise RuntimeError('UI exited before first draw')
  raise RuntimeError('No first draw')
 def await_text(text):
  deadline=time.monotonic()+5
  while time.monotonic()<deadline:
   read(50)
   clean=re.sub(r'\x1b\[[0-?]*[ -/]*[@-~]','',output.decode(errors='replace')).replace('▀','')
   if text in clean:return
  pathlib.Path('/tmp/l1b-pty-raw.txt').write_bytes(output)
  raise RuntimeError('Input did not reach '+text+'; visible output='+clean[-1400:])
 def screen_text():
  # Ratatui positions changed cells absolutely. Reconstruct this bounded grid;
  # stripping CSI alone incorrectly joins text from separate cursor writes.
  grid=[[' ']*120 for _ in range(36)];x=y=0;data=output.decode(errors='replace');i=0
  while i<len(data):
   if data[i]=='\x1b':
    match=re.match(r'\x1b\[([0-?]*)([ -/]*)([@-~])',data[i:])
    if match:
     params=match.group(1);kind=match.group(3);values=[int(v) if v.isdigit() else 0 for v in params.split(';')];a=values[0] or 1
     if kind in ('H','f'):y=(values[0] or 1)-1;x=(values[1] or 1)-1 if len(values)>1 else 0
     elif kind=='G':x=a-1
     elif kind=='d':y=a-1
     elif kind=='A':y=max(0,y-a)
     elif kind=='B':y=min(35,y+a)
     elif kind=='C':x=min(119,x+a)
     elif kind=='D':x=max(0,x-a)
     elif kind=='J' and values[0]==2:grid=[[' ']*120 for _ in range(36)]
     elif kind=='K' and 0<=y<36:
      for col in range(x,120):grid[y][col]=' '
     i+=match.end();continue
    if i+1<len(data) and data[i+1] in '_]P':
     end=data.find('\x1b\\',i+2);i=len(data) if end<0 else end+2;continue
    i+=2;continue
   ch=data[i]
   if ch=='\r':x=0
   elif ch=='\n':y=min(35,y+1)
   elif ord(ch)>=32:
    if 0<=y<36 and 0<=x<120:grid[y][x]=ch
    x+=1
   i+=1
  return '\n'.join(''.join(row) for row in grid)
 def send(data,ms=350):os.write(master,data if isinstance(data,bytes) else data.encode());read(ms)
 def paste(text):send(b'\x1b[200~'+text.encode()+b'\x1b[201~')
 def down(count):
  for _ in range(count):send(b'\x1b[B',150)
 def wait_revision(revision):
  deadline=time.monotonic()+6
  while time.monotonic()<deadline:
   read(50)
   if process.poll() is not None:raise RuntimeError('UI exited during step '+str(revision))
   try:
    work=json.loads((state/'owned-work.json').read_text())['state']; cache=json.loads((state/'work-state.json').read_text())
    if work['revision']==revision and cache['projection']['sources'].get(work['source'],{}).get('cursor')==revision:read(150);receipt['steps'].append({'revision':revision,'acknowledged':True});return work
   except (FileNotFoundError,json.JSONDecodeError,KeyError):pass
  pathlib.Path('/tmp/l1b-pty-raw.txt').write_bytes(output)
  raise RuntimeError('revision did not arrive: '+str(revision)+'; tail='+re.sub(r'\x1b\[[0-?]*[ -/]*[@-~]','',output.decode(errors='replace'))[-1500:])
 master,process,output,replied=start()
 try:
  wait_ready();send('b');await_text('QUEST BOARD');send('a');await_text('OWNED WORK');down(7);send('\r');send('\t\r');wait_revision(0)
  send('\r');paste('PTY authentication campaign');send('\t');paste('Preserve clients during migration');send('\t\r');wait_revision(1)
  down(1);send('\r');paste('Migrate API authentication');send('\t');down(2);send('\t');paste('Synthetic PTY work');send('\t\r');wait_revision(2)
  down(2);send('\r');paste('Keep the old endpoint?');send('\t');paste('Explicit compatibility policy');send('\t');down(1);send('\t\r');wait_revision(3)
  down(6);send('\r');send('\r',250);send('\r');paste('Keep for one release.\nDocument removal.');send('\t\r');work=wait_revision(4)
  read(250);send(b'\x1b',180);down(4);send('\r');paste('Migration report');send('\t');down(4);send('\t');paste('https://example.invalid/report');send('\t');down(1);send('\t\r');work=wait_revision(5)
  assert len(work['inventory']['campaigns'])==1 and len(work['inventory']['quests'])==1 and len(work['inventory']['artifacts'])==1
  assert list(work['decisions'].values())[0]['answer']['text']=='Keep for one release.\nDocument removal.'
  assert all(o['state']=='Resolved' for o in work['inventory']['obligations'].values())
  for _ in range(3):send(b'\x1b',180)
  send('q',250);process.wait(timeout=5);assert process.returncode==0
  first_work=(state/'owned-work.json').read_bytes();os.close(master)
  master,process,output,replied=start();wait_ready();send('b');send('a');down(5);send('\r');send('\r');down(6);send('\r');send('\r',250)
  clean=screen_text()
  assert 'Keep for one release.' in clean,clean
  (root/'docs/reviews/2026-10-03-owned-authoring/restart-pty-screen.txt').write_text(clean+'\n')
  assert (state/'owned-work.json').read_bytes()==first_work
  for _ in range(3):send(b'\x1b',180)
  send('q',250);process.wait(timeout=5);assert process.returncode==0
  receipt.update(restart_answer_visible=True,restart_authority_unchanged=True,owned_children_exited=True,herdr_resources_created=0)
 finally:
  if process.poll() is None:
   process.terminate()
   for _ in range(50):
    read(100)
    if process.poll() is not None:break
   if process.poll() is None:process.kill()
   process.wait(timeout=5)
  os.close(master)
receipt['temporary_resources_removed']=not state.exists()
(root/'docs/reviews/2026-10-03-owned-authoring/pty-receipt.json').write_text(json.dumps(receipt,indent=2)+'\n')
print(json.dumps(receipt))
