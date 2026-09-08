"""Typeset authored pixel-data studies exported through Questmancer's renderer."""
from pathlib import Path
import hashlib
import json
import sys
from PIL import Image, ImageDraw, ImageFont

ROOT = Path(__file__).resolve().parent
DATA = json.loads(Path(sys.argv[1]).read_text())
BG = '#151c22'; PANEL = '#202b32'; INK = '#efe2c7'; MUTED = '#acbcb9'; GOLD = '#dfb56d'
IDS = ['mage', 'sorcerer']
FONT = '/System/Library/Fonts/Avenir Next.ttc'
SERIF = '/System/Library/Fonts/Supplemental/Georgia.ttf'

def text(d, x, y, value, size=18, fill=INK, serif=False):
    d.text((x, y), value, font=ImageFont.truetype(SERIF if serif else FONT, size), fill=fill)

def put(im, frame, x, y, scale=1, silhouette=False):
    sprite = Image.new('RGBA', (frame['width'], frame['height']))
    sprite.putdata([((239, 226, 199, 255) if silhouette else (*p, 255)) if p else (0, 0, 0, 0) for p in frame['pixels']])
    sprite = sprite.resize((sprite.width * scale, sprite.height * scale), Image.Resampling.NEAREST)
    im.paste(sprite, (x, y), sprite)

def page(number, title, subtitle, height):
    im = Image.new('RGB', (1520, height), BG); d = ImageDraw.Draw(im)
    text(d, 44, 24, 'QUESTMANCER  /  CENSER & HALO STUDIES  /  ' + number, 15, GOLD)
    text(d, 44, 56, title, 35, serif=True)
    text(d, 44, 108, subtitle, 19, MUTED)
    d.line((44, height - 52, 1476, height - 52), fill='#405058')
    text(d, 44, height - 39, '08 SEPTEMBER 2026  ·  DESIGN CANDIDATES  ·  PRODUCTION ACTIVATION AWAITS APPROVAL', 14, MUTED)
    return im, d

im, d = page('01', 'A green ember and a golden halo', 'Larger heads, shorter bodies, planted feet. Class gear carries the identity.', 1170)
notes = {
'mage': ['Deep violet hood and green skull staff.', 'One hand tends a small silver censer.', 'The ember settles; the staff stays planted.'],
'sorcerer': ['A gold halo floats clear of the head.', 'Pale robes frame a silver hand-held focus.', 'The hands turn it around a warm spark.']}
for i, slug in enumerate(IDS):
    x = 174 + i * 720; c = DATA['classes'][slug]
    d.rounded_rectangle((x, 156, x+464, 1086), 12, fill=PANEL)
    text(d, x+20, 174, c['class'], 29, serif=True)
    text(d, x+20, 218, 'CURRENT WORLD', 14, MUTED); text(d, x+245, 218, 'PROPOSED', 14, GOLD)
    put(im, c['before'], x+31, 254, 8); put(im, c['frames']['working_a'], x+259, 254, 8)
    text(d, x+20, 466, 'NATIVE 1×', 14, MUTED)
    put(im, c['before'], x+162, 466); put(im, c['frames']['working_a'], x+258, 466)
    text(d, x+20, 521, 'SILHOUETTE', 14, GOLD)
    put(im, c['frames']['working_a'], x+33, 552, 5, True)
    for n, line in enumerate(notes[slug]): text(d, x+20, 704+n*29, line, 18)
    text(d, x+20, 818, 'CURRENT CARD / ROSTER', 14, MUTED)
    put(im, c['portrait'], x+32, 855, 5); put(im, c['roster'], x+235, 867, 5)
    text(d, x+197, 952, 'Retained for this study.', 16, MUTED)
    text(d, x+197, 981, 'Review card alignment', 16, MUTED)
    text(d, x+197, 1008, 'in a later production slice.', 16, MUTED)
im.save(ROOT/'01-directions.png')

keys = [('working_a','Work A'), ('working_b','Work B'), ('counsel_raise','Signal'), ('counsel_wait','Wait'), ('spoils','Spoils'), ('settled','Settled'), ('resting','Rest'), ('unknown','Unknown')]
im, d = page('02', 'Small hands, distinct rituals', 'Authored 16×24 frames shown at 4×. State markers stay readable when all motion is off.', 910)
for n,(key,label) in enumerate(keys): text(d, 208+n*157, 162, label, 18, GOLD)
for i,slug in enumerate(IDS):
    y=208+i*224; c=DATA['classes'][slug]
    d.rounded_rectangle((44,y,1476,y+204),12,fill=PANEL)
    text(d,61,y+29,c['class'],21,serif=True)
    for n,(key,label) in enumerate(keys):
        x=207+n*157
        put(im,c['frames'][key],x+8,y+28,4)
        if key in ['counsel_raise','counsel_wait','settled','unknown']:
            marker={'counsel_raise':'counsel','counsel_wait':'counsel','settled':'completed','unknown':'unknown'}[key]
            put(im,DATA['markers'][marker],x+32,y+5,3)
        caption={'working_a':'500 ms','working_b':'500 ms','counsel_raise':'0–600 ms','counsel_wait':'Then still','spoils':'0–1000 ms','settled':'Placed; then calm','resting':'Idle ≠ done','unknown':'No outcome claim'}[key]
        text(d,x,y+145,caption,14,MUTED)
        put(im,c['frames'][key],x+32,y+174)
text(d,44,690,'Resume only when Herdr reports working. Sending counsel does not restart the work loop.',20)
text(d,44,728,'Spoils are a sealed reagent case or wrapped focus: completion metaphors, never verified contents.',18,MUTED)
text(d,44,760,'Fresh spoils settle within three seconds. Reduced/still use static poses; exited adventurers leave the scene.',18,MUTED)
text(d,44,792,'The shared lantern and completion / unknown marks keep their existing meanings.',18,MUTED)
im.save(ROOT/'02-rituals.png')

im,d=page('03','In the Guild Hall and the Delve','Production backgrounds and actor bounds; candidate sprite compositing is a placement study.',1080)
text(d,44,157,'CURRENT',16,MUTED);text(d,824,157,'PROPOSED',16,GOLD)
for i,room in enumerate(['guild','delve']):
    y=195+i*388
    put(im,DATA['rooms'][room]['before'],44,y,4)
    put(im,DATA['rooms'][room]['proposed'],824,y,4)
text(d,44,985,'Mage working · Sorcerer seeking counsel. No live Herdr or native-terminal acceptance claimed.',17,MUTED)
im.save(ROOT/'03-worlds.png')

# Proposed work cadence only. This does not invoke or validate runtime scheduling.
frames=[]
for moment in ['working_a','working_b']:
    im,d=page('04','The working beat','Proposed two-frame loop, 500 ms per frame. Feet and faces stay still while hands and tools change.',530)
    for i,slug in enumerate(IDS):
        x=174+i*720;c=DATA['classes'][slug]
        d.rounded_rectangle((x,156,x+464,444),12,fill=PANEL)
        text(d,x+22,172,c['class'],25,serif=True)
        put(im,c['frames'][moment],x+35,227,8)
        text(d,x+260,232,'Native 1×',16,MUTED);put(im,c['frames'][moment],x+286,271)
        text(d,x+236,346,'Design playback',17,MUTED)
    frames.append(im)
frames[0].save(ROOT/'04-working-study.gif',save_all=True,append_images=frames[1:],duration=500,loop=0,disposal=2)
outputs={p.name:hashlib.sha256(p.read_bytes()).hexdigest() for p in sorted(ROOT.iterdir()) if p.suffix in {'.png','.gif'}}
(ROOT/'verification.json').write_text(json.dumps({'scope':'Design candidates; no production routing or native acceptance','visual_approval':'pending','dimensions':'All 16 authored frames are 16x24 with identical foot anchors per class','checks':'Rust exporter validates palette, frame bounds, feet, nonempty frames and working pixel changes','output_sha256':outputs},indent=2)+'\n')
