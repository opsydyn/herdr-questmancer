"""Headless protocol/runtime evidence, not Ghostty visual acceptance."""
from pathlib import Path
import os, pty, termios, fcntl, struct, subprocess, select, time, signal, re, json, hashlib, unicodedata

ROOT=Path(__file__).resolve().parent
REPO=ROOT.parents[2]
OUT=Path('/private/tmp/questmancer-auto-density-release-20261117')
OUT.mkdir(exist_ok=True)

def visible(raw,cols,rows):
    # Reconstruct cursor-addressed cells; stripping ANSI would join unrelated text.
    source=raw.decode(errors='replace');grid=[[' ']*cols for _ in range(rows)]
    x=y=0;saved=(0,0);i=0
    token=re.compile(r'\x1b(?:_G.*?\x1b\\|\[[0-?]*[ -/]*[@-~]|[78])',re.S)
    while i<len(source):
        match=token.match(source,i)
        if match:
            t=match.group();i=match.end()
            if t.startswith('\x1b_G'):continue
            code=t[-1];values=t[2:-1]
            nums=[int(v) if v.isdigit() else 0 for v in values.split(';')]
            n=nums[0] or 1
            if code in ('H','f'):y=max(0,(nums[0] or 1)-1);x=max(0,((nums[1] if len(nums)>1 else 1) or 1)-1)
            elif code=='A':y=max(0,y-n)
            elif code=='B':y+=n
            elif code=='C':x+=n
            elif code=='D':x=max(0,x-n)
            elif code in ('s','7'):saved=(x,y)
            elif code in ('u','8'):x,y=saved
            elif code=='J' and nums[0]==2:grid=[[' ']*cols for _ in range(rows)]
            elif code=='K' and 0<=y<rows:
                for k in range(min(x,cols),cols):grid[y][k]=' '
            continue
        c=source[i];i+=1
        if c=='\r':x=0
        elif c=='\n':y+=1
        elif ord(c)>=32 and not unicodedata.combining(c):
            if 0<=x<cols and 0<=y<rows:grid[y][x]=c
            x+=2 if unicodedata.east_asian_width(c) in ('F','W') else 1
    return '\n'.join(''.join(row) for row in grid)

def run(mode, interrupt=False, storybook=False):
    import tempfile
    master,slave=pty.openpty()
    initial=termios.tcgetattr(slave)
    dimensions=[120,34]
    def resize(cols,rows):
        dimensions[:]=[cols,rows]
        pixels=(cols*8,rows*16) if mode!='missing-geometry' else (0,0)
        fcntl.ioctl(slave,termios.TIOCSWINSZ,struct.pack('HHHH',rows,cols,*pixels))
    resize(120,34)
    state=Path(tempfile.mkdtemp(prefix='qm-hall-fit-',dir=OUT))
    env={k:v for k,v in os.environ.items() if not k.startswith(('HERDR_','KITTY_','TERM_PROGRAM','TMUX','LC_TERMINAL','ITERM'))}
    env.update(TERM='xterm-256color',HERDR_PLUGIN_STATE_DIR=str(state),HERDR_PLUGIN_CONFIG_DIR=str(state/'config'))
    (state/'config').mkdir()
    binary=REPO/('target/debug/questmancer-storybook' if storybook else 'target/release/questmancer')
    args=[str(binary),'--story','world.delve-fit-resting','--inspect'] if storybook else [str(binary),'ui','--view','delve']
    process=subprocess.Popen(args,stdin=slave,stdout=slave,stderr=slave,env=env,close_fds=True,start_new_session=True)
    captured=bytearray();carry=b'';packets=[];queries=[];checks=[]
    def pump(duration=.08):
        nonlocal carry
        until=time.monotonic()+duration
        while time.monotonic()<until:
            ready,_,_=select.select([master],[],[],max(0,until-time.monotonic()))
            if not ready:break
            try:data=os.read(master,65536)
            except OSError:break
            if not data:break
            captured.extend(data);carry+=data
            if len(captured)>64*1024*1024:raise AssertionError('Unbounded output')
            responses=[(b'\x1b[6n',b'\x1b[1;1R')]
            if mode!='missing-geometry':responses += [(b'\x1b[16t',b'\x1b[6;16;8t'),(b'\x1b[14t',b'\x1b[4;544;960t')]
            for request,response in responses:
                if request in carry:
                    os.write(master,response);queries.append(request.decode());carry=carry.replace(request,b'')
            while True:
                match=re.search(rb'\x1b_G([^;]*);([^\x1b]*)\x1b\\',carry)
                if not match:break
                controls=match.group(1)
                if b'a=q' in controls and mode!='unsupported':
                    identity=re.search(rb'(?:^|,)i=(\d+)',controls)
                    if identity:os.write(master,b'\x1b_Gi='+identity.group(1)+b';OK\x1b\\')
                packets.append(controls.decode());carry=carry[:match.start()]+carry[match.end():]
            if b'\x1b[c' in carry:
                os.write(master,b'\x1b[?1;2c');carry=carry.replace(b'\x1b[c',b'')
            if b'\x1b[5n' in carry:
                os.write(master,b'\x1b[0n');carry=carry.replace(b'\x1b[5n',b'')
            carry=carry[-20000:]
    def wait(predicate,description,seconds=10):
        until=time.monotonic()+seconds
        while time.monotonic()<until:
            pump()
            if predicate():checks.append(description);return
            if process.poll() is not None:break
        raise AssertionError(description+'; tail='+bytes(captured[-1500:]).decode(errors='replace'))
    def uploads():return [p for p in packets if 'a=T' in p]
    def text():return visible(bytes(captured),*dimensions)
    def press(keys):os.write(master,keys);pump(.3)
    try:
        if mode=='native':
            wait(lambda:len(uploads())==1,'negotiated native dungeon upload')
            pump(.5);assert len(uploads())==1,'static dungeon reuploaded without a change'
            checks.append('static upload suppression')
            resize(96,28);process.send_signal(signal.SIGWINCH)
            wait(lambda:len(uploads())==2,'resize upload')
            if storybook:
                press(b'\x1b[<0;29;14M')
                wait(lambda:'ADVENTURER' in text(),'transformed dungeon actor mouse target')
                press(b'\x1b')
                press(b'L')
            else:press(b'?')
            wait(lambda:"LIBRARIAN'S LEDGER" in text(),'Ledger overlay access')
            press(b'j');wait(lambda:'Page 2 /' in text(),'Ledger page navigation')
            press(b'\x1b');wait(lambda:len(uploads())>=(6 if storybook else 4),'fit resumes after Ledger')
            if not storybook:
                before=len(uploads());press(b'1')
                wait(lambda:len(uploads())>before,'Hall fits after Delve')
                before=len(uploads());press(b'2')
                wait(lambda:len(uploads())>before,'Delve fits after Hall')
        else:
            wait(lambda:'▀' in text(),'half-block fallback')
            pump(.4);assert not uploads(),'fallback emitted native image'
            press(b'?');wait(lambda:"LIBRARIAN'S LEDGER" in text(),'fallback Ledger access')
            press(b'\x1b')
        if interrupt:process.send_signal(signal.SIGINT)
        else:press(b'q')
        wait(lambda:process.poll() is not None,'clean shutdown')
        assert process.returncode==0,process.returncode
        restored=termios.tcgetattr(slave)
        assert restored[3]&(termios.ICANON|termios.ECHO)==initial[3]&(termios.ICANON|termios.ECHO)
        assert b'\x1b[?1049l' in captured and b'\x1b[?1000l' in captured
        deletes=[p for p in packets if 'a=d' in p]
        assert all('d=I' in p and re.search(r'(?:^|,)i=\d+',p) for p in deletes)
        image_ids={re.search(r'(?:^|,)i=(\d+)',p).group(1) for p in uploads()}
        deleted_ids={re.search(r'(?:^|,)i=(\d+)',p).group(1) for p in deletes}
        assert deleted_ids<=image_ids and len(deleted_ids)<=2
        if mode=='native':assert len(image_ids-deleted_ids)==(2 if storybook else 1),'card/Ledger identities must remain outside world cleanup'
        name=mode+('-storybook' if storybook else '')+('-interrupt' if interrupt else '')
        (OUT/(name+'.ansi')).write_bytes(captured)
        return dict(mode=name,exit=process.returncode,checks=checks,uploads=uploads(),deletions=deletes,
            binary_sha256=hashlib.sha256(binary.read_bytes()).hexdigest(),output_bytes=len(captured),
            transcript_sha256=hashlib.sha256(captured).hexdigest(),state_directory=str(state),
            raw_mode_restored=True,alternate_screen_restored=True,mouse_capture_restored=True,
            evidence='Synthetic isolated PTY producer evidence; not native display acceptance')
    finally:
        if process.poll() is None:
            process.terminate()
            try:process.wait(timeout=3)
            except subprocess.TimeoutExpired:process.kill();process.wait()
        (OUT/'last-attempt.ansi').write_bytes(captured)
        os.close(master);os.close(slave)

if __name__=='__main__':
    results=[run('native'),run('unsupported'),run('missing-geometry'),run('native',True)]
    (ROOT/'pty-release.json').write_text(json.dumps(results,indent=2)+'\n')
    print(json.dumps([dict(mode=r['mode'],uploads=len(r['uploads']),deletions=len(r['deletions']),exit=r['exit']) for r in results]))
