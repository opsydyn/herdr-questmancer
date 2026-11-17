"""Typeset authored Librarian studies on production background exports."""
from pathlib import Path
import json
from PIL import Image,ImageDraw,ImageFont
ROOT=Path(__file__).resolve().parent;CURRENT=json.loads((ROOT/'current.json').read_text());BGDATA=json.loads((ROOT/'background.json').read_text())
BG='#151f22';PANEL='#202d30';INK='#f0e4c9';MUTED='#acbdb3';GOLD='#e4bc73';EDGE='#3a4b49'
FONT='/System/Library/Fonts/Avenir Next.ttc';SERIF='/System/Library/Fonts/Supplemental/Georgia.ttf'
IDS=['current','hearth','stacks'];TITLES=['Current master','A / Hearth Archivist','B / Keeper of the Stacks']
def text(d,xy,s,size=18,fill=INK,serif=False):d.text(xy,s,font=ImageFont.truetype(SERIF if serif else FONT,size),fill=fill)
def pixels(frame):
    im=Image.new('RGBA',(frame['width'],frame['height']));im.putdata([(*p,255) if p is not None else (0,0,0,0) for p in frame['pixels']]);return im
def put(target,source,xy,scale=1):
    if isinstance(source,dict):source=pixels(source)
    source=source.resize((source.width*scale,source.height*scale),Image.Resampling.NEAREST);target.paste(source,xy,source if source.mode=='RGBA' else None)
def page(title,subtitle,h):
    im=Image.new('RGB',(1500,h),BG);d=ImageDraw.Draw(im);text(d,(40,24),'QUESTMANCER / LIBRARIAN STUDY / 17 NOVEMBER 2026',15,GOLD)
    text(d,(40,56),title,36,INK,True);text(d,(40,110),subtitle,18,MUTED);d.line((40,h-49,1460,h-49),fill=EDGE)
    text(d,(40,h-35),'REVIEW ONLY / STATIC 16 x 24 WORLD ART / NATIVE ILLUSTRATION AND LIVE PARTY UNCHANGED',13,MUTED);return im,d
notes=[['Bright fur and blocky robe.','The face reads more as a mask.'],['Broad muzzle and low gold spectacles.','Friendly front view; short folded robe.'],['A gently turned head and shoulders.','Three books, a tucked volume, warm fur.']]
im,d=page('A Librarian worth asking','Two authored world directions. Same 16 x 24 footprint, static pose and planted feet.',1110)
for i,key in enumerate(IDS):
    x=40+i*480;d.rounded_rectangle((x,155,x+460,800),12,fill=PANEL);text(d,(x+20,178),TITLES[i],25,GOLD if i else INK,True)
    put(im,CURRENT['frames'][key],(x+145,230),10);text(d,(x+20,493),'10x / nearest-neighbour inspection',16,MUTED)
    silhouette=pixels(CURRENT['frames'][key]);silhouette.putdata([(235,229,201,255) if silhouette.getpixel((xx,yy))[3] else (0,0,0,0) for yy in range(24) for xx in range(16)])
    put(im,silhouette,(x+35,552),5);text(d,(x+20,520),'Silhouette',16,GOLD);put(im,CURRENT['frames'][key],(x+275,574));text(d,(x+230,534),'Native 1x pixels',16,GOLD)
    for n,line in enumerate(notes[i]):text(d,(x+20,707+n*29),line,17)
reference=Image.open(Path.cwd()/'src/assets/librarian.png').convert('RGBA');reference.thumbnail((180,180),Image.Resampling.LANCZOS);im.paste(reference,(55,845),reference)
text(d,(265,850),'Keep the orangutan identity',25,GOLD,True)
text(d,(265,895),'The detailed Ledger illustration stays: warm rust fur, low spectacles, books, green waistcoat and purple robe.',18)
text(d,(265,931),'The chosen world direction will guide the independently authored 24 x 32 Ledger fallback.',18,MUTED)
text(d,(265,966),'No animation, new NPC behaviour, hit-region change or production art replacement in this study.',18,MUTED)
im.save(ROOT/'01-librarian-directions.png')
im,d=page('At home in the Hall','Actual canonical and compact Hall backgrounds; identical NPC placement and candidate half-block conversion.',1130)
for i,key in enumerate(IDS):
    x=40+i*480;text(d,(x,166),TITLES[i],25,GOLD if i else INK,True)
    for row,room in enumerate(['canonical','compact']):
        y=240+row*390;data=CURRENT if key=='current' else BGDATA;r=data['rooms'][room];src=pixels(r['options'][key]);xx,yy,_,_=r['bounds']
        xx=max(0,min(xx-12,src.width-44));yy=max(0,min(yy-8,src.height-40));crop=src.crop((xx,yy,xx+44,yy+40));put(im,crop,(x,y),8)
        text(d,(x,y-31),'Canonical Hall' if room=='canonical' else 'Compact Hall',18,MUTED)
text(d,(40,1010),'Placement studies only. The existing native illustration and independent Ledger fallback are retained until direction selection.',17,MUTED)
im.save(ROOT/'02-librarian-hall-context.png')
print('Wrote Librarian direction and Hall-context sheets.')
