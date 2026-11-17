"""Headless protocol/runtime evidence, not Ghostty visual acceptance."""
from pathlib import Path
import os, pty, termios, fcntl, struct, subprocess, select, time, signal, re, json, hashlib, unicodedata

ROOT=Path(__file__).resolve().parent
REPO=ROOT.parents[2]
OUT=Path('/private/tmp/questmancer-storybook-fit-20261117')
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

def run(graphics):
    master,slave=pty.openpty()
    initial=termios.tcgetattr(slave)
    dimensions=[120,34]
    def resize(cols,rows):
        dimensions[:]=[cols,rows]
        fcntl.ioctl(slave,termios.TIOCSWINSZ,struct.pack('HHHH',rows,cols,cols*8,rows*16))
    resize(120,34)
    env={k:v for k,v in os.environ.items() if not k.startswith(('HERDR_','KITTY_','TERM_PROGRAM','TMUX'))}
    env['TERM']='xterm-256color'
    args=[str(REPO/'target/debug/questmancer-storybook'),'--story','world.guild-fit','--inspect']
    if not graphics:args+=['--no-graphics']
    process=subprocess.Popen(args,stdin=slave,stdout=slave,stderr=slave,env=env,close_fds=True,start_new_session=True)
    captured=bytearray();carry=b'';packets=[];queries=[]
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
            if len(captured)>64*1024*1024:raise AssertionError('Unbounded PTY output')
            for request,response in [(b'\x1b[6n',b'\x1b[1;1R'),
                (b'\x1b[16t',b'\x1b[6;16;8t'),(b'\x1b[14t',b'\x1b[4;544;960t')]:
                if request in carry:
                    os.write(master,response);queries.append(request.decode());carry=carry.replace(request,b'')
            # Only complete Kitty packets are consumed; their payloads may span reads.
            while True:
                match=re.search(rb'\x1b_G([^;]*);([^\x1b]*)\x1b\\',carry)
                if not match:break
                controls=match.group(1)
                if b'a=q' in controls:
                    identity=re.search(rb'(?:^|,)i=(\d+)',controls)
                    if identity:os.write(master,b'\x1b_Gi='+identity.group(1)+b';OK\x1b\\')
                packets.append(controls.decode())
                carry=carry[:match.start()]+carry[match.end():]
            # Keep the final status report after all capability replies.
            if b'\x1b[c' in carry:
                os.write(master,b'\x1b[?1;2c');queries.append('primary DA after graphics');carry=carry.replace(b'\x1b[c',b'')
            if b'\x1b[5n' in carry:
                os.write(master,b'\x1b[0n');queries.append('final status after graphics and geometry');carry=carry.replace(b'\x1b[5n',b'')
            carry=carry[-20000:]
    def wait(predicate,description,seconds=10):
        until=time.monotonic()+seconds
        while time.monotonic()<until:
            pump()
            if predicate():return
            if process.poll() is not None:break
        raise AssertionError(description+'; tail='+bytes(captured[-1500:]).decode(errors='replace'))
    def uploads():return [x for x in packets if 'a=T' in x]
    def text():return visible(bytes(captured),*dimensions)
    def press(value):os.write(master,value);pump(.3)
    try:
        wait(lambda:'Kitty fit' in text() if graphics else 'Fit fallback' in text(),'startup')
        if graphics:
            wait(lambda:len(uploads())>=1,'native upload')
            first=len(uploads());pump(.35);assert len(uploads())==first,'static view uploaded without an input'
            press(b'\t');wait(lambda:len(uploads())>first,'selection upload')
            before=len(uploads());resize(96,28);os.kill(process.pid,signal.SIGWINCH)
            wait(lambda:len(uploads())>before,'resize upload')
            # At this geometry the cell centre maps to the Librarian at world (15,77).
            press(b'\x1b[<0;16;22M');wait(lambda:"LIBRARIAN'S LEDGER" in text(),'mouse opens Ledger')
            press(b'j');wait(lambda:'Page 2 / 5' in text(),'Ledger page navigation')
            press(b'\x1b')
            press(b'c');wait(lambda:'ADVENTURER' in text(),'card')
            press(b'\x1b');press(b'L');wait(lambda:"LIBRARIAN'S LEDGER" in text(),'Ledger')
            press(b'\x1b');press(b'?');wait(lambda:'STORYBOOK HELP' in text(),'help')
            press(b'\x1b');press(b'j');pump(.3);press(b'j');wait(lambda:'5/11' in text(),'crowded fallback')
        else:
            assert not uploads(),'graphics-off run emitted images'
        press(b'q');wait(lambda:process.poll() is not None,'clean exit')
        assert process.returncode==0,process.returncode
        restored=termios.tcgetattr(slave)
        assert (restored[3] & (termios.ICANON|termios.ECHO))==(initial[3] & (termios.ICANON|termios.ECHO))
        assert b'\x1b[?1049l' in captured,'alternate screen not restored'
        assert b'\x1b[?1000l' in captured,'mouse capture not restored'
        deletes=[x for x in packets if 'a=d' in x]
        assert all('d=I' in x and re.search(r'(?:^|,)i=\d+',x) for x in deletes)
        assert not any('d=A' in x or 'd=a,' in x for x in deletes)
        image_ids={re.search(r'(?:^|,)i=(\d+)',x).group(1) for x in uploads()}
        deleted_ids={re.search(r'(?:^|,)i=(\d+)',x).group(1) for x in deletes}
        assert deleted_ids<=image_ids and len(deleted_ids)<=2
        if graphics:assert len(image_ids-deleted_ids)==2,'card and Ledger image IDs must remain outside preview cleanup'
        name='native' if graphics else 'graphics-off'
        (OUT/(name+'.ansi')).write_bytes(captured)
        return dict(mode=name,exit=process.returncode,queries=queries,uploads=uploads(),deletions=deletes,
                    output_bytes=len(captured),transcript_sha256=hashlib.sha256(captured).hexdigest(),
                    raw_mode_restored=True,alternate_screen_restored=True,mouse_capture_restored=True,
                    evidence='Synthetic PTY with explicit geometry/graphics replies; not a real terminal')
    finally:
        if process.poll() is None:
            process.terminate()
            try:process.wait(timeout=3)
            except subprocess.TimeoutExpired:process.kill();process.wait()
        (OUT/'last-attempt.ansi').write_bytes(captured)
        os.close(master);os.close(slave)

results=[run(True),run(False)]
(ROOT/'pty-receipt.json').write_text(json.dumps(results,indent=2)+'\n')
print(json.dumps([{'mode':r['mode'],'exit':r['exit'],'uploads':len(r['uploads']),'deletions':len(r['deletions'])} for r in results]))
