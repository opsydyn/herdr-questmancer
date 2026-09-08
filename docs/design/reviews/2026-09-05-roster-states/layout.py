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
    text(d,(48,28),'QUESTMANCER  /  ROSTER STATE REVIEW  /  '+number,15,GOLD,'mono')
    text(d,(48,62),title,36,INK,'serif'); text(d,(48,112),subtitle,18,MUTED)
    d.line((48,height-55,1392,height-55),fill=EDGE)
    text(d,(48,height-38),'PRODUCTION RENDER PATHS  ·  05 SEPTEMBER 2026  ·  TERMINAL / LIVE ACCEPTANCE SEPARATE',13,MUTED,'mono')
    return im,d
def pixels(key):
    data=DATA[key]; im=Image.new('RGB',(data['width'],data['height']))
    im.putdata([tuple(p) for p in data['pixels']]); return im
def put(im,key,x,y,scale=1):
    src=pixels(key); im.paste(src.resize((src.width*scale,src.height*scale),Image.Resampling.NEAREST),(x,y))
def terminal(key):
    data=DATA[key]; cw=8; ch=16; im=Image.new('RGB',(data['columns']*cw,data['rows']*ch)); d=ImageDraw.Draw(im)
    for i,c in enumerate(data['cells']):
        x=(i%data['columns'])*cw; y=(i//data['columns'])*ch
        d.rectangle((x,y,x+cw-1,y+ch-1),fill=tuple(c['bg']))
        if c['text']=='▀': d.rectangle((x,y,x+cw-1,y+ch//2-1),fill=tuple(c['fg']))
        elif c['text'].strip(): text(d,(x,y-1),c['text'],13,tuple(c['fg']),'mono')
    return im
states=['working','counsel','resting','completed','unknown']
im,d=page('01','Five states, even without a nameplate','One fixed Wizard persona at 30×30 RGB pixels. Authored 8×12 body; shared 5×5 state cue. Enlarged 6×.',1140)
for j,state in enumerate(states): text(d,(235+j*230,157),state.title(),20,GOLD)
for i,(world,mode,title) in enumerate([('guild','rgb','Guild Hall'),('delve','rgb','Delve'),('guild','ansi','Guild Hall'),('delve','ansi','Delve')]):
    y=192+i*220; text(d,(48,y+35),title,23,INK,'serif'); text(d,(48,y+75),'Truecolour' if mode=='rgb' else 'ANSI 16',16,MUTED)
    for j,state in enumerate(states):
        x=232+j*230; key=f'{world}-{state}-{mode}'
        put(im,key,x,y,6); put(im,key,x+150,y+184)
        text(d,(x,y+189),'Actual 1x',12,MUTED)
im.save(ROOT/'01-state-cues.png')

im,d=page('02','The return ends. Completed stays readable.','Full motion: gutter sparkles at 8 fps for less than three seconds. The completion cue stays in place.',1040)
times=[0,125,250,2999,3000,6000]
for j,t in enumerate(times): text(d,(153+j*208,164),f'{t} ms',19,GOLD)
for i,world in enumerate(['guild','delve']):
    y=202+i*237; text(d,(48,y+50),'Hall' if world=='guild' else 'Delve',22,INK,'serif')
    for j,t in enumerate(times):
        x=150+j*208; key=f'{world}-return-{t}'; put(im,key,x,y,5)
        delay=DATA[key]['next_ms']; text(d,(x,y+167),'No timer' if delay is None else f'Next: {delay} ms',15,MUTED)
d.line((48,656,1392,656),fill=EDGE)
text(d,(48,680),'Reduced and still',26,GOLD,'serif')
text(d,(48,721),'One stable completed cue.',19)
text(d,(48,752),'No sparkles or cleanup timer.',19)
for j,(world,mode) in enumerate([('guild','reduced'),('guild','still'),('delve','reduced'),('delve','still')]):
    x=460+j*224; put(im,f'{world}-{mode}',x,702,4); text(d,(x,837),f'{world.title()} / {mode}',15,MUTED)
text(d,(48,913),'Newer presence cancels the old return. Disconnection stops it; reconnecting cannot replay retained summons.',18)
text(d,(48,944),'“Completed” means Herdr reported done. It does not imply tests passed, merge, or approval.',18,MUTED)
im.save(ROOT/'02-completion-deadline.png')

im,d=page('03','Keep cues and focus clear','Production half-block cells with current overlays. Text is reconstructed in Menlo; these are not terminal screenshots.',1360)
for col,world in enumerate(['guild','delve']):
    x=48+col*704; text(d,(x,164),'Guild Hall' if world=='guild' else 'Delve',27,GOLD,'serif')
    for row,mode in enumerate(['unicode','ascii']):
        y=211+row*302; im.paste(terminal(f'{world}-labels-{mode}'),(x,y))
        text(d,(x,y+250),'60 columns × 15 rows / '+('Unicode + truecolour' if mode=='unicode' else 'ASCII + ANSI 16'),17,MUTED)
    im.paste(terminal(f'{world}-crowded'),(x,844))
    im.paste(terminal(f'{world}-reconnect'),(x+270,844))
    text(d,(x,800),'Six adventurers / 30 columns × 24 rows',18)
    text(d,(x,1237),'Connected',17,MUTED); text(d,(x+270,1237),'Reconnecting',17,MUTED)
im.save(ROOT/'03-terminal-layout.png')
print('Wrote 3 production review sheets.')
