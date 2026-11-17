"""Assessment sheets from production RGB exports; no production changes."""
from pathlib import Path
import json
from PIL import Image, ImageDraw, ImageFont

ROOT = Path(__file__).resolve().parent
DATA = json.loads((ROOT/'rendered.json').read_text())
BG = '#151f27'
INK = '#f0e4cc'
MUTED = '#acbac7'
GOLD = '#dfb775'
CYAN = '#78d4dc'
FONT = '/System/Library/Fonts/Avenir Next.ttc'
SERIF = '/System/Library/Fonts/Supplemental/Georgia.ttf'

def text(d, xy, value, size=18, fill=INK, serif=False):
    d.text(xy, value, font=ImageFont.truetype(SERIF if serif else FONT, size), fill=fill)

def pixels(frame):
    im = Image.new('RGB', (frame['width'], frame['height']))
    im.putdata([tuple(p) for p in frame['pixels']])
    return im

def sample(cols, rows, party=5):
    return next(x for x in DATA if (x['columns'], x['rows'], x['party']) == (cols, rows, party))

def page(title, subtitle, height):
    im = Image.new('RGB', (1500, height), BG)
    d = ImageDraw.Draw(im)
    text(d, (40, 25), 'QUESTMANCER / HALL RESPONSIVENESS REVIEW / 17 NOVEMBER 2026', 15, CYAN)
    text(d, (40, 58), title, 35, INK, True)
    text(d, (40, 113), subtitle, 18, MUTED)
    d.line((40, height-48, 1460, height-48), fill='#405261')
    text(d, (40, height-35), 'ASSESSMENT ONLY / RGB FIXTURES / NATIVE FIT IS A VISUAL PROPOSAL, NOT LIVE TERMINAL EVIDENCE', 12, MUTED)
    return im, d

im, d = page('Smaller terminal, same room and character detail',
             'Five fixed adventurers plus Librarian. Current compact composition versus fitting the existing canonical RGB scene.', 1550)
text(d, (40, 162), 'Current half-block output', 27, GOLD, True)
text(d, (780, 162), 'Proposed native-image fit', 27, CYAN, True)
canonical = pixels(sample(160, 45))
for index, (cols, rows) in enumerate([(120, 34), (96, 28), (80, 24)]):
    y = 250 + index*390
    physical = (cols*8, rows*16)
    # Shared physical-cell assumption: 8x16 pixels. Display both panes at 1/2 size.
    left = pixels(sample(cols, rows)).resize(physical, Image.Resampling.NEAREST)
    right = Image.new('RGB', physical, '#10171c')
    scale = min(physical[0]//160, physical[1]//90)
    fitted = canonical.resize((160*scale, 90*scale), Image.Resampling.NEAREST)
    right.paste(fitted, ((physical[0]-fitted.width)//2, (physical[1]-fitted.height)//2))
    for x, panel in [(40, left), (780, right)]:
        im.paste(panel.resize((physical[0]//2, physical[1]//2), Image.Resampling.NEAREST), (x, y))
    text(d, (40, y-33), f'{cols} x {rows} cells / 16 x 24 sprites', 19, GOLD)
    text(d, (780, y-33), f'Same pane / {scale}x display pixels per source pixel', 19, CYAN)
    text(d, (40, y+285), f'Actor height: {24/(rows*2):.0%} of pane height', 18, MUTED)
    text(d, (780, y+285), f'Actor height: {24*scale/(rows*16):.0%}; source remains 16 x 24', 18, MUTED)
text(d, (40, 1450), 'Assumes 8x16 physical-pixel cells and integer image scaling. Text labels, cards, mouse mapping and live delivery are not demonstrated.', 16, MUTED)
im.save(ROOT/'01-current-versus-fit.png')

im, d = page('Where the existing Hall changes composition',
             'Actual production renders. All panels use 4 display pixels per RGB pixel; their different dimensions are intentional.', 1670)
for index, (cols, rows) in enumerate([(160,45),(160,44),(120,34),(96,28),(80,24),(60,20)]):
    frame = sample(cols, rows)
    x = 40+(index%2)*740
    y = 235+(index//2)*450
    tier = 'Canonical' if cols>=160 and rows>=45 else ('Roster' if cols<64 else 'Compact')
    text(d, (x,y-55), f'{cols} x {rows} / {tier}', 25, GOLD, True)
    text(d, (x,y-24), f"Actors {frame['actors'][0]['width']}x{frame['actors'][0]['height']}; visible party {len(frame['actors'])}/5", 17, MUTED)
    im.paste(pixels(frame).resize((cols*4, rows*8), Image.Resampling.NEAREST),(x,y))
text(d, (40, 1583), 'A one-row reduction at 160 columns replaces the full Hall. The small roster appears only after compact capacity or minimum size fails.', 17, MUTED)
im.save(ROOT/'02-current-breakpoints.png')

rows=[]
for f in DATA:
    sizes=sorted({(a['width'],a['height']) for a in f['actors']})
    rows.append({'party':f['party'],'columns':f['columns'],'rows':f['rows'],
                 'actor_sizes':sizes,'visible_actors':len(f['actors']),
                 'librarian_visible':f['interactables']>0,
                 'actor_height_fraction':max((a['height']/f['height'] for a in f['actors']),default=0)})
(ROOT/'measurements.json').write_text(json.dumps(rows,indent=2)+'\n')
print('Wrote current breakpoints, measured actor proportions and hypothetical native-fit comparison.')
