"""Review-only indexed sprite sheets; nearest-neighbour pixels, no production edits."""
from pathlib import Path
import json, hashlib
from PIL import Image, ImageDraw, ImageFont
ROOT=Path(__file__).resolve().parent
DATA=json.loads((ROOT/'rendered.json').read_text())
SOURCE=json.loads((ROOT/'candidates.json').read_text())
BG='#151f22'; PANEL='#202d30'; EDGE='#3a4b49'; INK='#f0e4c9'; MUTED='#acbdb3'; GOLD='#e4bc73'
FONT='/System/Library/Fonts/Avenir Next.ttc'; SERIF='/System/Library/Fonts/Supplemental/Georgia.ttf'
IDS=['current','grove','bramble']; TITLES=['Current master','A / Grove Keeper','B / Bramble Sage']
def text(d,xy,value,size=18,fill=INK,serif=False):
 d.text(xy,value,font=ImageFont.truetype(SERIF if serif else FONT,size),fill=fill)
def pixels(frame):
 im=Image.new('RGBA',(frame['width'],frame['height']))
 im.putdata([(*p,255) if p is not None else (0,0,0,0) for p in frame['pixels']]);return im
def put(target,src,xy,scale=1):
 if isinstance(src,dict):src=pixels(src)
 src=src.resize((src.width*scale,src.height*scale),Image.Resampling.NEAREST)
 target.paste(src,xy,src if src.mode=='RGBA' else None)
def page(title,subtitle,h):
 im=Image.new('RGB',(1500,h),BG);d=ImageDraw.Draw(im)
 text(d,(40,24),'QUESTMANCER / DRUID STUDY / 17 NOVEMBER 2026',15,GOLD)
 text(d,(40,54),title,36,INK,True);text(d,(40,107),subtitle,18,MUTED)
 d.line((40,h-49,1460,h-49),fill=EDGE)
 text(d,(40,h-36),'REVIEW ONLY / 16 x 24 AUTHORED PIXELS / PRODUCTION AND LIVE PARTY UNCHANGED',13,MUTED)
 return im,d
notes=[['Existing master and persona colouring.','Straight body, thin beard, broad dark feet.'],['Rounded leaf hood and fuller beard.','Warm bark, pale face, a soft cloak edge.'],['Turned face and asymmetric leaf cloak.','Right-hand staff, compact pouch and stance.']]
im,d=page('One Druid, two new directions','Static masters at 10x inspection scale. Candidate colours are authored study palettes.',1160)
for i,key in enumerate(IDS):
 x=40+i*480;d.rounded_rectangle((x,155,x+460,863),12,fill=PANEL)
 text(d,(x+20,176),TITLES[i],27,GOLD if i else INK,True)
 put(im,DATA['frames'][key],(x+145,230),10)
 text(d,(x+20,495),'10x / nearest-neighbour inspection',16,MUTED)
 text(d,(x+20,540),'Silhouette',16,GOLD)
 silhouette=pixels(DATA['frames'][key]);silhouette.putdata([(235,229,201,255) if p[3] else (0,0,0,0) for p in [silhouette.getpixel((x,y)) for y in range(silhouette.height) for x in range(silhouette.width)]])
 put(im,silhouette,(x+40,575),5)
 text(d,(x+230,540),'Native 1x pixels',16,GOLD);put(im,DATA['frames'][key],(x+275,585))
 text(d,(x+230,640),'Same 16 x 24 budget',16,MUTED)
 text(d,(x+230,669),'Feet remain on row 21',16,MUTED)
 for n,line in enumerate(notes[i]):text(d,(x+20,752+n*29),line,17)
portrait=Image.open(Path.cwd()/'src/assets/portraits/druid-card.png').convert('RGBA')
# Scale a supplied native reference for layout only. Candidate pixels are authored above.
portrait.thumbnail((150,200),Image.Resampling.LANCZOS)
im.paste(portrait,(52,900),portrait)
text(d,(245,911),'Card continuity',25,GOLD,True)
text(d,(245,954),'The existing card is the identity reference: living staff, leaf hood, pale beard and warm equipment.',18)
text(d,(245,989),'A favours a friendly front view. B explores a turned pose and stronger asymmetry.',18)
text(d,(245,1024),'Choose the shape and expression before authoring animation or changing the live Druid.',18,MUTED)
im.save(ROOT/'01-druid-directions.png')

im,d=page('Readability in both rooms','Identical production room backgrounds and actor positions; candidate placement studies, through the half-block adapter.',1120)
for i,key in enumerate(IDS):
 x=40+i*480;text(d,(x,163),TITLES[i],25,GOLD if i else INK,True)
 for j,room in enumerate(['hall','delve']):
  y=234+j*410;r=DATA['rooms'][room];bounds=r['bounds'];src=pixels(r['options'][key])
  # Unscaled 16x24 actor in a 52x44 logical pixel crop; whole crop enlarged evenly.
  xx=max(0,min(bounds[0]-18,src.width-52));yy=max(0,min(bounds[1]-9,src.height-44))
  crop=src.crop((xx,yy,xx+52,yy+44));put(im,crop,(x,y),8)
  text(d,(x,y-31),'Guild Hall' if room=='hall' else 'Delve',18,MUTED)
text(d,(40,1002),'These panels judge shape and contrast. Candidates are composited after room lighting; they are not activated ritual renders.',17,MUTED)
im.save(ROOT/'02-druid-room-crops.png')

im,d=page('Whole-room scale check','Room masters at 3x. Each Druid uses its original 16x24 pixel footprint; no live resource or overlay changes.',890)
for i,key in enumerate(IDS):
 x=20+i*490;text(d,(x+10,161),TITLES[i],24,GOLD if i else INK,True)
 for j,room in enumerate(['hall','delve']):
  y=230+j*305;put(im,DATA['rooms'][room]['options'][key],(x,y),3)
  text(d,(x+10,y-26),'Guild Hall' if room=='hall' else 'Delve',16,MUTED)
im.save(ROOT/'03-druid-full-rooms.png')
print('Wrote three Druid review sheets.')
