"""Arrange production RGB pixels and reconstructed Ratatui cells for review."""
import json,sys
from pathlib import Path
from PIL import Image, ImageDraw, ImageFont
ROOT=Path(__file__).resolve().parent
DATA=json.loads(Path(sys.argv[1]).read_text())
BG='#181c24'; INK='#e7ddc5'; MUTED='#acb4c2'; GOLD='#e5b95c'
FONT='/System/Library/Fonts/Menlo.ttc'
def text(draw,xy,value,size=18,colour=INK):
 draw.text(xy,value,font=ImageFont.truetype(FONT,size),fill=colour)
def sprite(data):
 im=Image.new('RGBA',(data['width'],data['height']))
 im.putdata([tuple(p)+(255,) if p is not None else (0,0,0,0) for p in data['pixels']]);return im

def terminal(data):
 cw,ch=6,12
 im=Image.new('RGB',(data['columns']*cw,data['rows']*ch));d=ImageDraw.Draw(im)
 for i,c in enumerate(data['cells']):
  x=i%data['columns']*cw;y=i//data['columns']*ch
  d.rectangle((x,y,x+cw-1,y+ch-1),fill=tuple(c['bg']))
  if c['text']=='▀':d.rectangle((x,y,x+cw-1,y+ch//2-1),fill=tuple(c['fg']))
  elif c['text'].strip():text(d,(x,y-1),c['text'],10,tuple(c['fg']))
 return im
im=Image.new('RGB',(1600,1510),BG);d=ImageDraw.Draw(im)
text(d,(40,26),'Campaign heraldry / production review',30,GOLD)
text(d,(40,72),'7x8 pennants; fixed workspace-key identity; visual sign-off pending.',18,MUTED)
for i,crest in enumerate(DATA['crests']):
 col=i%4;row=i//4;x=40+col*390;y=132+row*84
 badge=sprite(crest['sprite']);im.paste(badge.resize((42,48),Image.Resampling.NEAREST),(x,y),badge.resize((42,48),Image.Resampling.NEAREST))
 text(d,(x+60,y+6),crest['name'],16)
 im.paste(badge,(x+60,y+36),badge)
text(d,(40,490),'Hall / current Storybook party',21,GOLD)
text(d,(830,490),'Shared tables / eight campaign crests',21,GOLD)
for key,x in [('hall',40),('shared',830)]:
 data=DATA[key];world=Image.new('RGB',(data['width'],data['height']));world.putdata([tuple(p) for p in data['pixels']]);im.paste(world.resize((640,360),Image.Resampling.NEAREST),(x,530))
text(d,(40,907),'Pennants sit below adventurer rows; actors keep their full size and targets.',17,MUTED)
text(d,(40,953),'Parchment / Unicode + RGB',21,GOLD)
text(d,(830,953),'Parchment / ASCII + ANSI-16',21,GOLD)
im.paste(terminal(DATA['card']),(40,993));im.paste(terminal(DATA['ascii']),(830,993))
text(d,(40,1451),'Production pixels and Ratatui cell reconstruction; not a native terminal screenshot.',17,MUTED)
im.save(ROOT/'campaign-heraldry.png')
print(ROOT/'campaign-heraldry.png')
