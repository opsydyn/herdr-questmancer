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
    text(d,(48,28),'QUESTMANCER  /  LIBRARIAN ART REVIEW  /  '+number,15,GOLD,'mono')
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

im,d=page('01','A librarian worth asking','Broader face, low gold spectacles, orange fur, a short purple robe and books. Static at both authored sizes.',1110)
text(d,(48,165),'World / 16 × 24',25,GOLD,'serif')
text(d,(552,165),'Ledger fallback / 24 × 32',25,GOLD,'serif')
text(d,(1140,165),'Native reference',23,GOLD,'serif')
for key,x,y,scale,label in [('before-world',76,232,8,'Before'),('world',300,232,8,'Revised'),('before-portrait',573,216,7,'Before'),('portrait',838,216,7,'Revised')]:
    put(im,key,x,y,scale); text(d,(x,466),label,20)
    put(im,key,x+scale*12,y+250,1)
ref=Image.open(ROOT.parents[3]/'src/assets/librarian.png').convert('RGB').resize((250,250),Image.Resampling.LANCZOS)
im.paste(ref,(1140,216))
text(d,(48,540),'World feet now share row 21 with the party. The head is broader; the neck and long legs are gone.',19)
text(d,(48,573),'The Ledger has its own face, fingers and book spines. Its 24 × 32 canvas is no longer padded world art.',19,MUTED)
d.line((48,620,1392,620),fill=EDGE)
text(d,(48,647),'Readability at production scale',26,GOLD,'serif')
for col,(bg,label) in enumerate([('dark','Dark room'),('warm','Warm wood'),('parchment','Parchment')]):
    x=48+col*462; text(d,(x,695),label,21)
    for j,mode in enumerate(['rgb','ansi']):
        xx=x+j*210; put(im,f'world-{bg}-{mode}',xx,740,5); put(im,f'portrait-{bg}-{mode}',xx+96,722,4)
        text(d,(xx,879),'Truecolour' if mode=='rgb' else 'ANSI 16',16,MUTED)
        put(im,f'world-{bg}-{mode}',xx,917); put(im,f'portrait-{bg}-{mode}',xx+28,909)
        text(d,(xx+65,918),'actual 1x',13,MUTED)
text(d,(48,990),'Help access and native illustration routing are unchanged. No animation or new persona state.',18)
im.save(ROOT/'01-librarian-proportions.png')

im,d=page('02','At home in the Hall and Ledger','Current production room pixels and actual Ratatui overlay cells. Text reconstructed in Menlo; not terminal screenshots.',1450)
text(d,(48,167),'Canonical Hall / 160 × 90 RGB',25,GOLD,'serif'); put(im,'hall',48,215,4)
text(d,(833,167),'Compact Hall / 64 × 40 RGB',25,GOLD,'serif'); put(im,'compact',833,215,7)
text(d,(833,522),'The help station keeps its complete click bounds.',18)
text(d,(833,553),'Smaller tiers keep ? access to the Ledger.',18,MUTED)
for key,x in [('hall',48),('compact',400)]:
    bounds=DATA[key]['librarian'][0]; xx,yy,w,h=bounds
    crop=pixels(key).crop((xx,yy,xx+w,yy+h)); crop=crop.resize((w*6,h*6),Image.Resampling.NEAREST); im.paste(crop,(x,618))
    text(d,(x+116,650),'Librarian',20); text(d,(x+116,682),key.title(),16,MUTED)
d.line((48,800,1392,800),fill=EDGE)
text(d,(48,830),'Ledger / truecolour',25,GOLD,'serif'); text(d,(752,830),'Ledger / ANSI 16',25,GOLD,'serif')
im.paste(terminal('ledger'),(48,887)); im.paste(terminal('ledger-ansi'),(752,887))
text(d,(48,1261),'Both views use the authored fallback because no native image protocol is supplied in this fixture.',18)
text(d,(48,1292),'The existing native illustration remains available when the complete terminal transport supports it.',18,MUTED)
text(d,(48,1335),'Review: face / spectacles / book shapes / robe length / grounding / readable handbook text.',18)
im.save(ROOT/'02-librarian-context.png')
print('Wrote two Librarian review sheets.')
