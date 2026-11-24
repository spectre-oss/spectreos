// Platform adapter only. Layout, input, motion and painting run in this worker.
const listeners=new Map();let editor=null,offscreen,status={};
const listen=(target,type,fn)=>{const key=target+':'+type;if(!listeners.has(key))listeners.set(key,[]);listeners.get(key).push(fn)};
const dispatch=(target,type,e)=>{for(const fn of listeners.get(target+':'+type)||[])fn(e)};
const send=(type,data={})=>postMessage({type,...data});
self.onmessage=async ({data:m})=>{
 if(m.type==='init'){
  offscreen=m.canvas;self.innerWidth=m.width;self.innerHeight=m.height;self.devicePixelRatio=m.dpr;
  self.window=self;self.matchMedia=()=>({matches:m.reduced});
  self.addEventListener=(type,fn)=>listen('window',type,fn);
  offscreen.addEventListener=(type,fn)=>listen('canvas',type,fn);
  offscreen.focus=()=>send('focus');offscreen.setPointerCapture=()=>{};
  const statusProxy=new Proxy({}, {set(o,k,v){if(o[k]!==v){o[k]=v;send('status',{key:k,value:v})}return true}});
  self.document={hidden:false,querySelector:s=>s==='canvas'?offscreen:s==='#status'?statusProxy:s==='#editor'?editor:null,
   addEventListener:(type,fn)=>listen('document',type,fn),
   body:{append(e){send('editor',{kind:e.kind,value:e.value,style:{...e.style},label:e.label})}},
   createElement(kind){if(kind==='canvas')return new OffscreenCanvas(1,1);
    editor={kind,value:'',id:'',style:new Proxy({}, {set(o,k,v){o[k]=v;send('editorStyle',{key:k,value:v});return true}}),
     setAttribute(k,v){if(k==='aria-label')this.label=v},addEventListener:(type,fn)=>listen('editor',type,fn),
     focus:()=>send('editorFocus'),blur:()=>send('editorBlur'),remove(){send('editorRemove');editor=null;for(const k of listeners.keys())if(k.startsWith('editor:'))listeners.delete(k)}};return editor;}};
  self.bhooPublish=d=>{send('debug',{data:{ready:d.ready,errors:d.errors,seen:d.seen,items:status.items?undefined:d.items,nodes:d.nodes.map(({item,...n})=>n),slots:d.slots,offset:d.offset,total:d.total,cacheHits:d.cacheHits,tileBytes:d.tileBytes,background:d.background,paletteMoving:d.paletteMoving,frameMs:d.frameMs,maxQueue:d.maxQueue,contexts:d.contexts,worker:true,resources:performance.getEntriesByType("resource").map(r=>r.name)}});status.items=true;};
  if(!self.requestAnimationFrame){self.requestAnimationFrame=fn=>setTimeout(()=>fn(performance.now()),16);self.cancelAnimationFrame=clearTimeout;}
  try{await import('./canvas-runtime.js');}catch(e){send('failure',{error:String(e)})}
 } else if(m.type==='event'){
  if(m.width){self.innerWidth=m.width;self.innerHeight=m.height;self.devicePixelRatio=m.dpr;}
  if(m.target==='document')document.hidden=m.hidden;
  if(m.target==='editor'&&editor)editor.value=m.value;
  const e={...m.event,preventDefault(){},target:m.editorTarget?editor:offscreen};dispatch(m.target,m.name,e);
 } else if(m.type==='dialogAction')self.bhooDialogAction?.(m.kind,m.id,m.context,m.key);
 else if(m.type==='debugCall')self.bhooCanvas?.[m.method]?.(...m.args);
 else if(m.type==='debugSet'&&self.bhooCanvas)self.bhooCanvas[m.key]=m.value;
};
self.addEventListener('error',e=>send('failure',{error:e.message}));
