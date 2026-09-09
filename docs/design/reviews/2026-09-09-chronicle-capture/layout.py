"""Arrange actual production Ratatui cells; not native terminal capture."""
import json,sys
from pathlib import Path
from PIL import Image,ImageDraw,ImageFont
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
im=Image.new('RGB',(1540,1650),BG);d=ImageDraw.Draw(im)
text(d,(30,24),'Chronicle capture / observed facts and retained records',28,GOLD)
text(d,(30,74),'Tab requests a last-hour guild chapter; Tab returns to the existing selected record list.',16,MUTED)
for i,(key,title) in enumerate([('records','c / existing selected-adventurer records'),('chapter','Tab / last-hour guild chapter'),('sources','j / source entries below the recap'),('empty','A later window / no retained events'),('compact','80 columns / chapter'),('narrow','40 columns / existing clipped chapter contract')]):
 x=30+(i%2)*770;y=126+(i//2)*492
 text(d,(x,y),title,18,GOLD);im.paste(terminal(DATA[key]),(x,y+35))
text(d,(30,1600),'Production Ratatui reconstruction. Fixed fixture names and times; visual review pending.',16,MUTED)
im.save(ROOT/'chronicle-capture.png')
for key,data in DATA.items():
 terminal(data).resize((data['columns']*12,data['rows']*24),Image.Resampling.NEAREST).save(ROOT/(key+'.png'))
