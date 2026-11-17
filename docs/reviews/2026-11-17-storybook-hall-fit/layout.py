"""Actual fit projection frames with annotated text; not terminal screenshots."""
from pathlib import Path
import json
from PIL import Image, ImageDraw, ImageFont

ROOT=Path(__file__).resolve().parent
DATA=json.loads((ROOT/'renders.json').read_text())
BG='#151f27';INK='#f0e4cc';MUTED='#acbac7';GOLD='#dfb775'
FONT='/System/Library/Fonts/Avenir Next.ttc'
SERIF='/System/Library/Fonts/Supplemental/Georgia.ttf'
MONO='/System/Library/Fonts/Menlo.ttc'
def text(d,xy,value,size=18,colour=INK,font=FONT):
    d.text(xy,value,font=ImageFont.truetype(font,size),fill=colour)
sheet=Image.new('RGB',(2050,1840),BG);d=ImageDraw.Draw(sheet)
text(d,(40,25),'QUESTMANCER / IMPLEMENTED STORYBOOK HALL FIT / 17 NOVEMBER 2026',17,GOLD)
text(d,(40,65),'Complete Hall, preserved source pixels',42,INK,SERIF)
text(d,(40,124),'Actual FitScene projections at the native output sizes, with caption text reconstructed for review. Fixed 8x16 cell geometry.',20,MUTED)
for col,party in enumerate(['five','eleven']):
    x=40+col*1020
    text(d,(x,178),'Five adventurers' if party=='five' else 'Eleven adventurers',30,GOLD,SERIF)
    for row,(cols,rows) in enumerate([(120,34),(96,28),(80,24)]):
        data=next(v for v in DATA if v['file']==f'fit-{party}-{cols}x{rows}.png')
        frame=Image.new('RGB',(cols*8,rows*16),'#0c1016');fd=ImageDraw.Draw(frame)
        image=Image.open(ROOT/data['file']).convert('RGB')
        px,py,_,_=data['image_cells'];frame.paste(image,(px*8,py*16))
        text(fd,(1,0),f"Kitty fit {data['scale']}x / {data['actors']}/{data['actors']} adventurers",13,GOLD,MONO)
        text(fd,(1,(rows-2)*16),'fit 00 / Druid / Working / 1 need counsel',13,INK,MONO)
        text(fd,(1,(rows-1)*16),'Tab adventurer / c card / L Ledger / Esc back',13,INK,MONO)
        y=[255,870,1390][row]
        text(d,(x,y-32),f"{cols}x{rows} cells / integer scale {data['scale']} / all {data['actors']} actors + Librarian",19,MUTED)
        sheet.paste(frame,(x,y))
text(d,(40,1800),'SOURCE PIXELS VERIFIED / IMAGE TRANSPORT TESTED WITH SYNTHETIC PTY / GHOSTTY VISUAL ACCEPTANCE PENDING',15,MUTED)
sheet.save(ROOT/'01-storybook-fit-review.png')
print('Wrote fitted Hall review sheet from the actual prototype projection.')
