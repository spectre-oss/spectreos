// Generate the original, synthetic image used by the Beer CSS comparison fixture.
import fs from 'node:fs';
import {encodePng} from './bhoo-png-diff.mjs';
const width=64,height=40,pixels=new Uint8Array(width*height*4);
for(let y=0;y<height;y++)for(let x=0;x<width;x++){const i=(y*width+x)*4;pixels.set([30+x*3,80+y*3,160,255],i);}
fs.writeFileSync('codes/apps/bhoo-web/site/beercss/demo-media.png',encodePng(width,height,pixels));
