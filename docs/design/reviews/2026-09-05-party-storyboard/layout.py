"""Lay out static review sheets from RGB data exported by Questmancer.

Sprite decoding and room rendering happen in export.rs through existing
production functions. This script only typesets the review documents; it has
no live app, animation, image-generation, or terminal-control behaviour.
"""
import argparse
from pathlib import Path
import json
from PIL import Image, ImageDraw, ImageFont

ROOT=Path(__file__).resolve().parent
parser=argparse.ArgumentParser()
parser.add_argument('input',type=Path)
args=parser.parse_args()
DATA=json.loads(args.input.read_text())
BG='#151b21'; PANEL='#1d252d'; EDGE='#35414b'; INK='#eee1c3'; MUTED='#adb7b5'; GOLD='#dfb86c'; TEAL='#8bcbc3'
REG='/System/Library/Fonts/Avenir Next.ttc'
SER='/System/Library/Fonts/Supplemental/Georgia.ttf'
MONO='/System/Library/Fonts/Menlo.ttc'

def font(size,kind='sans'):
    p={'sans':REG,'serif':SER,'mono':MONO}[kind]
    return ImageFont.truetype(p,size)

def text(draw,xy,s,size=20,fill=INK,kind='sans',anchor=None):
    draw.text(xy,s,font=font(size,kind),fill=fill,anchor=anchor)

def image_from(frame):
    im=Image.new('RGBA',(frame['width'],frame['height']))
    im.putdata([(*p,255) if p is not None else (0,0,0,0) for p in frame['pixels']])
    return im

def put(page,frame,x,y,scale=1,silhouette=False):
    im=image_from(frame)
    if silhouette:
        alpha=im.getchannel('A')
        im=Image.new('RGBA',im.size,(238,225,195,255))
        im.putalpha(alpha)
    im=im.resize((im.width*scale,im.height*scale),Image.Resampling.NEAREST)
    page.paste(im,(x,y),im)

def page(number,title,subtitle,height=1000):
    im=Image.new('RGB',(1440,height),BG);d=ImageDraw.Draw(im)
    text(d,(48,28),f'QUESTMANCER  /  PARTY STORYBOARD  /  {number}',15,GOLD,'mono')
    text(d,(48,62),title,36,INK,'serif')
    text(d,(48,112),subtitle,19,MUTED)
    d.line((48,height-55,1392,height-55),fill=EDGE,width=1)
    text(d,(48,height-39),'REVIEW CANDIDATES  ·  05 SEPTEMBER 2026  ·  PRODUCTION ROUTES UNCHANGED',14,MUTED,'mono')
    return im,d

ids=['wizard','ranger','barbarian']
labels={'wizard':'Wizard','ranger':'Ranger','barbarian':'Barbarian'}
# 1. Proportion comparison. Enlargement is nearest-neighbour; the 1x line is literal.
im,d=page('01','The party, in proportion','Same 16×24 canvas. Same foot line. A larger head, a shorter body, and clear class gear.',1040)
zone_notes={
'wizard':['Face + beard: 6 to 8 rows','Body: 6 to 5 rows','Hat is a separate silhouette band.'],
'ranger':['Head + hood: 7 to 9 rows','Body: 9 to 6 rows','Bow and quiver stay outside the face.'],
'barbarian':['Head + hair: 7 to 10 rows','Body: 10 to 6 rows','Broad shoulders; short, grounded feet.']}
for i,slug in enumerate(ids):
    x=48+i*456;c=DATA['classes'][slug]
    d.rounded_rectangle((x,166,x+432,956),radius=12,fill=PANEL,outline=EDGE)
    text(d,(x+24,185),labels[slug],28,INK,'serif')
    text(d,(x+24,227),'CURRENT',14,MUTED,'mono');text(d,(x+228,227),'PROPOSED',14,TEAL,'mono')
    put(im,c['before'],x+28,259,8);put(im,c['frames']['working_a'],x+230,259,8)
    d.line((x+20,435,x+406,435),fill='#4b5a61',width=1)
    text(d,(x+24,467),'At 1×',15,MUTED,'mono')
    put(im,c['before'],x+160,466);put(im,c['frames']['working_a'],x+230,466)
    d.line((x+24,512,x+408,512),fill=EDGE)
    text(d,(x+24,533),'SILHOUETTE CHECK',14,GOLD,'mono')
    put(im,c['before'],x+51,566,5,True);put(im,c['frames']['working_a'],x+253,566,5,True)
    text(d,(x+24,725),zone_notes[slug][0],19)
    text(d,(x+24,757),zone_notes[slug][1],19)
    text(d,(x+24,793),zone_notes[slug][2],15,MUTED)
    text(d,(x+24,842),'AUTHORED MATERIAL PALETTE',13,MUTED,'mono')
    for j,key in enumerate(['o','c','C','K','h','r','R','D','l','M','e']):
        sx=x+24+j*33;d.rectangle((sx,878,sx+24,902),fill='#'+c['palette'][key]);text(d,(sx+8,909),key,13,MUTED,'mono')
im.save(ROOT/'01-proportions.png')

# 2. Static frames as a storyboard, no autoplay before silhouette approval.
columns=[('working_a','Working A','Read / check / grip'),('working_b','Working B','Small tool change'),('counsel','Counsel','Patient, clear signal'),('resumed','Resumed','Herdr reports working'),('spoils','Spoils','Returned; unverified'),('resting','Resting','Static, relaxed stance'),('unknown','Unknown','Uncertain whereabouts'),('settled','Completed','Calm after the return')]
im,d=page('02','Three adventurers. One readable journey.','Static pose studies at 4×. The resumed frame is shown only after an observed working event.',1180)
for j,(_,title,_) in enumerate(columns):
    x=164+j*154;text(d,(x+61,166),title,17,INK,anchor='mt')
for i,slug in enumerate(ids):
    y=210+i*220;c=DATA['classes'][slug]
    d.rounded_rectangle((48,y,1392,y+200),radius=10,fill=PANEL,outline=EDGE)
    text(d,(65,y+28),labels[slug],21,INK,'serif')
    for j,(key,title,caption) in enumerate(columns):
        x=164+j*154
        put(im,c['frames'][key],x+29,y+26,4)
        if key in ['counsel','unknown','settled']:
            mark={'counsel':'counsel','unknown':'unknown','settled':'completed'}[key]
            put(im,DATA['markers'][mark],x+53,y+3,3)
        text(d,(x+62,y+136),title,16,INK,anchor='mt')
        # Width-limited two-line caption, hand-authored to remain legible.
        lines={'working_a':[{'wizard':'Turns a page','ranger':'Checks a map','barbarian':'Adjusts a grip'}[slug]], 'working_b':['Same foot anchor'], 'counsel':['No urgency escalation'], 'resumed':['Not a send receipt'], 'spoils':['No approval implied'], 'resting':['No redraw loop'], 'unknown':['No invented outcome'], 'settled':['Persists as done']}[key]
        for n,line in enumerate(lines):text(d,(x+62,y+162+n*17),line,12,MUTED,anchor='mt')
text(d,(48,905),'Proposed timing',24,GOLD,'serif')
text(d,(48,943),'Working: 2 distinct frames, initially 2 fps. Counsel: one short gesture, then still. Return: ≤3 seconds, then completed.',18)
text(d,(48,971),'Reduced / still: a clear static pose and marker. A newer state, departure or disconnection interrupts old theatre.',17,MUTED)
d.line((48,1009,1392,1009),fill=EDGE)
text(d,(48,1031),'DEPARTURE',15,GOLD,'mono')
d.rectangle((230,1026,278,1098),outline=EDGE)
text(d,(312,1030),'An empty actor slot',20,INK,'serif')
text(d,(312,1068),'Exited adventurers leave the live party. No actor, nameplate or completion flourish remains.',17,MUTED)
im.save(ROOT/'02-moments.png')

# 3. Backgrounds and positions come from existing production renderers.
im,d=page('03','At home, and down in the Delve','Placement studies on production backgrounds. Proposed sprites use the current actor bounds and foot line.',1030)
text(d,(48,160),'CURRENT SPRITES',15,MUTED,'mono');text(d,(752,160),'PROPOSED PROPORTIONS',15,TEAL,'mono')
for i,name in enumerate(['guild','delve']):
    y=198+i*376
    put(im,DATA['rooms'][name]['before'],48,y,4)
    put(im,DATA['rooms'][name]['proposed'],752,y,4)
    text(d,(54,y+338),name.upper(),14,INK,'mono')
text(d,(48,948),'Composition mock-ups: transient lighting/markers need the later production pilot. These are not live Herdr captures.',16,MUTED)
im.save(ROOT/'03-world-context.png')

# Native half-block cells are reconstructed exactly; ordinary text is typeset for
# this static sheet. This is evidence of layout, not a real terminal screenshot.
def terminal_image(buffer,cw=8,ch=16):
    out=Image.new('RGB',(buffer['columns']*cw,buffer['rows']*ch));d=ImageDraw.Draw(out);f=font(13,'mono')
    for i,c in enumerate(buffer['cells']):
        x=(i%buffer['columns'])*cw;y=(i//buffer['columns'])*ch;fg=tuple(c['fg']);bg=tuple(c['bg'])
        d.rectangle((x,y,x+cw-1,y+ch-1),fill=bg)
        if c['text']=='▀':d.rectangle((x,y,x+cw-1,y+ch//2-1),fill=fg)
        elif c['text']=='▄':d.rectangle((x,y+ch//2,x+cw-1,y+ch-1),fill=fg)
        elif c['text']=='█':d.rectangle((x,y,x+cw-1,y+ch-1),fill=fg)
        elif c['text'].strip():d.text((x,y-1),c['text'],font=f,fill=fg)
    return out
im,d=page('04','Leave room for the useful words','Existing label and card layout, with proposed world sprites. The selected Ranger retains the current portrait fallback.',1760)
im.paste(terminal_image(DATA['rooms']['guild']['labels']),(80,169))
text(d,(80,905),'Guild Hall with current identity labels and command ribbon',18,GOLD)
im.paste(terminal_image(DATA['rooms']['guild']['card']),(80,953))
text(d,(80,1680),'Static Ratatui-buffer reconstruction. Native terminal graphics and live transport are unreviewed.',15,MUTED)
im.save(ROOT/'04-card-and-labels.png')

# 5. Readability checks, backgrounds and proposed shared roster cues.
im,d=page('05','A small party must still tell the truth','Independent 8×12 roster studies and shape cues. Marker grammar is proposed; production state handling is unchanged.',1140)
backgrounds=[('DEEP DELVE','#121d24'),('NEUTRAL','#333940'),('WARM HALL','#58351f')]
for i,slug in enumerate(ids):
    x=48+i*456;c=DATA['classes'][slug]
    text(d,(x,165),labels[slug],24,INK,'serif')
    for j,(name,bg) in enumerate(backgrounds):
        bx=x+j*138;d.rectangle((bx,212,bx+128,371),fill=bg)
        put(im,c['frames']['working_a'],bx+24,228,5)
        text(d,(bx+64,382),name,11,MUTED,'mono',anchor='mt')
    text(d,(x,430),'Current roster',15,MUTED);text(d,(x+204,430),'Proposed roster',15,TEAL)
    put(im,c['roster_before'],x+49,464,7);put(im,c['roster'],x+260,464,7)
    text(d,(x,576),'ANSI-16  /  production colour adapter',13,MUTED,'mono')
    ansi=terminal_image(c['ansi'],6,12);im.paste(ansi,(x+34,609))
    text(d,(x+166,610),'At 1×',14,MUTED,'mono');put(im,c['frames']['working_a'],x+244,610);put(im,c['roster'],x+300,616)
    text(d,(x+166,652),'Shape carries the state.',15)
    text(d,(x+166,678),'Colour supports recognition.',15,MUTED)
text(d,(48,795),'Proposed shared cues at roster scale',24,GOLD,'serif')
marker_order=['working','counsel','resting','completed','unknown']
for i,name in enumerate(marker_order):
    x=48+i*276
    d.rounded_rectangle((x,843,x+248,1041),radius=10,fill=PANEL,outline=EDGE)
    put(im,DATA['markers'][name],x+108,859,5)
    put(im,DATA['classes']['ranger']['roster'],x+99,894,6)
    text(d,(x+124,980),name.title(),18,INK,anchor='mt')
    put(im,DATA['markers'][name],x+29,994,1);put(im,DATA['classes']['ranger']['roster'],x+27,1001,1)
    text(d,(x+47,997),'1× reference',12,MUTED,'mono')
im.save(ROOT/'05-small-scale.png')

# Literal native-size strip, useful without document scaling.
strip=Image.new('RGBA',(16*8,24*3))
for i,slug in enumerate(ids):
    for j,(key,_,_) in enumerate(columns):put(strip,DATA['classes'][slug]['frames'][key],j*16,i*24)
strip.save(ROOT/'native-frames.png')

# 6. Current fallback portraits stay visible as continuity references.
im,d=page('06','One class, three authored scales','Current portrait fallbacks beside the proposed world and roster studies. Portrait repainting remains a later art decision.',940)
for i,slug in enumerate(ids):
    x=48+i*456;c=DATA['classes'][slug]
    d.rounded_rectangle((x,165,x+432,841),radius=10,fill=PANEL,outline=EDGE)
    text(d,(x+24,186),labels[slug],28,INK,'serif')
    text(d,(x+24,238),'CURRENT 24×32 FALLBACK',14,MUTED,'mono')
    put(im,c['portrait'],x+120,282,7)
    text(d,(x+24,556),'PROPOSED 16×24',14,TEAL,'mono')
    text(d,(x+250,556),'8×12 STUDY',14,TEAL,'mono')
    put(im,c['frames']['working_a'],x+54,603,6)
    put(im,c['roster'],x+286,627,8)
text(d,(48,867),'Existing portraits remain references here. Approving these world studies does not approve a new portrait style.',16,MUTED)
im.save(ROOT/'06-portrait-continuity.png')
print('Exported six static review sheets and native-frames.png')
