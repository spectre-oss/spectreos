#!/usr/bin/env python3
"""Generate AnalogClock meshes from the pinned Apache-2.0 AOSP vectors.

Requires svg.path and earcut in context/build, like generate-beercss-shapes.py.
"""
from pathlib import Path
import json, subprocess, sys, xml.etree.ElementTree as ET
ROOT=Path(__file__).resolve().parents[1]
sys.path.insert(0,str(ROOT/'context/build/svgpath'))
from svg.path import parse_path
from svg.path.path import Move,Line,Close
source=ROOT/'codes/libs/ui/libbhoo/assets/android-analog-clock'
shapes=[]
for name in ('clock_dial','clock_hand_hour','clock_hand_minute'):
    root=ET.parse(source/(name+'.xml')).getroot()
    element=root.find('path')
    if element is None:raise ValueError('Missing vector path: '+name)
    path=element.attrib['{http://schemas.android.com/apk/res/android}pathData']
    contours=[];points=[]
    def point(z):return [round(z.real/380,6),round(z.imag/380,6)]
    def flush():
        if len(points)>1 and points[-1]==points[0]:points.pop()
        if len(points)>=3:contours.append(points.copy())
        points.clear()
    for segment in parse_path(path):
        if isinstance(segment,Move):flush();points.append(point(segment.end));continue
        steps=1 if isinstance(segment,(Line,Close)) else 12
        for i in range(1,steps+1):
            p=point(segment.point(i/steps))
            if not points or p!=points[-1]:points.append(p)
    flush()
    if len(contours)!=1:raise ValueError('Expected one contour: '+name)
    shapes.append({'name':'aosp-'+name.replace('_','-'),'contours':contours})
helper=ROOT/'context/build/shape-gen/android-clock.mjs'
helper.write_text("""import earcut from './node_modules/earcut/src/earcut.js';let s='';for await(const c of process.stdin)s+=c;const shapes=JSON.parse(s);for(const shape of shapes){const flat=shape.contours[0].flat();shape.triangles=earcut(flat,null,2)}process.stdout.write(JSON.stringify(shapes));""")
shapes=json.loads(subprocess.check_output(['node',str(helper)],input=json.dumps(shapes),text=True))
def number(n):
    v=f'{n:.6f}'.rstrip('0').rstrip('.')
    return v if '.' in v else v+'.0'
out=['//! Generated from pinned AOSP AnalogClock vector drawables, Apache-2.0.', 'use crate::beercss_shapes::ShapeData;', '#[rustfmt::skip]', 'pub static SHAPES: &[ShapeData] = &[']
for shape in shapes:
    points=shape['contours'][0]
    out.append('ShapeData { name:'+json.dumps(shape['name'])+', points:&['+','.join('['+number(x)+','+number(y)+']' for x,y in points)+'], contours:&[0,'+str(len(points))+'], triangles:&['+','.join(map(str,shape['triangles']))+'] },')
out.append('];')
(ROOT/'codes/libs/ui/libbhoo/src/shapes/android_clock_shapes.rs').write_text('\n'.join(out)+'\n')
print('Generated',len(shapes),'AOSP clock meshes')
