#!/usr/bin/env python3
"""Generate sampled, tessellated Material Icons from the local MUI SVG snapshot.

Requires context/build/svgpath and context/build/shape-gen/node_modules/earcut.
The SVG geometry is Google's Material Icons (Apache-2.0); see THIRD_PARTY_NOTICES.md.
"""
from pathlib import Path
import json, subprocess, sys, xml.etree.ElementTree as ET
ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / 'context/build/svgpath'))
from svg.path import parse_path
from svg.path.path import Move, Line, Close
source = ROOT / 'codes/libs/ui/libbhoo/assets/material-icons'
names = ['home', 'home_outlined', 'folder', 'folder_outlined', 'settings', 'settings_outlined', 'alarm', 'alarm_outlined', 'access_time_filled', 'schedule', 'hourglass_full', 'hourglass_empty', 'timer', 'timer_outlined', 'menu', 'menu_open', 'arrow_back', 'arrow_forward', 'search', 'more_vert', 'add', 'tab']
shapes = []
for name in names:
    root = ET.parse(source / (name + '_24px.svg')).getroot()
    paths = [e.get('d') for e in root.iter() if e.tag.endswith('path') and e.get('d') and e.get('fill') != 'none']
    contours, points = [], []
    def flush():
        if len(points) > 1 and points[-1] == points[0]: points.pop()
        if len(points) >= 3: contours.append(points.copy())
        points.clear()
    for path in paths:
        for seg in parse_path(path):
            if isinstance(seg, Move):
                flush(); points.append([round(seg.end.real/24,6),round(seg.end.imag/24,6)]); continue
            for i in range(1, (1 if isinstance(seg,(Line,Close)) else 8)+1):
                steps = 1 if isinstance(seg,(Line,Close)) else 8
                p=seg.point(i/steps); v=[round(p.real/24,6),round(p.imag/24,6)]
                if not points or points[-1] != v: points.append(v)
        flush()
    for e in root.iter():
        if e.tag.endswith('rect') and e.get('fill') != 'none':
            x=float(e.get('x','0')); y=float(e.get('y','0')); w=float(e.get('width','0')); h=float(e.get('height','0'))
            if w > 0 and h > 0: contours.append([[round(x/24,6),round(y/24,6)],[round((x+w)/24,6),round(y/24,6)],[round((x+w)/24,6),round((y+h)/24,6)],[round(x/24,6),round((y+h)/24,6)]])
    shapes.append({'name': name, 'contours': contours})
# Nesting parity identifies disjoint filled contours and their holes.
script='''import earcut from "./node_modules/earcut/src/earcut.js";let s="";for await(const c of process.stdin)s+=c;const shapes=JSON.parse(s);function inside(p,poly){let c=false;for(let i=0,j=poly.length-1;i<poly.length;j=i++){let a=poly[i],b=poly[j];if((a[1]>p[1])!==(b[1]>p[1])&&p[0]<(b[0]-a[0])*(p[1]-a[1])/(b[1]-a[1])+a[0])c=!c;}return c}for(const shape of shapes){let cs=shape.contours,starts=[],n=0;for(const c of cs){starts.push(n);n+=c.length}let parents=cs.map((c,i)=>cs.map((x,j)=>j!==i&&inside(c[0],x)?j:-1).filter(j=>j>=0));let tris=[];for(let i=0;i<cs.length;i++){if(parents[i].length%2)continue;let group=[i,...cs.map((_,j)=>j).filter(j=>parents[j].length===parents[i].length+1&&parents[j].includes(i))];let flat=[],holes=[],map=[];for(let k=0;k<group.length;k++){if(k)holes.push(map.length);let g=group[k];for(let j=0;j<cs[g].length;j++){flat.push(...cs[g][j]);map.push(starts[g]+j)}}tris.push(...earcut(flat,holes,2).map(j=>map[j]))}shape.triangles=tris}process.stdout.write(JSON.stringify(shapes));'''
helper=ROOT/'context/build/shape-gen/material-icons.mjs';helper.write_text(script)
shapes=json.loads(subprocess.check_output(['node',str(helper)],input=json.dumps(shapes),text=True))
out=['//! Generated from Google Material Icons; see THIRD_PARTY_NOTICES.md.', 'use crate::beercss_shapes::ShapeData;', '#[rustfmt::skip]', 'pub static ICONS: &[ShapeData] = &[']
def num(x):
    s=f'{x:.6f}'.rstrip('0').rstrip('.'); return s if '.' in s else s+'.0'
for shape in shapes:
    points=[p for c in shape['contours'] for p in c]; starts=[]; n=0
    for c in shape['contours']: starts.append(n); n+=len(c)
    starts.append(n)
    if not shape['triangles']: raise ValueError(shape['name'])
    out.append('ShapeData { name: '+json.dumps(shape['name'])+', points: &['+','.join('['+num(x)+','+num(y)+']' for x,y in points)+'], contours: &['+','.join(map(str,starts))+'], triangles: &['+','.join(map(str,shape['triangles']))+'] },')
out.append('];')
dest=ROOT/'codes/libs/ui/libbhoo/src/icons/material_navigation.rs';dest.write_text('\n'.join(out)+'\n')
print('Generated',len(shapes),'icons;',dest.stat().st_size,'bytes')
