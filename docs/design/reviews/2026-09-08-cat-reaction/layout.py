"""Arrange production RGB scene frames; this is not native terminal capture."""
import json,sys
from pathlib import Path
from PIL import Image,ImageDraw,ImageFont
ROOT=Path(__file__).resolve().parent
DATA=json.loads(Path(sys.argv[1]).read_text())
BG='#181c24';INK='#e7ddc5';GOLD='#e5b95c';MUTED='#acb4c2'
def label(d,xy,text,size=18,fill=INK):
 d.text(xy,text,font=ImageFont.truetype('/System/Library/Fonts/Menlo.ttc',size),fill=fill)
def scene(data):
 im=Image.new('RGB',(data['width'],data['height']));im.putdata([tuple(p) for p in data['pixels']]);return im
im=Image.new('RGB',(1160,1000),BG);d=ImageDraw.Draw(im)
label(d,(30,25),'A quiet guild / one cat reaction',30,GOLD)
label(d,(30,74),'The same known party becomes entirely resting. No claim of completed work.',17,MUTED)
frames=[scene(DATA['before'])]+[scene(item['scene']) for item in DATA['frames'][:3]]
labels=['Before / working','0 ms / head lifts','400 ms / stretch','800 ms / settled']
for i,(frame,title) in enumerate(zip(frames,labels)):
 x=30+i*280;label(d,(x,122),title,17,GOLD)
 crop=frame.crop((86,17,100,26));im.paste(crop.resize((224,144),Image.Resampling.NEAREST),(x,162))
 im.paste(crop,(x,320))
label(d,(30,365),'Canonical Guild Hall / reaction begins',20,GOLD)
im.paste(frames[1].resize((960,540),Image.Resampling.NEAREST),(30,408))
label(d,(30,967),'Production RGB export. Cat reservation stays 10x5; visual approval pending.',16,MUTED)
im.save(ROOT/'cat-reaction.png')
# Play the event once: no loop extension, then retain the sleeping frame.
playback=[]
for frame,title in zip(frames,labels):
 page=Image.new('RGB',(960,590),BG);draw=ImageDraw.Draw(page)
 label(draw,(20,15),title,22,GOLD)
 page.paste(frame.resize((960,540),Image.Resampling.NEAREST),(0,50));playback.append(page)
playback[0].save(ROOT/'cat-reaction.gif',save_all=True,append_images=playback[1:],duration=[1000,400,400,2000],disposal=2)
