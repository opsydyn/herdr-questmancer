"""Profile/world row-diff producer checks. No Herdr or GUI automation."""
from pathlib import Path
import os, pty, termios, fcntl, struct, subprocess, select, time, signal, re, json, hashlib, unicodedata, tempfile, shutil
ROOT=Path(__file__).resolve().parent;REPO=ROOT.parents[3]
OUT=Path('/private/tmp/questmancer-capacity-study-20261117');OUT.mkdir(exist_ok=True)
# Decode the pinned renderer's official row/column alphabet, including high image-ID bits.
source=next((Path.home()/'.cargo/registry/src').glob('*/ratatui-image-11.0.6/src/protocol/kitty.rs')).read_text()
source=source[source.index('static DIACRITICS:'):]
DIACRITICS=[chr(int(value,16)) for value in re.findall(r"'\\u\{([0-9A-Fa-f]+)\}'",source)]
assert len(DIACRITICS)==297
INDEX={c:i for i,c in enumerate(DIACRITICS)}

def screen(raw,cols,rows):
    source=raw.decode(errors='replace');grid={};x=y=0;fg=None;saved=(0,0,None);i=0
    token=re.compile(r'\x1b(?:_G.*?\x1b\\|\[[0-?]*[ -/]*[@-~]|[78])',re.S)
    while i<len(source):
        match=token.match(source,i)
        if match:
            seq=match.group();i=match.end()
            if seq.startswith('\x1b_G'):continue
            code=seq[-1];values=seq[2:-1];nums=[int(v) if v.isdigit() else 0 for v in values.split(';')];n=nums[0] or 1
            if code in ('H','f'):y=(nums[0] or 1)-1;x=((nums[1] if len(nums)>1 else 1) or 1)-1
            elif code=='A':y=max(0,y-n)
            elif code=='B':y+=n
            elif code=='C':x+=n
            elif code=='D':x=max(0,x-n)
            elif code in ('s','7'):saved=(x,y,fg)
            elif code in ('u','8'):x,y,fg=saved
            elif code=='J' and nums[0]==2:grid={}
            elif code=='K':
                for xx in range(max(0,x),cols):grid.pop((xx,y),None)
            elif code=='m':
                k=0
                while k<len(nums):
                    if nums[k] in (0,39):fg=None
                    elif nums[k]==38 and k+4<len(nums) and nums[k+1]==2:
                        fg=(nums[k+2]<<16)|(nums[k+3]<<8)|nums[k+4];k+=4
                    k+=1
            continue
        char=source[i];i+=1
        if char=='\r':x=0;continue
        if char=='\n':y+=1;continue
        if ord(char)<32 or unicodedata.combining(char):continue
        marks=[]
        while i<len(source) and unicodedata.combining(source[i]):marks.append(source[i]);i+=1
        item={'char':char}
        if char=='\U0010eeee':
            if len(marks)>=3:row,col,high=[INDEX[m] for m in marks[:3]]
            else:
                previous=grid.get((x-1,y));assert previous and 'image' in previous,('placeholder lost row/column origin',x,y,fg,previous)
                row,col,high=previous['row'],previous['col']+1,previous['image']>>24
            assert fg is not None
            item.update(image=(high<<24)|fg,row=row,col=col)
        grid[(x,y)]=item
        x+=2 if unicodedata.east_asian_width(char) in ('F','W') else 1
    return {position:value for position,value in grid.items() if 0<=position[0]<cols and 0<=position[1]<rows}

def geometry(cols,rows):
    scale=min(cols*8//160,(rows-3)*16//90,16);assert scale>0
    width=(160*scale+7)//8;height=(90*scale+15)//16
    world=((cols-width)//2,1+(rows-3-height)//2,width,height)
    detailed=cols>=96 and rows>=20;card_width=min(cols-4,78 if detailed else 48)
    card=(cols-card_width-1,2,card_width+1,min(18 if detailed else 13,rows-2))
    return world,card

def run(world,initial_profile,size):
    master,slave=pty.openpty();initial=termios.tcgetattr(slave);dimensions=list(size)
    def resize(cols,rows):
        dimensions[:]=[cols,rows]
        fcntl.ioctl(slave,termios.TIOCSWINSZ,struct.pack('HHHH',rows,cols,cols*8,rows*16))
    resize(*size)
    state=Path(tempfile.mkdtemp(prefix='state-',dir=OUT));(state/'config').mkdir()
    env={k:v for k,v in os.environ.items() if not k.startswith(('HERDR_','KITTY_','TERM_PROGRAM','TMUX','LC_TERMINAL','ITERM'))}
    env.update(TERM='xterm-256color',HERDR_PLUGIN_STATE_DIR=str(state),HERDR_PLUGIN_CONFIG_DIR=str(state/'config'))
    binary=OUT/'questmancer-storybook-pinned';story='world.capacity-'+('hall' if world=='guild' else 'delve')+'-32-small'
    process=subprocess.Popen([str(binary),'--story',story,'--inspect'],stdin=slave,stdout=slave,stderr=slave,env=env,close_fds=True,start_new_session=True)
    captured=bytearray();carry=b'';packets=[];checks=[]
    def pump(duration=.07):
        nonlocal carry
        until=time.monotonic()+duration
        while time.monotonic()<until:
            ready,_,_=select.select([master],[],[],max(0,until-time.monotonic()))
            if not ready:break
            try:data=os.read(master,65536)
            except OSError:break
            if not data:break
            captured.extend(data);carry+=data;assert len(captured)<64*1024*1024
            for query,response in [(b'\x1b[6n',b'\x1b[1;1R'),(b'\x1b[16t',b'\x1b[6;16;8t')]:
                if query in carry:os.write(master,response);carry=carry.replace(query,b'')
            while True:
                match=re.search(rb'\x1b_G([^;]*);([^\x1b]*)\x1b\\',carry)
                if not match:break
                controls=dict(p.split('=',1) for p in match[1].decode().split(',') if '=' in p)
                if controls.get('a')=='q':os.write(master,b'\x1b_Gi='+controls['i'].encode()+b';OK\x1b\\')
                packets.append(controls);carry=carry[:match.start()]+carry[match.end():]
            if b'\x1b[c' in carry:os.write(master,b'\x1b[?1;2c');carry=carry.replace(b'\x1b[c',b'')
            if b'\x1b[5n' in carry:os.write(master,b'\x1b[0n');carry=carry.replace(b'\x1b[5n',b'')
            carry=carry[-20000:]
    def grid():return screen(bytes(captured),*dimensions)
    def text():
        g=grid();return '\n'.join(''.join(g.get((x,y),{}).get('char',' ') for x in range(dimensions[0])) for y in range(dimensions[1]))
    def uploads():return [p for p in packets if p.get('a')=='T']
    def world_uploads():return [p for p in uploads() if (int(p.get('s',0)),int(p.get('v',0))) in {(800,464),(640,368),(480,272),(160,96)}]
    def wait(predicate,description):
        until=time.monotonic()+12
        while time.monotonic()<until:
            pump()
            if predicate():checks.append(description);return
            assert process.poll() is None,description+': process exited'
        raise AssertionError(description+'; tail='+text()[-1200:])
    def press(keys):os.write(master,keys);pump(.15)
    def settle():
        until=time.monotonic()+5
        while time.monotonic()<until:
            before=len(captured);pump(.1)
            if len(captured)==before:return
        raise AssertionError('fixed Storybook output did not settle')
    def check_layout(profile):
        settle()
        w,h=dimensions;area,card=geometry(w,h);wx,wy,ww,wh=area;cx,cy,cw,ch=card
        expected={(x,y):(y-wy,x-wx) for y in range(wy,wy+wh) for x in range(wx,wx+ww)
            if not(profile and cx<=x<cx+cw and cy<=y<cy+ch)}
        ids={int(p['i']) for p in world_uploads()}
        actual={position:(v['row'],v['col']) for position,v in grid().items() if v.get('image') in ids}
        assert actual==expected,dict(world=world,size=dimensions,profile=profile,expected=len(expected),actual=len(actual),
            missing=list(expected.keys()-actual.keys())[:5],extra=list(actual.keys()-expected.keys())[:5])
        portrait_ids={int(p['i']) for p in uploads()}-ids
        portrait={position for position,value in grid().items() if value.get('image') in portrait_ids}
        detailed=w>=96 and h>=20
        expected_portrait={(x,y) for y in range(3,19) for x in range(cx+2,cx+26)} if profile and detailed else set()
        assert portrait==expected_portrait,('native portrait footprint changed',len(portrait),len(expected_portrait))
        if profile:assert 'ADVENTURER' in text()
        else:assert 'ADVENTURER' not in text(),'profile text remained after closing'
        checks.append('exact visible placeholder cells and source row/column coordinates; profile='+str(profile))
    try:
        wait(lambda:'ADVENTURER' in text() if initial_profile else len(world_uploads())==1,'startup')
        check_layout(initial_profile)
        if initial_profile:press(b'\x1b');wait(lambda:len(world_uploads())==1,'first/full exposure');check_layout(False)
        initial_count=len(world_uploads());assert initial_count==1
        for _ in range(2):
            press(b'c');wait(lambda:'ADVENTURER' in text(),'open profile');check_layout(True)
            assert len(world_uploads())==initial_count,'profile open reencoded world'
            # Pure local preview: a click on the profile must not select an actor behind it.
            _,card=geometry(*dimensions);cx,cy,_,_=card
            press(('\x1b[<0;'+str(cx+2)+';'+str(cy+3)+'M').encode());check_layout(True)
            press(b'\x1b');check_layout(False)
            assert len(world_uploads())==initial_count,'profile close reencoded world'
        press(b'c');resize(96,28);process.send_signal(signal.SIGWINCH)
        wait(lambda:len(world_uploads())==(1 if size==(96,28) else 2),'resize with profile')
        check_layout(True);press(b'\x1b');check_layout(False)
        press(b'q');until=time.monotonic()+5
        while process.poll() is None and time.monotonic()<until:pump()
        assert process.returncode==0
        ids={p['i'] for p in world_uploads()};deletes=[p for p in packets if p.get('a')=='d']
        assert all(p.get('d')=='I' and p.get('i') in ids for p in deletes)
        assert {p['i'] for p in deletes}==ids and len(ids)<=2
        restored=termios.tcgetattr(slave);assert restored[3]&(termios.ICANON|termios.ECHO)==initial[3]&(termios.ICANON|termios.ECHO)
        assert b'\x1b[?1049l' in captured and b'\x1b[?1000l' in captured
        name=world+'-'+('initial-profile' if initial_profile else 'initial-world')+'-'+str(size[0])+'x'+str(size[1])
        (OUT/(name+'.ansi')).write_bytes(captured)
        return dict(scenario=name,checks=checks,world_uploads=world_uploads(),world_deletions=deletes,exit=process.returncode,
            binary_sha256=hashlib.sha256(binary.read_bytes()).hexdigest(),transcript_sha256=hashlib.sha256(captured).hexdigest(),
            output_bytes=len(captured),terminal_restored=True,evidence='Isolated executable ANSI/placeholder producer; native compositing remains human review')
    finally:
        if process.poll() is None:
            process.terminate()
            try:process.wait(timeout=3)
            except subprocess.TimeoutExpired:process.kill();process.wait()
        (OUT/'last-attempt.ansi').write_bytes(captured);os.close(master);os.close(slave)

if __name__=='__main__':
    shutil.copy2(REPO/'target/debug/questmancer-storybook',OUT/'questmancer-storybook-pinned')
    results=[run(world,opened,size) for world in ['guild','delve']
        for opened,size in [(False,(120,34)),(False,(80,24))]]
    (ROOT/'pty-receipt.json').write_text(json.dumps(results,indent=2)+'\n')
    print(json.dumps([dict(scenario=r['scenario'],world_uploads=len(r['world_uploads']),exit=r['exit']) for r in results]))
