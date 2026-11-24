import {createTimePickerAccessibility} from './time-picker-accessibility.js';
import {createCarouselAccessibility} from './carousel-accessibility.js';
import {createDialogAccessibility} from './dialog-accessibility.js';
import {createLiveStatus} from './live-status.js';
import {paintPacket} from './canvas-painter.js';
import {names,mapping,light,night} from './showcase-data.js';
const inWorker=typeof WorkerGlobalScope!=='undefined' && self instanceof WorkerGlobalScope;
const syncLiveStatus=inWorker?()=>{}:createLiveStatus();
const canvas=document.querySelector('canvas'),ctx=canvas.getContext('2d',{alpha:false}),encoder=new TextEncoder();
let carouselKeyboardPending=false;
const syncCarouselAccessibility=inWorker?()=>{}:createCarouselAccessibility(canvas,{
 focus(id,context){dialogAction('focus',id,context);},
 activate(id,context){dialogAction('activate',id,context);},
 key(id,context,key){dialogAction('key',id,context,key);},
 tab(id,context,shift){carouselKeyboardPending=true;dialogAction('tab',id,context,shift);},
 shouldTransfer(){return carouselKeyboardPending;},
 transferred(){carouselKeyboardPending=false;}
});
const syncTimePickerAccessibility=inWorker?()=>{}:createTimePickerAccessibility(canvas,{
 allow(context){return items[Math.floor((context-1000)/2)]?.slug==='time-picker';},
 focus:(id,context)=>dialogAction('focus',id,context),
 key:(id,context,key)=>dialogAction('key',id,context,key),
 activate:(id,context)=>dialogAction('activate',id,context)
});
const syncDialogAccessibility=inWorker?()=>{}:createDialogAccessibility(canvas,{
 separate:true,
 focus(id,context){dialogAction('focus',id,context);},
 activate(id,context){dialogAction('activate',id,context);},
 escape(context){dialogAction('escape',null,context);}
});
let engine, catalog, items=[], slots=[], nodes=[], raf=0,timer=0, dark=false,reduced=matchMedia('(prefers-reduced-motion:reduce)').matches,expanded=true,paletteIndex=0;
let pointer=[NaN,NaN],owner=null,hover=null,focus=null,events=[],capture=0,dragged=false,code=null,editor=null,lastTouch=false,codeClosing=false,codePending=false,restoreFocus=null;
const fontMetrics=[];let fontBaselines=[0.3,0.3];
const tiles=new Map();let tileBytes=0,cacheEpoch=0,lastRailWidth=-1;const TILE_LIMIT=24*1024*1024;
function rasterSurface(w,h){if(typeof OffscreenCanvas!=='undefined')return new OffscreenCanvas(w,h);const c=document.createElement('canvas');c.width=w;c.height=h;return c}
function discardTile(id){const c=tiles.get(id);if(c){tileBytes-=c.bytes;tiles.delete(id);}}
function blit(tile,s){ctx.save();ctx.beginPath();ctx.rect(...s.clip);ctx.clip();ctx.drawImage(tile.surface,s.x,s.y,s.w,s.h);ctx.restore()}
const initialized=new Set(),seen=new Set(),errors=[];
const debug=window.bhooCanvas={ready:false,errors,seen,items:[],nodes:[],offset:0,total:0,contexts:initialized,goItem(i){engine.bhoo_page_item(i);request();},go(i){engine.bhoo_page_jump(i+1);request();}};
const keys={Tab:1,Enter:2,' ':3,ArrowLeft:4,ArrowRight:5,ArrowUp:6,ArrowDown:7,Home:8,End:9,Escape:10,ContextMenu:11,PageUp:12,PageDown:13};
function request(){if(engine&&!raf&&!document.hidden)raf=requestAnimationFrame(render)}
function select(id){engine.bhoo_context_select(id);}
function dialogAction(kind,id,context,key){
 if(kind==='tab'){const available=nodes.filter(n=>n.enabled&&!['group','status','dialog','dialog description','carousel'].includes(n.role)&&(!code||n.context===5));const i=available.findIndex(n=>n.context===context&&n.id===id);const next=available[(i+(key?-1:1)+available.length)%available.length];if(next){focus=next;select(next.context);engine.bhoo_adapter_focus64(BigInt(next.id),1);canvas.focus({preventScroll:true});queue({});}return;}
 if(kind==='escape'){focus={...focus,context};queue({key:10});return;}
 focus={id,context};select(context);engine.bhoo_adapter_focus64(BigInt(id),1);
 queue(kind==='activate'?{key:2}:kind==='key'?{key:keys[key]}:{});
}
if(inWorker)self.bhooDialogAction=dialogAction;
function config(title,description){for(const [kind,value]of [[0,title],[1,description]]){const bytes=encoder.encode(value||'').subarray(0,2048);new Uint8Array(engine.memory.buffer,engine.bhoo_edit_buffer(),bytes.length).set(bytes);engine.bhoo_showcase_text(kind,bytes.length)}}
function info(id){if(id>=1000){const item=items[Math.floor((id-1000)/2)];return {item,title:item.name,description:item.summary}}if(id>=10&&id<52)return {title:names[id-10],description:`${items.filter(c=>c.group===id-10).length} interactive examples`};return {}}
function palette(){const colors=(dark?night:light).map(c=>[...c]);if(paletteIndex){colors[5]=paletteIndex===1?(dark?[160,210,179]:[43,106,77]):(dark?[229,202,110]:[112,93,0]);colors[6]=dark?[18,34,20]:[255,255,255];}colors.forEach((c,i)=>engine.bhoo_palette_set(i,(c[0]|c[1]<<8|c[2]<<16|255<<24)>>>0))}
function closeCode(){if(!code)return;select(5);engine.bhoo_showcase_dialog(0);codeClosing=true;request()}

function intersects(s,x,y){return x>=s.x&&y>=s.y&&x<s.x+s.w&&y<s.y+s.h&&x>=s.clip[0]&&y>=s.clip[1]&&x<s.clip[0]+s.clip[2]&&y<s.clip[1]+s.clip[3]}
function hit(x,y){return [...slots].reverse().find(s=>intersects(s,x,y))}
function localSurface(s){if(!s)return false;if(s.id===1||s.id===5)return true;const slug=info(s.id).item?.slug;if(['scroll-area-viewport','scrollbar','virtualized-list','virtualized-grid','top-app-bar','bottom-app-bar','toolbar','carousel'].includes(slug))return true;select(s.id);return engine.bhoo_overlay_open()!==0;}
function nodeAt(s,x,y){return nodes.find(n=>!['group','status','dialog','dialog description','carousel'].includes(n.role)&&n.context===s?.id&&x>=n.bounds[0]&&y>=n.bounds[1]&&x<n.bounds[0]+n.bounds[2]&&y<n.bounds[1]+n.bounds[3])}
function queue(e){if((e.flags&127)||e.key)cacheEpoch++;const next={...e,x:pointer[0],y:pointer[1]},last=events.at(-1);const motion=v=>!(v.flags&127)&&!v.key&&!v.pageKey;
 if(last&&motion(last)&&motion(next)&&!!last.pageScroll===!!next.pageScroll&&!!last.scroll===!!next.scroll){next.pageScroll=(last.pageScroll||0)+(next.pageScroll||0);next.scroll=(last.scroll||0)+(next.scroll||0);events[events.length-1]=next;}else events.push(next);debug.maxQueue=Math.max(debug.maxQueue||0,events.length);request()}
function activate(action,s){if(!action)return;if(action===1001){dark=!dark;palette()}else if(action===1004)reduced=!reduced;else if(action===1011){paletteIndex=(paletteIndex+1)%3;palette()}else if(action===1006){restoreFocus=focus;codeClosing=false;codePending=true;code=info(s.id).item||{name:info(s.id).title,code:'// Bhoo Rust canvas showcase\n// codes/apps/bhoo-web/src/onepage.rs\n// codes/apps/bhoo-web/src/showcase.rs'};editor?.blur()}else if(action===1007){closeCode()}else if(action===1010){engine.bhoo_page_action(1)}else engine.bhoo_page_action(action);request()}
function render(time){const frameStart=performance.now();raf=0;clearTimeout(timer);try{
 const w=innerWidth,h=innerHeight,dpr=Math.min(devicePixelRatio||1,2);if(canvas.width!==Math.round(w*dpr)||canvas.height!==Math.round(h*dpr)){canvas.width=Math.round(w*dpr);canvas.height=Math.round(h*dpr)}
 const event=events.shift()||{x:pointer[0],y:pointer[1],flags:0,key:0,scroll:0};
 engine.bhoo_page_reduced(Number(reduced));engine.bhoo_palette_tick(time/1000,Number(reduced));
 const result=engine.bhoo_page_update(w,h,time/1000,event.x,event.y,(event.flags||0)|(event.pageKey?16:0),event.pageKey||0,event.pageScroll||0,capture||Number(!!code));
 if(result&1)dragged=true;
 const raw=new Float32Array(engine.memory.buffer,engine.bhoo_page_slots(),engine.bhoo_page_slots_len()).slice();slots=[];
 for(let i=0;i<raw.length;i+=10)slots.push({id:raw[i],scene:raw[i+1],x:raw[i+2],y:raw[i+3],w:raw[i+4],h:raw[i+5],clip:Array.from(raw.slice(i+6,i+10))});
 if(code){slots.push({id:5,scene:10004,x:0,y:0,w,h,clip:[0,0,w,h]})}
 ctx.setTransform(dpr,0,0,dpr,0,0);const bg=engine.bhoo_palette_background();ctx.fillStyle=`rgb(${bg&255},${(bg>>>8)&255},${(bg>>>16)&255})`;ctx.fillRect(0,0,w,h);
 if(dragged&&owner&&!slots.some(s=>s.id===owner.id)){select(owner.id);engine.bhoo_frame(owner.scene,owner.w,owner.h,time/1000,NaN,NaN,4,0,0,16|32|(1<<8)|(dark?4:0)|(reduced?2:0));}
 const railWidth=slots.find(s=>s.id===1)?.w||0,stableLayout=railWidth===lastRailWidth;lastRailWidth=railWidth;
 const visibleIds=new Set(slots.map(s=>s.id));for(const id of tiles.keys())if(!visibleIds.has(id))discardTile(id);
 const target=owner||hit(event.x,event.y);let active=!!(result&2)||!!engine.bhoo_palette_moving(),wake=Infinity;const nextNodes=[];
 for(const s of slots){
   if(s.id===1&&engine.bhoo_page_drawer_progress()>0){ctx.fillStyle=`rgba(0,0,0,${0.4*engine.bhoo_page_drawer_progress()})`;ctx.fillRect(0,0,w,h)}
   select(s.id);const meta=info(s.id);
   if(!initialized.has(s.id)){engine.bhoo_reset(s.scene);for(const [code,n,m]of fontMetrics)engine.bhoo_font_advance(code,n,m);engine.bhoo_font_baseline(...fontBaselines);initialized.add(s.id);if(s.scene>=10000){config(s.id===5?code.name:meta.title,s.id===5?code.code:meta.description)}}
   if(s.id===5){config(code.name,code.code);if(codePending){engine.bhoo_showcase_dialog(1);engine.bhoo_adapter_focus(50,1);focus={context:5,id:'50'};codePending=false;}}
   if(s.scene>=10000)engine.bhoo_showcase_config(engine.bhoo_page_selected(),engine.bhoo_page_expanded());
   const targeted=target?.id===s.id&&(!code||s.id===5)&&!codeClosing;
   const keyTarget=focus?.context===s.id;
   let flags=targeted?(event.flags||0):0;
   if(dragged&&targeted){flags=(flags&~3)|4;}
   const key=keyTarget?(event.key||0):0;if(key)flags|=16|(event.shift?64:0);
   const px=targeted&&!dragged?event.x-s.x:NaN,py=targeted&&!dragged?event.y-s.y:NaN;
   const localDelta=engine.bhoo_local_scroll(targeted?(event.scroll||0):0,time/1000,Number(reduced),Number(targeted&&!dragged));
   const localMoving=!!engine.bhoo_local_scroll_moving();active||=localMoving;
   const canCache=!localMoving&&!debug.disableCache&&stableLayout&&s.id!==5&&(!targeted||dragged)&&(!keyTarget||dragged)&&!engine.bhoo_palette_moving();
   const signature=`${s.w}/${s.h}/${dpr}/${dark}/${reduced}/${paletteIndex}/${cacheEpoch}/${s.id===1?engine.bhoo_page_selected():0}/${s.id===1?engine.bhoo_page_expanded():0}`;
   let tile=tiles.get(s.id),frame;
   if(canCache&&tile?.signature===signature&&(tile.frame.wake<0||tile.frame.wake*1000>time)) {frame=tile.frame;blit(tile,s);debug.cacheHits=(debug.cacheHits||0)+1;}
   else {
     discardTile(s.id);
     const ptr=engine.bhoo_frame(s.scene,s.w,s.h,time/1000,px,py,flags,key,localDelta,16|32|(1<<8)|(dark?4:0)|(reduced?2:0));
     const packet=new Uint8Array(engine.memory.buffer,ptr,engine.bhoo_packet_len());
     const animating=new DataView(packet.buffer,packet.byteOffset,packet.byteLength).getUint32(12,true)!==0;
     const pw=Math.max(1,Math.ceil(s.w*dpr)),ph=Math.max(1,Math.ceil(s.h*dpr)),bytes=pw*ph*4;
     if(canCache&&!animating&&bytes<=TILE_LIMIT){
       while(tileBytes+bytes>TILE_LIMIT&&tiles.size)discardTile(tiles.keys().next().value);
       const surface=rasterSurface(pw,ph),tileCtx=surface.getContext('2d');tileCtx.setTransform(dpr,0,0,dpr,0,0);
       frame=paintPacket(tileCtx,engine,packet,[0,0,s.w,s.h],[0,0,s.w,s.h]);
       tile={surface,frame,signature,bytes};tiles.set(s.id,tile);tileBytes+=bytes;blit(tile,s);
     }else frame=paintPacket(ctx,engine,packet,[s.x,s.y,s.w,s.h],s.clip);
   }
   active||=frame.active;if(frame.wake>=0)wake=Math.min(wake,frame.wake*1000-time);
   for(const n of frame.semantics){const bounds=[n.bounds[0]+s.x,n.bounds[1]+s.y,...n.bounds.slice(2)];if(bounds[1]+bounds[3]>Math.max(s.y,s.clip[1])&&bounds[1]<Math.min(s.y+s.h,s.clip[1]+s.clip[3]))nextNodes.push({...n,context:s.id,bounds,item:meta.item})}
   if(s.id>=1001&&s.id%2===1)seen.add(meta.item.slug);
   if(s.scene>=10000)activate(engine.bhoo_showcase_action(),s);
   if(s.id===5&&codeClosing&&!frame.active){code=null;codeClosing=false;focus=restoreFocus;if(focus){select(focus.context);engine.bhoo_adapter_focus(Number(focus.id),1);}request();}
 }
 nodes=nextNodes;syncLiveStatus(nodes);syncDialogAccessibility(nodes);syncCarouselAccessibility(nodes);syncTimePickerAccessibility(nodes);debug.nodes=nodes;debug.slots=slots;debug.tileBytes=tileBytes;debug.background=bg;debug.paletteMoving=!!engine.bhoo_palette_moving();debug.offset=engine.bhoo_page_offset();debug.total=engine.bhoo_page_total();
 // Scrollbar is a canvas primitive; its offset and drag behavior are Rust-owned.
 const trackH=h-(w<600?80:0),thumbH=Math.max(32,trackH*trackH/debug.total),thumbY=debug.offset/Math.max(1,debug.total-h)*(trackH-thumbH);
 ctx.fillStyle=dark?'#ffffff55':'#00000055';ctx.beginPath();ctx.roundRect(w-8,thumbY,6,thumbH,3);ctx.fill();
 if(focus&&editor)placeEditor();
 if(event.flags&2){owner=null;dragged=false;capture=0;}
 document.querySelector('#status').hidden=true;debug.ready=true;debug.frameMs=performance.now()-frameStart;debug.maxQueue=Math.max(debug.maxQueue||0,events.length);
 globalThis.bhooPublish?.(debug);
 if(active||events.length)request();else if(Number.isFinite(wake))timer=setTimeout(request,Math.max(1,wake));
 }catch(e){errors.push(String(e));console.error(e);document.querySelector('#status').hidden=false;document.querySelector('#status').textContent=String(e)}}
function placeEditor(){if(!editor||!focus)return;const n=nodes.find(n=>n.context===focus.context&&n.id===focus.id);if(!n){editor.remove();editor=null;return}const [x,y,w,h]=n.bounds;Object.assign(editor.style,{left:x+'px',top:y+'px',width:w+'px',height:h+'px'});}
function edit(n){editor?.remove();focus=n;select(n.context);engine.bhoo_adapter_focus(Number(n.id),1);const multiline=n.item?.slug==='multiline-text-field';editor=document.createElement(multiline?'textarea':'input');editor.id='editor';editor.setAttribute('aria-label',n.label);editor.autocomplete='off';editor.spellcheck=false;
 const len=engine.bhoo_text_len();if(len)editor.value=new TextDecoder().decode(new Uint8Array(engine.memory.buffer,engine.bhoo_text_pointer(),len));
 editor.addEventListener('input',()=>{cacheEpoch++;select(n.context);const bytes=encoder.encode(editor.value).subarray(0,2048);new Uint8Array(engine.memory.buffer,engine.bhoo_edit_buffer(),bytes.length).set(bytes);engine.bhoo_edit_commit(bytes.length);request()});editor.addEventListener('blur',()=>{select(n.context);engine.bhoo_adapter_focus(Number(n.id),0);request()});editor.addEventListener('wheel',wheel,{passive:false});document.body.append(editor);placeEditor();editor.focus();request()}
canvas.addEventListener('pointerdown',e=>{carouselKeyboardPending=false;if(e.button!==0||!e.isPrimary)return;if(engine.bhoo_page_menu()&&e.clientX>280){engine.bhoo_page_action(1003);request();return}canvas.focus({preventScroll:true});pointer=[e.clientX,e.clientY];lastTouch=e.pointerType==='touch';owner=hit(...pointer);const n=nodeAt(owner,...pointer);focus=n||focus;capture=Number(['slider','text field'].includes(n?.role)||localSurface(owner)||!!code);dragged=false;canvas.setPointerCapture(e.pointerId);queue({flags:1|(lastTouch?128:0)});});
canvas.addEventListener('pointermove',e=>{pointer=[e.clientX,e.clientY];hover=hit(...pointer);queue({flags:e.pointerType==='touch'?128:0})});
canvas.addEventListener('pointerup',e=>{pointer=[e.clientX,e.clientY];const n=nodeAt(owner,...pointer);queue({flags:2|(e.pointerType==='touch'?128:0)});if(n?.role==='text field'&&!dragged)edit(n);if(e.pointerType==='touch')pointer=[NaN,NaN]});
canvas.addEventListener('pointercancel',()=>{queue({flags:4});owner=null;capture=0;dragged=false});
canvas.addEventListener('pointerleave',()=>{if(!owner){pointer=[NaN,NaN];request()}});
function wheel(e){e.preventDefault();pointer=[e.clientX,e.clientY];const s=hit(...pointer),local=localSurface(s);const delta=e.deltaY*(e.deltaMode===1?40:e.deltaMode===2?innerHeight:1);queue(local?{scroll:delta}:{pageScroll:delta})}
canvas.addEventListener('wheel',wheel,{passive:false});
window.addEventListener('keydown',e=>{if(e.target===canvas)carouselKeyboardPending=e.key==='Tab';if(e.target===editor&&e.key!=='Escape'&&e.key!=='Tab')return;if(e.key==='Tab'){e.preventDefault();const available=nodes.filter(n=>n.enabled&&!['group','status','dialog','dialog description','carousel'].includes(n.role)&&(!code||n.context===5));let i=available.findIndex(n=>n.context===focus?.context&&n.id===focus?.id);i=(i+(e.shiftKey?-1:1)+available.length)%available.length;const n=available[i];if(n){editor?.blur();editor?.remove();editor=null;focus=n;select(n.context);engine.bhoo_adapter_focus(Number(n.id),1);if(n.role==='text field')edit(n);request()}return}if(e.key==='Escape'){editor?.blur();editor?.remove();editor=null;if(code)closeCode();else if(engine.bhoo_page_menu())engine.bhoo_page_action(1003);queue({key:10});return}const key=keys[e.key];if(!key)return;e.preventDefault();const carouselFocus=focus?.item?.slug==='carousel'&&nodes.some(n=>n.context===focus.context&&n.id==='130'&&n.role==='carousel'&&focus.bounds[0]<n.bounds[0]+n.bounds[2]&&focus.bounds[0]+focus.bounds[2]>n.bounds[0]&&focus.bounds[1]<n.bounds[1]+n.bounds[3]&&focus.bounds[1]+focus.bounds[3]>n.bounds[1]);const pageKey=([12,13].includes(key)||([8,9].includes(key)&&!carouselFocus)||(!focus&&[6,7].includes(key)))?key:0;queue({key:pageKey?0:key,pageKey,shift:e.shiftKey})});
window.addEventListener('resize',()=>{queue({flags:4});});window.addEventListener('blur',()=>{queue({flags:8});owner=null;capture=0;});document.addEventListener('visibilitychange',()=>{if(document.hidden){cancelAnimationFrame(raf);raf=0;clearTimeout(timer)}else request()});
try{const [bytes,data]=await Promise.all([fetch('bhoo.wasm').then(r=>r.arrayBuffer()),fetch('catalog.json').then(r=>r.json())]);engine=(await WebAssembly.instantiate(bytes,{})).instance.exports;catalog=data;
 ctx.save();for(let code=32;code<127;code++){const ch=String.fromCharCode(code);ctx.font='400 100px Arial, sans-serif';const n=ctx.measureText(ch).width/100;ctx.font='500 100px Arial, sans-serif';fontMetrics.push([code,n,ctx.measureText(ch).width/100]);}fontBaselines=[400,700].map(weight=>{ctx.font=`${weight} 100px Arial, sans-serif`;const m=ctx.measureText('Hg');return ((m.fontBoundingBoxAscent??80)-(m.fontBoundingBoxDescent??20))/200;});ctx.restore();
 const used=new Set();names.forEach((name,group)=>{for(const slug of mapping[name]||[]){const item=catalog.components.find(c=>c.slug===slug);if(item){items.push({...item,group});used.add(slug)}}});for(const item of catalog.components)if(!used.has(item.slug))items.push({...item,group:41});
 for(const item of items){const height=['date-picker','date-range-picker','expressive-shapes','button-group','toolbar','navigation-rail'].includes(item.slug)?640:['dialog','bottom-sheet','floating-toolbar','tree','stepper','data-table'].includes(item.slug)?520:384;engine.bhoo_page_add(item.scene,item.group,height)}debug.items=items;palette();request();
}catch(e){errors.push(String(e));document.querySelector('#status').textContent=String(e);console.error(e)}
