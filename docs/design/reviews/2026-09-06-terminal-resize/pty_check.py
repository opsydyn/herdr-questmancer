"""Exercise an owned Storybook process in a PTY; no native-app visual claim."""
from pathlib import Path
import errno, fcntl, hashlib, json, os, re, select, signal, struct, sys, termios, time

binary=Path(sys.argv[1]).resolve()
stories={s['id']:s for s in json.loads(Path(sys.argv[2]).read_text())}
output=Path(sys.argv[3])
targets=['world.guild','world.delve','asset.native-wizard-portrait',
         'asset.native-ranger-portrait','asset.native-barbarian-portrait',
         'interaction.librarian-ledger','asset.librarian',
         'asset.wizard-poses','asset.ranger-poses','asset.barbarian-poses']
assert all(key in stories for key in targets),sorted(stories)
ansi=re.compile(r'\x1b\[[0-?]*[ -/]*[@-~]|\x1b\][^\x07]*(?:\x07|\x1b\\)')
master,slave=os.openpty()
original=termios.tcgetattr(slave)
fcntl.ioctl(slave,termios.TIOCSWINSZ,struct.pack('HHHH',46,161,0,0))
pid=os.fork()
if pid==0:
    try:
        os.close(master);os.setsid();fcntl.ioctl(slave,termios.TIOCSCTTY,0)
        os.tcsetpgrp(slave,os.getpgrp())
        for fd in (0,1,2):os.dup2(slave,fd)
        if slave>2:os.close(slave)
        env=dict(os.environ)
        for key in list(env):
            if key.startswith('HERDR_') or key in {'TERM_PROGRAM','TERM_PROGRAM_VERSION','KITTY_WINDOW_ID','ITERM_SESSION_ID','WEZTERM_PANE','LC_TERMINAL','VTE_VERSION'}:env.pop(key,None)
        env.update(TERM='xterm-256color',COLORTERM='truecolor')
        os.execve(binary,[str(binary)],env)
    except BaseException as error:
        os.write(2,str(error).encode());os._exit(127)

captured=bytearray();exited=False
report={'kind':'isolated PTY executable check; not native terminal visual acceptance',
        'binary_sha256':hashlib.sha256(binary.read_bytes()).hexdigest(),'cases':[]}

def read_frame(timeout=2.0,quiet=0.12):
    data=bytearray();deadline=time.monotonic()+timeout;last=None
    while time.monotonic()<deadline:
        wait=min(0.05,max(0,deadline-time.monotonic()))
        ready,_,_=select.select([master],[],[],wait)
        if ready:
            try:chunk=os.read(master,65536)
            except OSError as error:
                if error.errno==errno.EIO:break
                raise
            if not chunk:break
            data.extend(chunk);captured.extend(chunk);last=time.monotonic()
        elif last is not None and time.monotonic()-last>=quiet:break
    return bytes(data)

def send(keys):
    os.write(master,keys);return read_frame()

def resize(width,height):
    fcntl.ioctl(slave,termios.TIOCSWINSZ,struct.pack('HHHH',height,width,0,0))
    # This signal is sent only to the exact child process created above.
    os.kill(pid,signal.SIGWINCH)
    return read_frame()

try:
    startup=bytearray();deadline=time.monotonic()+8
    while b'QUESTMANCER STORYBOOK' not in startup and time.monotonic()<deadline:
        startup.extend(read_frame(timeout=1))
    assert b'QUESTMANCER STORYBOOK' in startup,'Storybook did not reach its catalogue'
    assert b'authored sprite fallback' in startup,'PTY must select the authored fallback route'
    report['fallback_confirmed']=True
    inspecting=False
    for key in targets:
        story=stories[key]
        resize(161,46)
        if inspecting:send(b'\x1b')
        category={'Worlds':0,'Assets':1,'Interactions':2}[story['category']]
        # Return to the first world before selecting from the live catalogue.
        navigation=b'hhk'+b'l'*category+b'j'*story['within_category']
        send(navigation)
        entered=send(b'\r');inspecting=True
        assert entered,'Entering inspection must emit a frame'
        for width,height in [(160,45),(100,30),(64,20),(30,15),(24,13),(16,9),(160,45)]:
            frame=resize(width,height)
            assert frame,f'{key} {width}x{height}: resize produced no output'
            text=ansi.sub('',frame.decode('utf-8',errors='replace'))
            too_small=bool(re.search(r'Needs\s*\d+x\d+',text))
            expected_small=not story['scene'] and (width<story['minimum'][0] or height<story['minimum'][1])
            assert too_small==expected_small,(key,width,height,text[:150])
            positions=[(int(row),int(col)) for row,col in re.findall(rb'\x1b\[(\d+);(\d+)H',frame)]
            assert all(1<=row<=height and 1<=col<=width for row,col in positions),(key,width,height,positions)
            report['cases'].append({'story':key,'columns':width,'rows':height,
                'bytes':len(frame),'minimum_message':too_small,'cursor_positions_in_bounds':True})
    tail=send(b'q')
    deadline=time.monotonic()+3
    while time.monotonic()<deadline:
        found,status=os.waitpid(pid,os.WNOHANG)
        if found:
            exited=True;report['exit_code']=os.waitstatus_to_exitcode(status);break
        tail+=read_frame(timeout=0.1)
    assert exited and report['exit_code']==0,'Storybook did not exit cleanly'
    report['terminal_modes_restored']=termios.tcgetattr(master)==original
    report['alternate_screen_left']=b'\x1b[?1049l' in captured
    report['cursor_restored']=b'\x1b[?25h' in captured
    assert all(report[k] for k in ['terminal_modes_restored','alternate_screen_left','cursor_restored'])
    report['result']='passed'
except BaseException as error:
    report['result']='failed';report['error']=str(error)
    raise
finally:
    if not exited:
        try:os.kill(pid,signal.SIGINT)
        except ProcessLookupError:pass
        deadline=time.monotonic()+2
        while time.monotonic()<deadline:
            found,_=os.waitpid(pid,os.WNOHANG)
            if found:exited=True;break
            read_frame(timeout=0.1)
        if not exited:
            os.kill(pid,signal.SIGKILL);os.waitpid(pid,0)
    os.close(master);os.close(slave)
    output.write_text(json.dumps(report,indent=2)+'\n')
print(f"Passed {len(report['cases'])} PTY resize cases; clean exit and terminal restoration verified.")
