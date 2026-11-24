#!/usr/bin/env python3
"""Measure key-face bounds from the original artwork; does not modify images.
Run from the project root with: uv run --with pillow python tools/measure-keyboard-reference.py
"""
from PIL import Image
from collections import deque
from pathlib import Path
import json
result={}
for lang in ('english','persian'):
 im=Image.open('keyboard/references/'+lang+'.png').convert('RGB'); w,h=im.size
 px=im.load(); seen=set(); rects=[]
 def face(rgb):
  r,g,b=rgb;return rgb==(70,69,74) or (60<=r<=65 and 65<=g<=73 and 90<=b<=101)
 for y in range(25,605):
  for x in range(w):
   if (x,y) in seen or not face(px[x,y]):continue
   col=px[x,y];todo=[(x,y)];seen.add((x,y));lo=[x,y];hi=[x,y];n=0
   while todo:
    a,b=todo.pop();n+=1;lo[0]=min(lo[0],a);lo[1]=min(lo[1],b);hi[0]=max(hi[0],a);hi[1]=max(hi[1],b)
    for c,d in ((a-1,b),(a+1,b),(a,b-1),(a,b+1)):
     if 0<=c<w and 25<=d<605 and (c,d) not in seen and face(px[c,d]):seen.add((c,d));todo.append((c,d))
   if n>1000:rects.append([*lo,hi[0]-lo[0]+1,hi[1]-lo[1]+1])
 rows=[]
 for y in (25,176,327,479):rows.append(sorted([r for r in rects if abs(r[1]-y)<3]))
 assert [len(r) for r in rows] == ([10,9,9,6] if lang=='english' else [11,11,11,7])
 print(lang, 'key geometry measured');result[lang]=rows
Path('keyboard/reference-geometry.json').write_text(json.dumps(result,indent=2)+'\n')
header='/* Measured from the user-provided 1080x777 reference artwork. */\n'
for lang,rows in result.items():
 header+=f'static const int spectre_{lang}_counts[4] = '+str([len(r) for r in rows]).replace('[','{').replace(']','}')+';\n'
 header+=f'static const double spectre_{lang}_faces[4][11][4] = {{\n'
 for row in rows:header+='  {'+', '.join('{'+', '.join(map(str,r))+'}' for r in row)+'},\n'
 header+='};\n'
Path('keyboard/reference-geometry.h').write_text(header)
