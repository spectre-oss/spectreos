#!/usr/bin/env python3
"""Strip the 1px nine-patch metadata border into RGBA image assets."""
from pathlib import Path
from android_png import decode_png
root=Path(__file__).resolve().parents[1]/'codes/libs/ui/libbhoo/assets/android-zoomcontrols'
for direction in ('down','up'):
    for state in ('normal','pressed','disabled','selected','disabled_focused'):
        source=root/f'btn_zoom_{direction}_{state}.9.png'
        width,height,data=decode_png(source)
        if (width,height)!=(81,50):raise ValueError(f'Unexpected dimensions: {source}')
        result=b''.join(data[(y*width+1)*4:(y*width+80)*4] for y in range(1,height-1))
        (root/f'{direction}_{state}.rgba').write_bytes(result)
