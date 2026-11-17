"""A paired class-identity review; production sources are never replaced."""
from pathlib import Path
import json
from PIL import Image,ImageDraw,ImageFont
ROOT=Path(__file__).resolve().parent;DATA=json.loads((ROOT/'rendered.json').read_text())
BG='#151f27';PANEL='#22303c';EDGE='#405261';INK='#f0e4cc';MUTED='#acbac7';CYAN='#78d4dc';GOLD='#dfb775'
FONT='/System/Library/Fonts/Avenir Next.ttc';SERIF='/System/Library/Fonts/Supplemental/Georgia.ttf'
def text(d,xy,s,size=18,fill=INK,serif=False):d.text(xy,s,font=ImageFont.truetype(SERIF if serif else FONT,size),fill=fill)
def pixels(frame):
 im=Image.new('RGBA',(frame['width'],frame['height']));im.putdata([(*p,255) if p is not None else (0,0,0,0) for p in frame['pixels']]);return im
def put(im,src,xy,scale=1):
 if isinstance(src,dict):src=pixels(src)
 src=src.resize((src.width*scale,src.height*scale),Image.Resampling.NEAREST);im.paste(src,xy,src if src.mode=='RGBA' else None)
def page(title,sub,h):
 im=Image.new('RGB',(1500,h),BG);d=ImageDraw.Draw(im);text(d,(40,24),'QUESTMANCER / STORM CHANNELER / SORCERER IDENTITY STUDY',15,CYAN)
 text(d,(40,57),title,36,INK,True);text(d,(40,112),sub,18,MUTED);d.line((40,h-49,1460,h-49),fill=EDGE)
 text(d,(40,h-36),'17 NOVEMBER 2026 / REVIEW ONLY / NO PRODUCTION ASSET OR LIVE PARTY CHANGES',13,MUTED);return im,d
im,d=page('A Sorcerer with power in hand','Exact 16 x 24 authored pixels. Swept hair, a raised collar, an asymmetric mantle and a persistent orb.',1050)
columns=[('current','Current / working'),('storm_working','Storm / working'),('storm_resting','Storm / resting'),('storm_unknown','Storm / unknown')]
for i,(key,title) in enumerate(columns):
 x=40+i*365;d.rounded_rectangle((x,157,x+345,680),12,fill=PANEL);text(d,(x+18,179),title,23,GOLD if i==0 else CYAN,True)
 put(im,DATA['frames'][key],(x+92,234),10);text(d,(x+18,501),'10x / inspection',16,MUTED);put(im,DATA['frames'][key],(x+150,554));text(d,(x+107,603),'Native 1x pixels',14,MUTED)
text(d,(40,724),'The orb keeps the class recognisable while resting or unknown. Quiet poses imply no work or success.',21)
for i,(key,label) in enumerate([('current_resting','Current / resting'),('current_unknown','Current / unknown')]):
 x=55+i*300;put(im,DATA['frames'][key],(x,778),5);text(d,(x+104,808),label,17,MUTED)
text(d,(710,785),'No halo or ceremonial staff.',21,CYAN)
text(d,(710,828),'Contained magic; no ambient particle loop.',19,MUTED)
text(d,(710,866),'Saved persona colours remain an implementation contract.',18,MUTED)
im.save(ROOT/'01-storm-world-study.png')
im,d=page('Class recognition in both rooms','Current and proposed masters on the same production backgrounds and actor coordinates. Placement studies.',1100)
for i,(key,title) in enumerate([('current','Current Sorcerer'),('storm_working','Storm Channeler')]):
 x=45+i*745;text(d,(x,165),title,27,GOLD if i==0 else CYAN,True)
 for row,room in enumerate(['hall','delve']):
  y=234+row*370;r=DATA['rooms'][room];src=pixels(r['options'][key]);xx,yy,_,_=r['bounds'];xx=max(0,min(xx-16,src.width-56));yy=max(0,min(yy-8,src.height-42));put(im,src.crop((xx,yy,xx+56,yy+42)),(x,y),8)
  text(d,(x,y-30),'Guild Hall' if room=='hall' else 'Delve',18,MUTED)
text(d,(40,1002),'Candidate sprites are composited after room lighting and passed through the production half-block adapter.',18,MUTED)
im.save(ROOT/'02-storm-room-study.png')
card=ROOT/'storm-channeler-card-draft.png'
if card.exists():
 im,d=page('World and card, one class identity','New card draft generated with the built-in image tool. Existing production portrait is retained pending review.',1060)
 for x,path,title in [(60,Path.cwd()/'src/assets/portraits/sorcerer-card.png','Current native card'),(720,card,'Storm Channeler card draft')]:
  text(d,(x,171),title,28,GOLD if x==60 else CYAN,True);image=Image.open(path).convert('RGBA');image.thumbnail((600,650),Image.Resampling.LANCZOS);im.paste(image,(x+(600-image.width)//2,225),image)
 put(im,DATA['frames']['storm_working'],(1260,730),7)
 text(d,(40,944),'Review the face, mantle, orb and full-body silhouette together. No halo, pointed hat, book or staff.',19,MUTED)
 im.save(ROOT/'03-storm-card-study.png')
print('Wrote Storm world and room studies; paired card sheet when a draft is present.')
