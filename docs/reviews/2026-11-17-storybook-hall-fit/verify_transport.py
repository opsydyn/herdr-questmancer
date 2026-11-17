"""Decode the executable's first native upload and compare every pixel to its projection."""
from pathlib import Path
import base64, re, json, hashlib
from PIL import Image

ROOT=Path(__file__).resolve().parent
raw=Path('/private/tmp/questmancer-storybook-fit-20261117/native.ansi').read_bytes()
current=None
first=None
for match in re.finditer(rb'\x1b_G([^;]*);([^\x1b]*)\x1b\\',raw):
    controls=dict(part.split('=',1) for part in match[1].decode().split(',') if '=' in part)
    if controls.get('a')=='T':
        current={'controls':controls,'chunks':[]}
    if current is not None and 'm' in controls:
        current['chunks'].append(match[2])
        if controls['m']=='0':first=current;break
assert first is not None
controls=first['controls'];assert controls['f']=='32' and controls['U']=='1'
size=(int(controls['s']),int(controls['v']))
pixels=base64.b64decode(b''.join(first['chunks']),validate=True)
assert len(pixels)==size[0]*size[1]*4
actual=Image.frombytes('RGBA',size,pixels)
expected=Image.open(ROOT/'fit-five-120x34.png').convert('RGBA')
assert actual.size==expected.size and actual.tobytes()==expected.tobytes(), 'native transport changed projected pixels'
actual.save(ROOT/'02-pty-world-upload.png')
receipt={'scope':'Executable native-transport pixel round trip with synthetic PTY replies',
         'dimensions':size,'image_id':controls['i'],'all_rgba_pixels_identical_to_fit_projection':True,
         'rgba_sha256':hashlib.sha256(pixels).hexdigest(),
         'limitation':'Transport producer verified; actual terminal placement, display and interaction remain human acceptance'}
(ROOT/'transport-pixels.json').write_text(json.dumps(receipt,indent=2)+'\n')
print('PASS: native RGBA payload exactly matches every projected image pixel at',size)
