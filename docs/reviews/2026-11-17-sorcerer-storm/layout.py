"""Production-rendered Sorcerer poses and room review; no Herdr or persistence."""
from pathlib import Path
import json
from PIL import Image,ImageDraw,ImageFont
ROOT=Path(__file__).resolve().parent;DATA=json.loads((ROOT/'rendered.json').read_text())
BG='#151f22';PANEL='#202d30';INK='#f0e4c9';MUTED='#acbdb3';GOLD='#e4bc73';EDGE='#3a4b49'
FONT='/System/Library/Fonts/Avenir Next.ttc';SERIF='/System/Library/Fonts/Supplemental/Georgia.ttf';MONO='/System/Library/Fonts/Menlo.ttc'
def text(d,xy,value,size=18,fill=INK,kind=FONT):d.text(xy,value,font=ImageFont.truetype(kind,size),fill=fill)
def pixels(data):
 im=Image.new('RGBA',(data['width'],data['height']));im.putdata([(*p,255) if p is not None else (0,0,0,0) for p in data['pixels']]);return im
def put(target,src,xy,scale=1):
 if isinstance(src,dict):src=pixels(src)
 src=src.resize((src.width*scale,src.height*scale),Image.Resampling.NEAREST);target.paste(src,xy,src if src.mode=='RGBA' else None)
def page(title,subtitle,h):
 im=Image.new('RGB',(1500,h),BG);d=ImageDraw.Draw(im);text(d,(40,24),'QUESTMANCER / STORM CHANNELER / PRODUCTION REVIEW',15,GOLD)
 text(d,(40,56),title,36,INK,SERIF);text(d,(40,110),subtitle,18,MUTED);d.line((40,h-48,1460,h-48),fill=EDGE)
 text(d,(40,h-35),'17 NOVEMBER 2026 / PRODUCTION RGB FIXTURES / NATIVE ACCEPTANCE REMAINS SEPARATE',13,MUTED);return im,d
POSES=[('work-a','Work A'),('work-b','Work B'),('signal','Counsel signal'),('wait','Counsel wait'),('spoils','Spoils held'),('settled','Settled'),('rest','Resting'),('unknown','Unknown')]
im,d=page('Storm Channeler, eight authored moments','The approved silhouette through the actual persona and ritual routes. All frames remain 16 x 24.',1010)
for i,(key,label) in enumerate(POSES):
 x=45+i*180;text(d,(x,170),label,18,GOLD);put(im,DATA['frames'][key],(x+12,220),8);put(im,DATA['frames'][key],(x+65,435));text(d,(x+50,475),'Native 1x',13,MUTED)
text(d,(40,545),'Saved skin tones retain the same silhouette',25,GOLD,SERIF)
for i,var in enumerate(DATA['variants']):
 x=65+i*230;put(im,var['frame'],(x,595),5);text(d,(x,735),var['skin'],17,MUTED)
text(d,(40,805),'Work: 500 ms frames. Counsel: 600 ms gesture, then still. Spoils: placed at 1000 ms, quiet by three seconds.',18)
text(d,(40,842),'Skin, hair, accent and small cloth trim use saved appearance. Silver streak and mantle remain class-owned.',18,MUTED)
im.save(ROOT/'01-production-poses.png')

def crop(world,key,scale=7):
 data=DATA['worlds'][world+'-'+key];src=pixels(data);x,y,_,_=data['bounds'];x=max(0,min(x-14,116));y=max(0,min(y-8,48));src=src.crop((x,y,x+44,y+42));return src.resize((44*scale,42*scale),Image.Resampling.NEAREST)
im,d=page('The Sorcerer at work in both rooms','Actual production lighting, grounding and semantic poses. Each crop is enlarged evenly from the rendered scene.',1230)
for col,(key,label) in enumerate([('work-a','Working'),('wait','Waiting for counsel'),('spoils','Returning with spoils'),('settled','Settled completion')]):
 x=40+col*365;text(d,(x,170),label,21,GOLD,SERIF)
 for row,world in enumerate(['hall','delve']):
  y=242+row*390;text(d,(x,y-31),'Guild Hall' if world=='hall' else 'Delve',18,MUTED);im.paste(crop(world,key),(x,y))
text(d,(40,1004),'The placed sealed pouch is a completion metaphor. It never verifies artifact contents or claims task success.',18,MUTED)
text(d,(40,1042),'Native card now uses Storm Channeler. Roster, animation scheduling and saved persona assignments are unchanged.',18,MUTED)
im.save(ROOT/'02-production-rooms.png')

def terminal(data):
 cw,ch=6,12;im=Image.new('RGB',(data['columns']*cw,data['rows']*ch));d=ImageDraw.Draw(im)
 for i,c in enumerate(data['cells']):
  x=(i%data['columns'])*cw;y=(i//data['columns'])*ch;d.rectangle((x,y,x+cw-1,y+ch-1),fill=tuple(c['bg']))
  if c['text']=='▀':d.rectangle((x,y,x+cw-1,y+ch//2-1),fill=tuple(c['fg']))
  elif c['text'].strip():text(d,(x,y-1),c['text'],10,tuple(c['fg']),MONO)
 return im
im,d=page('The same Sorcerer on the parchment','Actual Ratatui fallback card, reconstructed with Menlo. The native card uses the approved Storm Channeler illustration.',1000)
im.paste(terminal(DATA['card']),(40,170));put(im,DATA['fallback'],(1100,200),8);text(d,(1080,485),'24 x 32 fallback canvas',18,GOLD);text(d,(1080,520),'Unscaled 16 x 24 master',17,MUTED)
put(im,DATA['roster'],(1110,590),8);text(d,(1080,710),'Existing roster retained',17,MUTED)
text(d,(40,815),'Fallback reuses the personalised Working A sprite, centred with four-pixel margins.',20)
im.save(ROOT/'03-card-fallback.png')
frames=[]
for key in ['work-a','work-b']:
 im=Image.new('RGB',(900,445),BG);d=ImageDraw.Draw(im);text(d,(30,22),'STORM CHANNELER / PRODUCTION WORKING LOOP / 500 ms PER FRAME',16,GOLD)
 for i,world in enumerate(['hall','delve']):text(d,(30+i*450,65),'Guild Hall' if world=='hall' else 'Delve',22,INK,SERIF);im.paste(crop(world,key,8),(30+i*450,110))
 frames.append(im)
frames[0].save(ROOT/'working-loop.gif',save_all=True,append_images=frames[1:],duration=500,loop=0,disposal=2)
print('Wrote production pose, room, card and working-loop reviews.')

# Embedded native card: exact production asset, composited for review only.
im,d=page('Storm Channeler, world and native card','Exact embedded PNG shown with the actual personalised fallback. Native terminal appearance remains separate.',1000)
card=Image.open(ROOT.parents[2]/'src/assets/portraits/sorcerer-card.png')
im.paste(card,(170,185),card);put(im,DATA['fallback'],(830,260),12)
text(d,(170,745),'384 x 512 embedded native card',22,GOLD)
text(d,(830,690),'24 x 32 fallback canvas',22,GOLD)
text(d,(40,825),'Native illustration is class-selected. The fallback reuses saved skin, hair, accent and trim roles.',19,MUTED)
im.save(ROOT/'05-native-and-fallback.png')

# Exact canonical snapshot from tests/scene_delve.rs, exported by golden.rs.
golden=json.loads((ROOT/'canonical-delve.json').read_text())
put_image=pixels(golden).convert('RGB')
put_image.resize((960,540),Image.Resampling.NEAREST).save(ROOT/'04-canonical-delve.png')
