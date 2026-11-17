"""Byte-exact native upload comparison, using only Python's standard library."""
from pathlib import Path
import base64, re, json, hashlib, struct, zlib, binascii
ROOT=Path(__file__).resolve().parent

def png_rgb(path):
    data=path.read_bytes();assert data[:8]==b'\x89PNG\r\n\x1a\n'
    offset=8;compressed=bytearray();header=None
    while offset<len(data):
        length=struct.unpack('>I',data[offset:offset+4])[0]
        kind=data[offset+4:offset+8];chunk=data[offset+8:offset+8+length]
        crc=struct.unpack('>I',data[offset+8+length:offset+12+length])[0]
        assert binascii.crc32(kind+chunk)&0xffffffff==crc
        if kind==b'IHDR':header=struct.unpack('>IIBBBBB',chunk)
        elif kind==b'IDAT':compressed.extend(chunk)
        offset+=12+length
    w,h,depth,colour,compression,filtering,interlace=header
    assert (depth,colour,compression,filtering,interlace)==(8,2,0,0,0)
    raw=zlib.decompress(compressed);stride=w*3;previous=bytearray(stride);rgb=bytearray()
    def paeth(a,b,c):
        p=a+b-c;distances=[abs(p-a),abs(p-b),abs(p-c)]
        return [a,b,c][distances.index(min(distances))]
    for y in range(h):
        start=y*(stride+1);method=raw[start];row=bytearray(raw[start+1:start+1+stride])
        for i in range(stride):
            a=row[i-3] if i>=3 else 0;b=previous[i];c=previous[i-3] if i>=3 else 0
            assert method in range(5)
            prediction=[0,a,b,(a+b)//2,paeth(a,b,c)][method]
            row[i]=(row[i]+prediction)&255
        rgb.extend(row);previous=row
    return (w,h),b''.join(bytes(rgb[i:i+3])+b'\xff' for i in range(0,len(rgb),3))

def png_chunk(kind,data):
    return struct.pack('>I',len(data))+kind+data+struct.pack('>I',binascii.crc32(kind+data)&0xffffffff)

def main():
    raw=Path('/private/tmp/questmancer-production-hall-fit-20261117/native.ansi').read_bytes()
    current=None;first=None
    for match in re.finditer(rb'\x1b_G([^;]*);([^\x1b]*)\x1b\\',raw):
        controls=dict(part.split('=',1) for part in match[1].decode().split(',') if '=' in part)
        if controls.get('a')=='T':current={'controls':controls,'chunks':[]}
        if current is not None and 'm' in controls:
            current['chunks'].append(match[2])
            if controls['m']=='0':first=current;break
    assert first is not None
    controls=first['controls'];assert controls['f']=='32' and controls['U']=='1'
    size=(int(controls['s']),int(controls['v']))
    pixels=base64.b64decode(b''.join(first['chunks']),validate=True)
    expected_size,expected=png_rgb(ROOT/'expected-empty-hall.png')
    assert size==expected_size and pixels==expected,'native transport changed projected pixels'
    w,h=size;scan=b''.join(b'\0'+pixels[y*w*4:(y+1)*w*4] for y in range(h))
    png=b'\x89PNG\r\n\x1a\n'+png_chunk(b'IHDR',struct.pack('>IIBBBBB',w,h,8,6,0,0,0))+png_chunk(b'IDAT',zlib.compress(scan))+png_chunk(b'IEND',b'')
    (ROOT/'02-pty-world-upload.png').write_bytes(png)
    receipt=dict(scope='Production release binary native-transport pixel round trip with synthetic PTY replies',
        dimensions=size,image_id=controls['i'],all_rgba_pixels_identical_to_fit_projection=True,
        rgba_sha256=hashlib.sha256(pixels).hexdigest(),
        limitation='Producer verified; terminal display, animation, resize and mouse acceptance remain human checks')
    (ROOT/'transport-pixels.json').write_text(json.dumps(receipt,indent=2)+'\n')
    print('PASS: every native RGBA pixel matches the production projection at',size)

if __name__=='__main__':main()
