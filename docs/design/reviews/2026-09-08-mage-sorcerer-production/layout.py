"""Typeset production-rendered pixels and terminal cells as review documents."""
from pathlib import Path
import json, sys
from PIL import Image, ImageDraw, ImageFont
ROOT = Path(__file__).resolve().parent
DATA = json.loads(Path(sys.argv[1]).read_text())
BG='#151b21'; PANEL='#1d252d'; EDGE='#35414b'; INK='#eee1c3'; MUTED='#adb7b5'; GOLD='#dfb86c'
FONTS={'sans':'/System/Library/Fonts/Avenir Next.ttc','serif':'/System/Library/Fonts/Supplemental/Georgia.ttf','mono':'/System/Library/Fonts/Menlo.ttc'}
def text(d, xy, value, size=18, fill=INK, kind='sans'):
    d.text(xy, value, fill=fill, font=ImageFont.truetype(FONTS[kind], size))
def page(number, title, subtitle, height):
    im=Image.new('RGB',(1440,height),BG); d=ImageDraw.Draw(im)
    text(d,(48,28),'QUESTMANCER  /  CENSER & HALO PRODUCTION  /  '+number,15,GOLD,'mono')
    text(d,(48,62),title,36,INK,'serif'); text(d,(48,112),subtitle,18,MUTED)
    d.line((48,height-55,1392,height-55),fill=EDGE)
    text(d,(48,height-38),'PRODUCTION RGB + RATATUI  ·  VISUAL REVIEW PENDING  ·  TERMINAL / LIVE ACCEPTANCE SEPARATE',13,MUTED,'mono')
    return im,d
def pixels(key):
    data=DATA[key]; im=Image.new('RGBA',(data['width'],data['height']))
    im.putdata([tuple(p)+(255,) if p is not None else (0,0,0,0) for p in data['pixels']]); return im
def put(im,key,x,y,scale=1):
    src=pixels(key); src=src.resize((src.width*scale,src.height*scale),Image.Resampling.NEAREST); im.paste(src,(x,y),src)
def terminal(key):
    data=DATA[key]; cw=6; ch=12; im=Image.new('RGB',(data['columns']*cw,data['rows']*ch)); d=ImageDraw.Draw(im)
    for i,c in enumerate(data['cells']):
        x=(i%data['columns'])*cw; y=(i//data['columns'])*ch
        d.rectangle((x,y,x+cw-1,y+ch-1),fill=tuple(c['bg']))
        if c['text']=='▀': d.rectangle((x,y,x+cw-1,y+ch//2-1),fill=tuple(c['fg']))
        elif c['text'].strip(): text(d,(x,y-1),c['text'],10,tuple(c['fg']),'mono')
    return im

NAMES=['Mage','Sorcerer']
POSES=[('work-a','Work A'),('work-b','Work B'),('counsel','Signal'),('waiting','Waiting'),('spoils','Spoils'),('settled','Completed'),('rest','Resting'),('unknown','Unknown')]
im,d=page('01','Two adventurers, eight authored moments','Production 16 × 24 frames. Stable feet on row 21; persona colours applied by the existing palette system.',1080)
for col,(_,title) in enumerate(POSES): text(d,(194+col*150,166),title,17,GOLD)
for row,name in enumerate(NAMES):
    y=217+row*253; text(d,(48,y+50),name,23,INK,'serif')
    for col,(key,_) in enumerate(POSES):
        x=204+col*150; put(im,f'{row}-{key}',x,y,6)
        d.line((x-5,y+132,x+102,y+132),fill=EDGE)
        put(im,f'{row}-{key}',x+40,y+164);text(d,(x+31,y+196),'1x',12,MUTED)
text(d,(48,752),'Persona variation and retained roster partners',25,GOLD,'serif')
for index in range(12):
    x=48+index*92;put(im,f'variant-{index}',x,810,4)
for index in range(len(NAMES)):
    x=1170+index*80;put(im,f'{index}-roster',x,840,5);text(d,(x,916),NAMES[index][:3],14,MUTED)
text(d,(48,974),'Skull staff / green censer and halo / silver focus remain class-specific. The shared raised signal settles into a calm blocked pose.',18)
im.save(ROOT/'01-production-poses.png')

im,d=page('02','The same party in both rooms','Final Ratatui cells, including nameplates, selection and overlays. Menlo reconstruction; not terminal screenshots.',2930)
for col,world in enumerate(['hall','delve']):
    x=48+col*690;text(d,(x,165),'Guild Hall' if world=='hall' else 'Delve',28,GOLD,'serif')
    for label,y,caption in [('canonical',220,'160 × 90 / mixed party'),('medium',660,'100 × 60'),('compact',1100,'64 × 40'),('roster',1420,'30 × 30')]:
        src=terminal(f'{world}-{label}')
        # 160 columns are shown at 4 px per cell, all others retain 6 px cells.
        if label=='canonical':src=src.resize((640,360),Image.Resampling.NEAREST)
        im.paste(src,(x,y));text(d,(x,y-28),caption,16,MUTED)
    for label,offset in [('vignette',0),('status',270)]:
        y=1700;im.paste(terminal(f'{world}-{label}'),(x+offset,y));text(d,(x+offset,y-28),'24 × 26' if label=='vignette' else '16 × 18',16,MUTED)
    y=1950;text(d,(x,y-28),'30 × 30 / ANSI 16 + ASCII',17,GOLD);im.paste(terminal(f'{world}-roster-ansi'),(x,y))
    y=2250;text(d,(x,y-28),'11 adventurers / complete actors and overflow rules',16,MUTED)
    src=terminal(f'{world}-capacity').resize((640,360),Image.Resampling.NEAREST);im.paste(src,(x,y))
text(d,(48,2670),'Hall: complete actors through compact, roster, selected vignette and status-only tiers.',19)
text(d,(48,2703),'Delve: its own authored crop and capacity contract; small-room rosters only when the whole party fits.',19)
text(d,(48,2750),'The exporter also produces ANSI/ASCII buffers at every size and the 12-adventurer overflow case.',18,MUTED)
text(d,(48,2783),'Full / reduced / still timing and lifecycle interruption are covered by the production integration tests.',18,MUTED)
im.save(ROOT/'02-rooms-and-viewports.png')

im,d=page('03','A brief gesture, then a quiet room','Production room crops: Mage and Sorcerer in separate fixtures, each at its real station.',1930)
def actor_strip(key,scale=4):
    # Show each pilot separately so the Delve's two-per-station limit cannot hide a class.
    out=Image.new('RGB',(40*len(NAMES)*scale,44*scale),BG)
    for index in range(len(NAMES)):
        item=f'solo-{key}-{index}'; xx,yy,_,_=DATA[item]['actor_bounds'][0]
        crop=pixels(item).crop((xx-12,yy-10,xx+28,yy+34))
        out.paste(crop.resize((40*scale,44*scale),Image.Resampling.NEAREST),(index*40*scale,0))
    return out
for col,world in enumerate(['hall','delve']):
    x=48+col*690;text(d,(x,165),'Guild Hall' if world=='hall' else 'Delve',27,GOLD,'serif')
    for row,(key,label) in enumerate([('work-a','Working / 0 ms'),('waiting','Blocked / 600 ms, now still'),('spoils','Returned / 0 ms, parcel held'),('settled','Completed / 3000 ms, now still')]):
        y=231+row*344;text(d,(x,y-28),label,18,MUTED);im.paste(actor_strip(f'{world}-{key}'),(x,y))
text(d,(48,1640),'Working: two frames, 500 ms each. No whole-body bob. Gear and hands carry the movement.',19)
text(d,(48,1673),'Counsel: one 600 ms raised-hand gesture, then a still wait. Repeated reports do not restart it.',19)
text(d,(48,1706),'Spoils: parcel placed at 1000 ms; shared celebration ends at 3000 ms. Completion stays distinct.',19)
text(d,(48,1739),'Reduced / still: one meaningful static pose with no decorative or cleanup wake. Unknown stays uncertain.',19)
text(d,(48,1794),'Playback: working-loop.gif repeats; party-journey.gif plays once and includes the 1000 ms placement.',18,GOLD)
im.save(ROOT/'03-ritual-timeline.png')

im,d=page('04','A seal means the send was confirmed','Actual counsel state machine and Ratatui overlays. The blocked pose persists until a newer Herdr working report.',2040)
for index,(key,title) in enumerate([('draft','Draft'),('sending','Sending'),('confirmed','Confirmed / brief seal'),('rejected','Rejected / text can be corrected'),('uncertain','Uncertain / avoid duplicate text'),('submit','Submit recovery / retry Enter only'),('late-ascii','Late confirmation / ANSI 16 + ASCII')]):
    x=48+(index%2)*690;y=220+(index//2)*416
    text(d,(x,y-35),title,22,GOLD,'serif');im.paste(terminal(f'counsel-{key}'),(x,y))
text(d,(738,1533),'Confirmed delivery does not establish',21)
text(d,(738,1565),'that counsel was read or acted upon.',21)
text(d,(738,1637),'The red seal uses the existing notice row.',18,MUTED)
text(d,(738,1669),'It expires after six seconds; no new panel.',18,MUTED)
text(d,(48,1919),'Class cards retain their existing native art and authored fallbacks. Native transport is a separate manual gate.',18)
im.save(ROOT/'04-counsel-outcomes.png')

# Local playback frames are placed without resampling production pixels unevenly.
def playback(keys,label):
    out=Image.new('RGB',(1060,400),BG);draw=ImageDraw.Draw(out)
    text(draw,(30,17),'QUESTMANCER  /  PRODUCTION FIXTURE PLAYBACK',14,GOLD,'mono')
    text(draw,(30,53),label,24,INK,'serif')
    for col,(world,key) in enumerate(zip(['hall','delve'],keys)):
        x=30+col*520;text(draw,(x,100),('Guild Hall' if world=='hall' else 'Delve')+' / individual room fixtures',16,MUTED)

        for n, name in enumerate(NAMES): text(draw,(x+15+n*160,130),name,15,GOLD,'mono')
        out.paste(actor_strip(f'{world}-{key}'),(x,160))
    return out
work=[playback([key,key],'Working / '+str(i*500)+' ms') for i,key in enumerate(['work-a','work-b'])]
work[0].save(ROOT/'working-loop.gif',save_all=True,append_images=work[1:],duration=[500,500],loop=0,disposal=2)
frames=[];times=[]
for key,label,ms in [('work-a','Working / green censer and silver focus',500),('work-b','Working / the second frame',500),('counsel','A newer blocked report / asking for counsel',600),('waiting','600 ms later / waiting, now still',1400),('waiting','Counsel confirmed / remains blocked (seal shown in sheet 04)',1400),('work-a','A newer Herdr working report / work resumes',500),('work-b','Working / the second frame',500)]:
    frames.append(playback([key,key],label));times.append(ms)
for index in range(25):
    frames.append(playback([f'return-{index}']*2,f'Returned spoils / {index*125} ms'+(' / parcel placed' if index>=8 else ' / parcel held')))
    # GIF stores centiseconds. Alternate 120/130 ms for exact 8 fps in each pair.
    times.append(2000 if index==24 else 120 if index%2==0 else 130)
frames[0].save(ROOT/'party-journey.gif',save_all=True,append_images=frames[1:],duration=times,disposal=2)
# A small native-file proof for unchanged class-card layouts and the current Delve reference.
proof=Image.new('RGB',(1200,680),BG);draw=ImageDraw.Draw(proof)
for index in range(len(NAMES)):
    text(draw,(index*600+10,10),NAMES[index]+' / current card fallback',19,GOLD)
    proof.paste(terminal(f'card-{index}'),(index*600,55))
proof.paste(pixels('delve-golden').resize((480,270),Image.Resampling.NEAREST),(30,410))
text(draw,(550,460),'Quiet Delve reference fixture / current production art',22)
text(draw,(550,500),'Production paths; fixture exports.',20,MUTED)
proof.save(ROOT/'05-card-and-delve-check.png')
print('Wrote production pose, room, timing, counsel and card sheets plus two playback GIFs.')
