"""Reconstruct production Ratatui cells for keepsake review; no native capture."""
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
im=Image.new('RGB',(1510,1400),BG);d=ImageDraw.Draw(im)
text(d,(30,22),'Keepsakes / six saved companions',28,GOLD)
text(d,(30,65),'Production 8x8 art and current card cells. Visual review pending.',17,MUTED)
for i,entry in enumerate(DATA):
 x=30+(i%2)*750;y=115+(i//2)*365
 text(d,(x,y),entry['name'],21,GOLD)
 art=sprite(entry['sprite']);large=art.resize((48,48),Image.Resampling.NEAREST)
 im.paste(large,(x,y+37),large);im.paste(art,(x+64,y+57),art)
 # Existing detailed card at (41,2), 78x18 cells. Show at native cell ratio.
 card=terminal(entry['card']).crop((41*6,2*12,119*6,20*12))
 im.paste(card,(x+100,y+35))
 compact=terminal(entry['compact']).crop((11*6,2*12,59*6,14*12))
 # Compact card is shown separately at the foot, for the first two items.
 if i<2:
  text(d,(x,1140),'Compact / minimum 60x14 / ANSI-16',14,MUTED)
  im.paste(compact,(x,1160))
text(d,(30,1330),'Names and descriptions survive compact layouts; art stays in roomy cards.',16,MUTED)
text(d,(30,1365),'RGB + Ratatui reconstruction, not native terminal or portrait transport acceptance.',15,MUTED)
im.save(ROOT/'keepsakes.png')
