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
    text(d,(48,28),'QUESTMANCER  /  CARD FALLBACKS  /  '+number,15,GOLD,'mono')
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

NAMES=['Rogue','Pathseeker','Runewright']
im,d=page('01','The same adventurer on the card','The new personalised world sprites replace the three old long-bodied card fallbacks. Native pixels; no stretching.',920)
for index,name in enumerate(NAMES):
    x=48+index*462
    text(d,(x,170),name,29,GOLD,'serif')
    text(d,(x,228),'Previous fallback',17,MUTED);text(d,(x+211,228),'Current fallback',17,GOLD)
    put(im,f'before-{index}',x,275,6);put(im,f'after-{index}',x+218,275,6)
    for key,xx in [(f'before-{index}',x+60),(f'after-{index}',x+278)]:
        put(im,key,xx,515);text(d,(xx-11,564),'1x',13,MUTED)
    text(d,(x,628),'Current world sprite',18,GOLD)
    put(im,f'world-{index}',x+260,619,4)
    text(d,(x,663),'Exact same pixels and palette',16)
    text(d,(x,692),'centred in the card canvas.',16,MUTED)
text(d,(48,773),'The 24 × 32 card canvas keeps its layout. The sprite stays 16 × 24, with four pixels of space on each side.',19)
text(d,(48,805),'Skin, hair and trim now match the adventurer in the room. Native class illustrations keep their existing route.',18,MUTED)
im.save(ROOT/'01-fallback-before-after.png')

im,d=page('02','Stocky sprites, readable cards','Production Ratatui card buffers. Text reconstructed in Menlo; these are fixtures, not terminal screenshots.',1670)
text(d,(48,165),'Truecolour / Unicode',25,GOLD,'serif');text(d,(738,165),'ANSI 16 / ASCII',25,GOLD,'serif')
for index,name in enumerate(NAMES):
    y=254+index*422
    for key,x in [(f'card-{index}',48),(f'card-ansi-{index}',738)]:
        text(d,(x,y-35),name,23,INK,'serif');im.paste(terminal(key),(x,y))
text(d,(48,1545),'Existing card bounds, text and actions remain in place. The new fallback is a static class portrait.',19)
text(d,(48,1576),'Mage and Sorcerer retain their independent fallbacks. The revised Librarian remains independent.',18,MUTED)
im.save(ROOT/'02-cards-in-context.png')
print('Wrote before/after and production card-context review sheets.')
