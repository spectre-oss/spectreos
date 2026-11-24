import {createTimePickerAccessibility} from './time-picker-accessibility.js';
import {createCarouselAccessibility} from './carousel-accessibility.js';
import {createDialogAccessibility} from './dialog-accessibility.js';
import {createLiveStatus} from './live-status.js';
// Keep DOM/IME and trusted pointer capture on the main thread; render in worker.
let fallingBack=false;
let canvas=document.querySelector('canvas'),worker,editor,ready=false;
const removers=[];
function on(target,name,fn,options){target.addEventListener(name,fn,options);removers.push(()=>target.removeEventListener(name,fn,options))}
async function fallback(){if(fallingBack)return;fallingBack=true;worker?.terminate();worker=null;for(const remove of removers)remove();editor?.remove();const replacement=canvas.cloneNode();canvas.replaceWith(replacement);canvas=replacement;await import('./canvas-runtime.js');}
if(!canvas.transferControlToOffscreen||typeof Worker==='undefined'||new URLSearchParams(location.search).has('main')){
 await import('./canvas-runtime.js');
}else{
 try{
  worker=new Worker(new URL('./canvas-worker.js',import.meta.url),{type:'module'});
  const offscreen=canvas.transferControlToOffscreen();
  const snapshot={ready:false,errors:[],seen:new Set(),items:[],nodes:[],worker:true};
  let syncLiveStatus=null,syncDialogAccessibility=null,syncCarouselAccessibility=null,syncTimePickerAccessibility=null,carouselKeyboardPending=false;
  const dialogAction=(kind,id,context,key)=>worker?.postMessage({type:'dialogAction',kind,id,context,key});
  for(const method of ['go','goItem'])snapshot[method]=(...args)=>worker?.postMessage({type:'debugCall',method,args});
  window.bhooCanvas=new Proxy(snapshot,{set(o,key,value){o[key]=value;worker?.postMessage({type:'debugSet',key,value});return true}});
  const emit=(target,name,e,extra={})=>worker?.postMessage({type:'event',target,name,event:Object.fromEntries(['clientX','clientY','button','buttons','pointerId','pointerType','isPrimary','deltaY','deltaMode','key','shiftKey','timeStamp'].map(k=>[k,e[k]])),editorTarget:e.target===editor,...extra});
  worker.onmessage=({data:m})=>{
   if(m.type==='debug'){
    for(const [k,v]of Object.entries(m.data))if(v!==undefined)snapshot[k]=v;ready=m.data.ready;
    if(ready&&!syncLiveStatus){syncLiveStatus=createLiveStatus();syncDialogAccessibility=createDialogAccessibility(canvas,{separate:true,focus:(id,context)=>dialogAction('focus',id,context),activate:(id,context)=>dialogAction('activate',id,context),escape:context=>dialogAction('escape',null,context)});syncTimePickerAccessibility=createTimePickerAccessibility(canvas,{allow:context=>snapshot.items[Math.floor((context-1000)/2)]?.slug==='time-picker',focus:(id,context)=>dialogAction('focus',id,context),key:(id,context,key)=>dialogAction('key',id,context,key),activate:(id,context)=>dialogAction('activate',id,context)});syncCarouselAccessibility=createCarouselAccessibility(canvas,{focus:(id,context)=>dialogAction('focus',id,context),activate:(id,context)=>dialogAction('activate',id,context),key:(id,context,key)=>dialogAction('key',id,context,key),tab:(id,context,shift)=>{carouselKeyboardPending=true;dialogAction('tab',id,context,shift)},shouldTransfer:()=>carouselKeyboardPending,transferred:()=>{carouselKeyboardPending=false}});}
    if(ready&&m.data.nodes){syncLiveStatus(m.data.nodes);syncDialogAccessibility(m.data.nodes);syncCarouselAccessibility(m.data.nodes);syncTimePickerAccessibility(m.data.nodes);}
   }
   else if(m.type==='status')document.querySelector('#status')[m.key]=m.value;
   else if(m.type==='focus')canvas.focus({preventScroll:true});
   else if(m.type==='editor'){
    editor?.remove();editor=document.createElement(m.kind);editor.id='editor';editor.value=m.value;editor.setAttribute('aria-label',m.label||'Text');Object.assign(editor.style,m.style);document.body.append(editor);
    for(const name of ['input','blur'])editor.addEventListener(name,e=>emit('editor',name,e,{value:editor.value}));
    editor.addEventListener('wheel',e=>{e.preventDefault();emit('editor','wheel',e)},{passive:false});
   }else if(m.type==='editorStyle'&&editor)editor.style[m.key]=m.value;
   else if(m.type==='editorFocus')editor?.focus();else if(m.type==='editorBlur')editor?.blur();else if(m.type==='editorRemove'){editor?.remove();editor=null;}
   else if(m.type==='failure'){if(!ready)fallback();else{snapshot.errors.push(m.error);console.error(m.error);}}
  };
  worker.onerror=e=>{if(!ready){e.preventDefault();fallback();}};
  for(const name of ['pointerdown','pointermove','pointerup','pointercancel','pointerleave'])on(canvas,name,e=>{if(name==='pointerdown'){carouselKeyboardPending=false;canvas.focus({preventScroll:true});canvas.setPointerCapture(e.pointerId)}emit('canvas',name,e)});
  on(canvas,'wheel',e=>{e.preventDefault();emit('canvas','wheel',e)},{passive:false});
  on(window,'keydown',e=>{if(e.target===canvas)carouselKeyboardPending=e.key==='Tab';if((e.target!==editor||['Escape','Tab'].includes(e.key))&&['Tab','Enter',' ','ArrowLeft','ArrowRight','ArrowUp','ArrowDown','Home','End','Escape','ContextMenu','PageUp','PageDown'].includes(e.key))e.preventDefault();emit('window','keydown',e)});
  on(window,'resize',e=>emit('window','resize',e,{width:innerWidth,height:innerHeight,dpr:devicePixelRatio}));on(window,'blur',e=>emit('window','blur',e));
  on(document,'visibilitychange',e=>emit('document','visibilitychange',e,{hidden:document.hidden}));
  setTimeout(()=>{if(!ready&&!fallingBack)fallback()},15000);
  worker.postMessage({type:'init',canvas:offscreen,width:innerWidth,height:innerHeight,dpr:devicePixelRatio,reduced:matchMedia('(prefers-reduced-motion:reduce)').matches},[offscreen]);
 }catch(e){await fallback();}
}
