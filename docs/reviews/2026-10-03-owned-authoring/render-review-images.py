from PIL import Image,ImageDraw,ImageFont
import pathlib,json,re
root=pathlib.Path('docs/reviews/2026-10-03-owned-authoring/visuals')
font=ImageFont.truetype('/System/Library/Fonts/Menlo.ttc',16)
basic=[(0,0,0),(128,0,0),(0,128,0),(128,128,0),(0,0,128),(128,0,128),(0,128,128),(192,192,192),(128,128,128),(255,0,0),(0,255,0),(255,255,0),(0,0,255),(255,0,255),(0,255,255),(255,255,255)]
colours={'Reset':(0,0,0),'Black':basic[0],'Red':basic[1],'Green':basic[2],'Yellow':basic[3],'Blue':basic[4],'Magenta':basic[5],'Cyan':basic[6],'Gray':basic[7],'DarkGray':basic[8],'LightRed':basic[9],'LightGreen':basic[10],'LightYellow':basic[11],'LightBlue':basic[12],'LightMagenta':basic[13],'LightCyan':basic[14],'White':basic[15]}
def rgb(value):
 if value in colours:return colours[value]
 if value.startswith('Rgb('):return tuple(map(int,re.findall(r'\d+',value)))
 if value.startswith('Indexed('):
  i=int(re.findall(r'\d+',value)[0])
  if i<16:return basic[i]
  if i>=232:return (8+(i-232)*10,)*3
  n=i-16;axis=[0,95,135,175,215,255];return axis[n//36],axis[(n%36)//6],axis[n%6]
 raise ValueError(value)
for p in root.glob('*.json'):
 doc=json.loads(p.read_text());im=Image.new('RGB',(doc['width']*10,doc['height']*20));draw=ImageDraw.Draw(im)
 for cell in doc['cells']:
  x=cell['x']*10;y=cell['y']*20;bg=rgb(cell['bg']);fg=rgb(cell['fg']);draw.rectangle((x,y,x+9,y+19),fill=bg)
  if cell['text']=='▀':draw.rectangle((x,y,x+9,y+9),fill=fg)
  elif cell['text']!=' ':draw.text((x,y-1),cell['text'],font=font,fill=fg)
 im.save(p.with_suffix('.png'))
print('Rendered production cell buffers with Menlo; fixture QA, not native screenshots.')
