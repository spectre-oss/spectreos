#!/usr/bin/env python3
"""Generate compact static Beer CSS shape meshes. Requires svg.path 7 and earcut 3.

Run after cloning Beer CSS to context/beercss and installing svg.path into
context/build/svgpath and earcut into context/build/shape-gen. Runtime has no
Python/Node dependency. The generated paths are derived from Beer CSS (MIT).
"""
from pathlib import Path
import json, math, re, subprocess, sys, xml.etree.ElementTree as ET
ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / 'context/build/svgpath'))
from svg.path import parse_path
from svg.path.path import Move, Line, Close
source = ROOT / 'context/beercss/src/cdn/shapes'
shapes=[]
for path in sorted(source.glob('*.svg')):
    root=ET.parse(path).getroot()
    element=next((e for e in root.iter() if e.tag.endswith('path') and e.get('d')),None)
    if element is None: continue # animated loading indicator has no static path
    view=root.get('viewBox')
    if view: x,y,w,h=map(float,view.split())
    else: x=y=0.;w=float(root.get('width'));h=float(root.get('height'))
    extent=max(w,h); ox=(extent-w)/2; oy=(extent-h)/2
    contours=[]; points=[]
    def point(z):return [round((z.real-x+ox)/extent,6),round((z.imag-y+oy)/extent,6)]
    def flush():
        if len(points)>1 and points[-1]==points[0]:points.pop()
        if len(points)>=3:contours.append(points.copy())
        points.clear()
    for seg in parse_path(element.get('d')):
        if isinstance(seg,Move):
            flush(); points.append(point(seg.end)); continue
        steps=1 if isinstance(seg,(Line,Close)) else 8
        for i in range(1,steps+1):
            p=point(seg.point(i/steps))
            if not points or p!=points[-1]:points.append(p)
    flush()
    if not contours: raise ValueError(f'No contour: {path}')
    shapes.append({'name':path.stem,'contours':contours})
script='''import earcut from "./node_modules/earcut/src/earcut.js";let s="";for await(const c of process.stdin)s+=c;const shapes=JSON.parse(s);for(const shape of shapes){let flat=[],holes=[];for(let i=0;i<shape.contours.length;i++){if(i)holes.push(flat.length/2);for(const p of shape.contours[i])flat.push(...p)}shape.triangles=earcut(flat,holes,2)}process.stdout.write(JSON.stringify(shapes));'''
helper=ROOT/'context/build/shape-gen/generate.mjs';helper.write_text(script)
shapes=json.loads(subprocess.check_output(['node',str(helper)],input=json.dumps(shapes),text=True))
out=['//! Generated from Beer CSS SVG shapes by tools/generate-beercss-shapes.py.', '//! Copyright (c) 2021 Beer css!; MIT license, see THIRD_PARTY_NOTICES.md.', '//! Shape documentation credits Daria Dororoshchuk for SVG artwork.', '#[derive(Clone, Copy)]','pub struct ShapeData { pub name: &\'static str, pub points: &\'static [[f32; 2]], pub contours: &\'static [u16], pub triangles: &\'static [u16] }','#[rustfmt::skip]','pub static SHAPES: &[ShapeData] = &[']
for s in shapes:
    def area(contour):
        return abs(sum(a[0]*b[1]-a[1]*b[0] for a,b in zip(contour,contour[1:]+contour[:1])))/2
    expected=area(s['contours'][0])-sum(area(c) for c in s['contours'][1:])
    points_all=[p for contour in s['contours'] for p in contour]
    actual=sum(area([points_all[a],points_all[b],points_all[c]]) for a,b,c in zip(s['triangles'][::3],s['triangles'][1::3],s['triangles'][2::3]))
    if abs(actual-expected)>max(1e-5,expected*0.001):raise ValueError(f"Invalid tessellation: {s['name']} {actual} vs {expected}")
    points=[p for contour in s['contours'] for p in contour]
    if len(points)>=65536:raise ValueError(s['name'])
    starts=[];n=0
    for contour in s['contours']:starts.append(n);n+=len(contour)
    starts.append(n)
    def num(n):
        t=f'{n:.6f}'.rstrip('0').rstrip('.');return (t if '.' in t else t+'.0')
    out.append('ShapeData { name: '+json.dumps(s['name'])+', points: &['+','.join('['+num(x)+','+num(y)+']' for x,y in points)+'], contours: &['+','.join(map(str,starts))+'], triangles: &['+','.join(map(str,s['triangles']))+'] },')
out.append('];')
dest=ROOT/'codes/libs/ui/libbhoo/src/shapes/beercss_shapes.rs';dest.write_text('\n'.join(out)+'\n')
reference=ROOT/'codes/apps/bhoo-web/site/beercss-shapes.js';reference.write_text('export const names='+json.dumps([s['name'] for s in shapes],separators=(',',':'))+';\n')
print(f'Generated {len(shapes)} shape meshes; {dest.stat().st_size:,} source bytes')
