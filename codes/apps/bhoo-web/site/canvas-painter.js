// Shared Bhoo packet protocol, drawn into a caller-provided canvas region.
const decoder=new TextDecoder();
const roleNames=['button','checkbox','radio group','radio','switch','slider','tab list','tab','list','list item','progress','tooltip','group','text field','menu','menu item','link','toggle button','status','dialog','dialog description','carousel','carousel item'];
const imageCache=new Map();let imageCacheBytes=0;
const PERSIAN_RE=/[\u0600-\u06FF\u0750-\u077F\u08A0-\u08FF\uFB50-\uFDFF\uFE70-\uFEFF]/;
function takeColor(view,at){return `rgba(${view.getUint8(at)},${view.getUint8(at+1)},${view.getUint8(at+2)},${view.getUint8(at+3)/255})`;}
export function paintPacket(ctx, engine, packet, box, clip){
  const [ox,oy,w,h]=box; const embedded=true;
  const rootCtx=ctx;
  const defaultFont=embedded?'18px Arial, "Shabnam", sans-serif':'18px -apple-system, BlinkMacSystemFont, "Segoe UI", "Shabnam", sans-serif';
  ctx.save();ctx.beginPath();ctx.rect(...clip);ctx.clip();ctx.translate(ox,oy);ctx.beginPath();ctx.rect(0,0,w,h);ctx.clip();
  const v=new DataView(packet.buffer,packet.byteOffset,packet.byteLength);
  if(v.getUint32(0,true)!==0x42484f4f||v.getUint32(4,true)!==1)throw Error('Unsupported Bhoo render packet');
  const count=v.getUint32(8,true),active=Boolean(v.getUint32(12,true)),wake=v.getFloat64(16,true),semanticCount=v.getUint32(24,true);
  let at=32;const renderedText=[],ripples=[],paths=[],layers=[];
  const rect=()=>{const b=[0,4,8,12].map(offset=>v.getFloat32(at+offset,true));at+=16;return b;};
  const text=()=>{const n=v.getUint32(at,true);at+=4;const value=decoder.decode(packet.subarray(at,at+n));at=(at+n+3)&~3;return value;};
  ctx.font=defaultFont;ctx.textBaseline='alphabetic';
  for(let i=0;i<count;i++){
    const tag=v.getUint32(at,true);at+=4;
    if(tag===0){ctx.fillStyle=takeColor(v,at);at+=4;ctx.fillRect(0,0,w,h);}
    else if(tag===1){const [x,y,bw,bh]=rect(),radius=v.getFloat32(at,true);at+=4;ctx.fillStyle=takeColor(v,at);at+=4;if(bw<=0||bh<=0)continue;ctx.beginPath();ctx.roundRect(x,y,bw,bh,Math.max(0,Math.min(radius,bw/2,bh/2)));ctx.fill();}
    else if(tag===2){const [x,y,bw,bh]=rect();ctx.fillStyle=takeColor(v,at);at+=4;const label=text();renderedText.push(label);if(bw<=0||bh<=0)continue;ctx.save();ctx.beginPath();ctx.rect(x,y,bw,bh);ctx.clip();ctx.fillText(label,x+12,y+(bh-18)*.5+17);ctx.restore();}
    else if(tag===9){
      const [x,y,bw,bh]=rect(),color=takeColor(v,at);at+=4;
      const size=v.getFloat32(at,true),bits=v.getUint32(at+4,true),padding=v.getFloat32(at+8,true),spacing=v.getFloat32(at+12,true);at+=16;
      const label=text();renderedText.push(label);if(bw<=0||bh<=0)continue;
      const weight=bits&65535,align=(bits>>>16)&255,italic=Boolean((bits>>>24)&1),familyId=(bits>>>25)&3;
      const hasPersian=PERSIAN_RE.test(label);
      const familyCss=familyId===1?(hasPersian?'"Shabnam", Georgia, serif':'Georgia, "Shabnam", serif'):familyId===2?(hasPersian?'"Shabnam", monospace':'monospace, "Shabnam"'):(hasPersian?'"Shabnam", Arial, sans-serif':'Arial, "Shabnam", sans-serif');
      ctx.save();ctx.beginPath();ctx.rect(x,y,bw,bh);ctx.clip();ctx.fillStyle=color;ctx.font=`${italic?'italic ':''}${weight} ${size}px ${familyCss}`;ctx.letterSpacing=spacing+'px';
      const metrics=ctx.measureText(label),ascent=metrics.fontBoundingBoxAscent??size*.8,descent=metrics.fontBoundingBoxDescent??size*.2;
      ctx.textAlign=align===1?'center':align===2?'right':'left';ctx.textBaseline='alphabetic';
      ctx.fillText(label,align===1?x+bw/2:align===2?x+bw-padding:x+padding,y+(bh+ascent-descent)/2);ctx.restore();
    }
    else if(tag===10){
      const [x,y,bw,bh]=rect(),radius=v.getFloat32(at,true),dx=v.getFloat32(at+4,true),dy=v.getFloat32(at+8,true),blur=v.getFloat32(at+12,true),spread=v.getFloat32(at+16,true);at+=20;const color=takeColor(v,at);at+=4;
      ctx.save();ctx.shadowOffsetX=dx+10000;ctx.shadowOffsetY=dy;ctx.shadowBlur=blur;ctx.shadowColor=color;ctx.fillStyle="#000";
      ctx.beginPath();ctx.roundRect(x-10000-spread,y-spread,Math.max(0,bw+spread*2),Math.max(0,bh+spread*2),Math.max(0,radius+spread));ctx.fill();ctx.restore();
    }
    else if(tag===13){const origin=v.getFloat32(at,true),scale=v.getFloat32(at+4,true);at+=8;ctx.save();ctx.translate(0,origin);ctx.scale(1,scale);ctx.translate(0,-origin);}
    else if(tag===14){ctx.restore();}
    else if(tag===20){const tx_ox=v.getFloat32(at,true),tx_oy=v.getFloat32(at+4,true),tx=v.getFloat32(at+8,true),ty=v.getFloat32(at+12,true),sx=v.getFloat32(at+16,true),sy=v.getFloat32(at+20,true),rot=v.getFloat32(at+24,true);at+=28;ctx.save();ctx.translate(tx_ox+tx,tx_oy+ty);ctx.rotate(rot);ctx.scale(sx,sy);ctx.translate(-tx_ox,-tx_oy);}
    else if(tag===21){ctx.restore();}
    else if(tag===22){
      const [lx,ly,lw,lh]=rect(),alpha=Math.max(0,Math.min(1,v.getFloat32(at,true)));at+=4;
      const cw=ctx.canvas.width,ch=ctx.canvas.height;
      let skipped=alpha<=0||lw<=0||lh<=0||cw<=0||ch<=0,x0=0,y0=0,pw=0,ph=0,m=null;
      if(!skipped){
        m=ctx.getTransform();
        const xs=[],ys=[];
        for(const [px,py] of [[lx,ly],[lx+lw,ly],[lx+lw,ly+lh],[lx,ly+lh]]){xs.push(m.a*px+m.c*py+m.e);ys.push(m.b*px+m.d*py+m.f);}
        x0=Math.max(0,Math.floor(Math.min(...xs)));y0=Math.max(0,Math.floor(Math.min(...ys)));
        const x1=Math.min(cw,Math.ceil(Math.max(...xs))),y1=Math.min(ch,Math.ceil(Math.max(...ys)));
        pw=x1-x0;ph=y1-y0;if(pw<=0||ph<=0)skipped=true;
      }
      if(skipped){ctx.save();ctx.beginPath();ctx.rect(0,0,0,0);ctx.clip();layers.push({parent:ctx,skipped:true});}
      else{
        const surface=typeof OffscreenCanvas!=='undefined'?new OffscreenCanvas(pw,ph):document.createElement('canvas');
        surface.width=pw;surface.height=ph;
        const lctx=surface.getContext('2d');
        lctx.setTransform(m.a,m.b,m.c,m.d,m.e-x0,m.f-y0);
        lctx.beginPath();lctx.rect(lx,ly,lw,lh);lctx.clip();
        lctx.font=defaultFont;lctx.textBaseline='alphabetic';
        layers.push({parent:ctx,surface,x0,y0,pw,ph,alpha,skipped:false});
        ctx=lctx;
      }
    }
    else if(tag===23){
      const layer=layers.pop();if(!layer)throw Error('Unbalanced Bhoo layer stack');
      ctx=layer.parent;
      if(layer.skipped){ctx.restore();}
      else{ctx.save();ctx.setTransform(1,0,0,1,0,0);ctx.globalAlpha*=layer.alpha;ctx.drawImage(layer.surface,layer.x0,layer.y0,layer.pw,layer.ph);ctx.restore();}
    }
    else if(tag===3){const b=rect();ctx.save();ctx.beginPath();ctx.rect(...b);ctx.clip();}
    else if(tag===4)ctx.restore();
    else if(tag===5||tag===11){
      const [x,y,bw,bh]=rect(),corner=v.getFloat32(at,true),cx=v.getFloat32(at+4,true),cy=v.getFloat32(at+8,true),radius=v.getFloat32(at+12,true);at+=16;
      const r=v.getUint8(at),g=v.getUint8(at+1),b=v.getUint8(at+2),a=v.getUint8(at+3)/255;at+=4;
      ripples.push({soft:tag===5,bounds:[x,y,bw,bh],corner,center:[cx,cy],radius,alpha:a});
      if(bw<=0||bh<=0||radius<=0)continue;
      ctx.save();ctx.beginPath();ctx.roundRect(x,y,bw,bh,Math.max(0,Math.min(corner,bw/2,bh/2)));ctx.clip();
      if(tag===11){ctx.fillStyle=`rgba(${r},${g},${b},${a})`;ctx.beginPath();ctx.arc(cx,cy,radius,0,Math.PI*2);ctx.fill();ctx.restore();continue;}
      const initialRadius=Math.max(Math.floor(Math.max(bw,bh)*.2)*.5,.5),solid=Math.max(1-70/initialRadius,.65);
      const gradient=ctx.createRadialGradient(cx,cy,0,cx,cy,radius);gradient.addColorStop(0,`rgba(${r},${g},${b},${a})`);gradient.addColorStop(solid,`rgba(${r},${g},${b},${a})`);gradient.addColorStop(1,`rgba(${r},${g},${b},0)`);ctx.fillStyle=gradient;ctx.fillRect(x,y,bw,bh);ctx.restore();
    }
    else if(tag===6){
      ctx.fillStyle=takeColor(v,at);at+=4;const count=v.getUint32(at,true);at+=4;
      paths.push({curves:count,firstCurve:Array.from({length:8},(_,k)=>v.getFloat32(at+k*4,true))});
      ctx.beginPath();for(let j=0;j<count;j++){const c=Array.from({length:8},(_,k)=>v.getFloat32(at+k*4,true));at+=32;if(j===0)ctx.moveTo(c[0],c[1]);ctx.bezierCurveTo(...c.slice(2));}ctx.closePath();ctx.fill();
    }
    else if(tag===7||tag===12){const [x,y,w,h]=rect(),start=v.getFloat32(at,true),sweep=v.getFloat32(at+4,true),stroke=v.getFloat32(at+8,true);at+=12;const color=takeColor(v,at);at+=4;if(sweep<=0.0001||w<=0||h<=0)continue;ctx.save();ctx.beginPath();ctx.arc(x+w/2,y+h/2,Math.max(0,(Math.min(w,h)-stroke)/2),start,start+Math.min(sweep,Math.PI*2));ctx.strokeStyle=color;ctx.lineWidth=stroke;ctx.lineCap=tag===12?'butt':'round';ctx.stroke();ctx.restore();}
    else if(tag===8){const x1=v.getFloat32(at,true),y1=v.getFloat32(at+4,true),x2=v.getFloat32(at+8,true),y2=v.getFloat32(at+12,true),width=v.getFloat32(at+16,true);at+=20;const color=takeColor(v,at);at+=4;ctx.save();ctx.beginPath();ctx.moveTo(x1,y1);ctx.lineTo(x2,y2);ctx.lineWidth=width;ctx.strokeStyle=color;ctx.lineCap='round';ctx.stroke();ctx.restore();}
    else if(tag===19){const [x,y,bw,bh]=rect(),radius=v.getFloat32(at,true),width=v.getFloat32(at+4,true);at+=8;ctx.strokeStyle=takeColor(v,at);at+=4;const d=Math.min(Math.max(width,0),bw/2,bh/2);if(d<=0)continue;ctx.beginPath();if(radius<0){ctx.ellipse(x+bw/2,y+bh/2,(bw-d)/2,(bh-d)/2,0,0,Math.PI*2);}else{ctx.roundRect(x+d/2,y+d/2,bw-d,bh-d,Math.max(0,Math.min(radius-d/2,(bw-d)/2,(bh-d)/2)));}ctx.lineWidth=d;ctx.stroke();}
    else if(tag===18){const [x,y,bw,bh]=rect();ctx.fillStyle=takeColor(v,at);at+=4;if(bw<=0||bh<=0)continue;ctx.beginPath();ctx.ellipse(x+bw/2,y+bh/2,bw/2,bh/2,0,0,Math.PI*2);ctx.fill();}
    else if(tag===17){const p=Array.from({length:3},(_,i)=>[v.getFloat32(at+i*8,true),v.getFloat32(at+i*8+4,true)]);at+=24;const count=v.getUint32(at,true),width=v.getFloat32(at+4,true);at+=8;const color=takeColor(v,at);at+=4;ctx.save();ctx.beginPath();ctx.moveTo(...p[0]);for(let i=1;i<Math.min(count,3);i++)ctx.lineTo(...p[i]);ctx.lineWidth=width;ctx.strokeStyle=color;ctx.lineCap='butt';ctx.lineJoin='miter';ctx.stroke();ctx.restore();}
    else if(tag===15){const color=takeColor(v,at);at+=4;const bounds=rect(),contours=v.getUint32(at,true);at+=4;ctx.save();ctx.beginPath();ctx.rect(...bounds);ctx.clip();ctx.fillStyle=color;ctx.beginPath();for(let c=0;c<contours;c++){const n=v.getUint32(at,true);at+=4;for(let i=0;i<n;i++){const x=v.getFloat32(at,true),y=v.getFloat32(at+4,true);at+=8;if(i===0)ctx.moveTo(x,y);else ctx.lineTo(x,y);}ctx.closePath();}ctx.fill('evenodd');ctx.restore();}
    else if(tag===16){const [x,y,bw,bh]=rect(),uv=[0,4,8,12].map(o=>v.getFloat32(at+o,true));at+=16;const radius=v.getFloat32(at,true);at+=4;const alpha=v.getUint8(at+3)/255;at+=4;const id=v.getUint32(at,true),iw=v.getUint32(at+4,true),ih=v.getUint32(at+8,true),ptr=v.getUint32(at+12,true);at+=16;let entry=imageCache.get(id);if(!entry){const bytes=iw*ih*4;if(bytes>16*1024*1024||ptr+bytes>engine.memory.buffer.byteLength)throw Error('Invalid image buffer');while(imageCacheBytes+bytes>16*1024*1024&&imageCache.size){const [key,old]=imageCache.entries().next().value;imageCache.delete(key);imageCacheBytes-=old.bytes;}const surface=typeof OffscreenCanvas!=='undefined'?new OffscreenCanvas(iw,ih):document.createElement('canvas');surface.width=iw;surface.height=ih;surface.getContext('2d').putImageData(new ImageData(new Uint8ClampedArray(engine.memory.buffer.slice(ptr,ptr+bytes)),iw,ih),0,0);entry={surface,bytes};imageCache.set(id,entry);imageCacheBytes+=bytes;}else{imageCache.delete(id);imageCache.set(id,entry);}ctx.save();ctx.globalAlpha*=alpha;ctx.beginPath();ctx.roundRect(x,y,bw,bh,Math.max(0,Math.min(radius,bw/2,bh/2)));ctx.clip();const flipX=uv[2]<uv[0],flipY=uv[3]<uv[1];ctx.translate(x+(flipX?bw:0),y+(flipY?bh:0));ctx.scale(flipX?-1:1,flipY?-1:1);ctx.drawImage(entry.surface,Math.min(uv[0],uv[2])*iw,Math.min(uv[1],uv[3])*ih,Math.abs(uv[2]-uv[0])*iw,Math.abs(uv[3]-uv[1])*ih,0,0,bw,bh);ctx.restore();}
    else throw Error(`Unknown drawing command ${tag}`);
  }
  if(layers.length){rootCtx.restore();throw Error('Unbalanced Bhoo layer stack');}
  const semantics=[];
  for(let i=0;i<semanticCount;i++){
    const id=v.getBigUint64(at,true).toString();at+=8;
    const part=v.getUint32(at,true),role=v.getUint32(at+4,true);at+=8;
    const bounds=rect(),flags=v.getUint32(at,true),checked=v.getUint32(at+4,true),value=v.getFloat32(at+8,true),min=v.getFloat32(at+12,true),max=v.getFloat32(at+16,true);at+=20;
    const label=text();semantics.push({id,part,role:roleNames[role]||'unknown',label,bounds,enabled:Boolean(flags&1),focused:Boolean(flags&2),selected:Boolean(flags&4),invalid:Boolean(flags&8),filled:Boolean(flags&16),required:Boolean(flags&32),checked:checked?['','unchecked','checked','mixed'][checked]:null,value:Number.isFinite(value)?value:null,min,max});
  }
  rootCtx.restore();
  return {active,wake,semantics,text:renderedText,commands:count};
}
