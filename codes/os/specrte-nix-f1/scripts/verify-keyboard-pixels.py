#!/usr/bin/env python3
"""Compare an actual phone screenshot against an unchanged keyboard reference.
uv run --with pillow python scripts/verify-keyboard-pixels.py REFERENCE SCREENSHOT
Produces JSON only; never changes either image.
"""
import argparse
import json
from PIL import Image, ImageChops, ImageStat

p=argparse.ArgumentParser(description=__doc__)
p.add_argument('reference');p.add_argument('screenshot');p.add_argument('--top',type=int)
a=p.parse_args()
ref=Image.open(a.reference).convert('RGB');shot=Image.open(a.screenshot).convert('RGB')
if shot.width!=ref.width or shot.height<ref.height:
    raise SystemExit('Screenshot dimensions cannot contain the reference at native pixel size')
stride=ref.width*3;needle=ref.tobytes()[:20*stride];full=shot.tobytes()
candidates=[];start=0
while True:
    pos=full.find(needle,start)
    if pos<0:break
    if pos%stride==0 and pos//stride+ref.height<=shot.height:candidates.append(pos//stride)
    start=pos+stride
if a.top is not None:candidates=[a.top]
if not candidates:
    # Alignment fallback provides useful difference metrics for imperfect builds.
    points=[(x,y) for x in (0,50,200,400,700,1000,1079) for y in (0,25,80,176,350,479,600,705,755)]
    points=[(x,y) for x,y in points if x<ref.width and y<ref.height]
    samples=[(x,y,ref.getpixel((x,y))) for x,y in points]
    def error(top):return sum(sum(abs(v-w) for v,w in zip(rgb,shot.getpixel((x,top+y)))) for x,y,rgb in samples)
    candidates=[min(range(shot.height-ref.height+1),key=error)]
best=None
for top in candidates:
    if top<0 or top+ref.height>shot.height:raise SystemExit('Crop is outside the screenshot')
    diff=ImageChops.difference(ref,shot.crop((0,top,ref.width,top+ref.height)))
    different=sum(1 for rgb in diff.get_flattened_data() if any(rgb))
    stats=ImageStat.Stat(diff)
    result=dict(pixelPerfect=different==0,top=top,width=ref.width,height=ref.height,
        differentPixels=different,totalPixels=ref.width*ref.height,
        maxChannelDifference=max(hi for lo,hi in diff.getextrema()),
        meanAbsoluteChannelError=sum(stats.mean)/3)
    if best is None or result['differentPixels']<best['differentPixels']:best=result
    if not different:break
print(json.dumps(best,indent=2))
raise SystemExit(0 if best['pixelPerfect'] else 1)
