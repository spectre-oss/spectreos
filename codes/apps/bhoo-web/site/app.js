import {createTimePickerAccessibility} from './time-picker-accessibility.js';
import {createCarouselAccessibility} from './carousel-accessibility.js';
import {createDialogAccessibility} from './dialog-accessibility.js';
import {createLiveStatus} from './live-status.js';
const $ = (s) => document.querySelector(s);
const embedQuery=new URLSearchParams(location.search),embedded=embedQuery.has('embed'),embedRun=Number(embedQuery.get('run'));
let embedVisible=true,embedReady=false,sharedPalette=false;
const showcaseMode=embedQuery.get('showcase');
if(showcaseMode)document.documentElement.dataset.showcase=showcaseMode;
let showcaseState={};
function configureShowcase(){if(!engine||!showcaseMode)return;engine.bhoo_showcase_config(showcaseState.selected||0,Number(!!showcaseState.expanded));if(showcaseMode==='code')engine.bhoo_showcase_dialog(1);for(const [kind,value]of [[0,showcaseState.title||''],[1,showcaseState.description||'']]){const bytes=encoder.encode(value);new Uint8Array(engine.memory.buffer,engine.bhoo_edit_buffer(),Math.min(bytes.length,2048)).set(bytes.subarray(0,2048));engine.bhoo_showcase_text(kind,Math.min(bytes.length,2048));}request();}

const sendPreview=(type,data={})=>{if(embedded)parent.postMessage({source:'bhoo-preview',type,run:embedRun,...data},location.origin==='null'?'*':location.origin);};
const syncLiveStatus=createLiveStatus();
const canvas = $('#preview');
let carouselKeyboardPending=false;
const syncCarouselAccessibility=createCarouselAccessibility(canvas,{
  focus(id){engine.bhoo_adapter_focus64(BigInt(id),1);queue();},
  activate(id){engine.bhoo_adapter_focus64(BigInt(id),1);queue({key:2,flags:16});},
  key(id,context,key){engine.bhoo_adapter_focus64(BigInt(id),1);queue({key:keys[key],flags:16});},
  tab(id,context,shift){engine.bhoo_adapter_focus64(BigInt(id),1);carouselKeyboardPending=true;canvas.focus({preventScroll:true});queue({key:1,flags:16|(shift?64:0)});},
  shouldTransfer(){return carouselKeyboardPending;},
  transferred(){carouselKeyboardPending=false;}
});
const syncTimePickerAccessibility=createTimePickerAccessibility(canvas,{
  allow(){return current?.slug==='time-picker'&&selectedTab==='examples';},
  focus(id){engine.bhoo_adapter_focus64(BigInt(id),1);queue();},
  key(id,context,key){engine.bhoo_adapter_focus64(BigInt(id),1);queue({key:keys[key],flags:16});},
  activate(id){engine.bhoo_adapter_focus64(BigInt(id),1);queue({key:2,flags:16});}
});
const syncDialogAccessibility=createDialogAccessibility(canvas,{
  focus(id){engine.bhoo_adapter_focus(Number(id),1);queue();},
  activate(id){engine.bhoo_adapter_focus(Number(id),1);queue({key:2,flags:16});queue({key:2,flags:0});},
  escape(){queue({key:10,flags:16});}
});
const ctx = canvas.getContext('2d');
const decoder = new TextDecoder();
const roleNames = ['button','checkbox','radio group','radio','switch','slider','tab list','tab','list','list item','progress','tooltip','group','text field','menu','menu item','link','toggle button','status','dialog','dialog description','carousel','carousel item'];
const keys = {Tab:1,Enter:2,' ':3,ArrowLeft:4,ArrowRight:5,ArrowUp:6,ArrowDown:7,Home:8,End:9,Escape:10,ContextMenu:11,PageUp:12,PageDown:13};
let catalog, engine, current, selectedTab = 'examples', raf = 0, timer = 0, frames = 0;
const motionMedia=matchMedia('(prefers-reduced-motion: reduce)');
let motionOverride=false;
let pointerKind=0;
let pointer = [NaN,NaN], events = [], settings = {disabled:false,reduced:motionMedia.matches,dark:false,variant:1,expressive:false};
let lastState = '', toastTimer;
let adapter = null;
const xmlAdapters = new Map();
function clearXmlAdapters(){for(const input of xmlAdapters.values())input.remove();xmlAdapters.clear();}
const imageCache=new Map();let imageCacheBytes=0;
const encoder = new TextEncoder();
let calendarLocaleKey=null;
function installCalendarLocale(){
  const locale=navigator.language||'en-US';if(calendarLocaleKey===locale)return;
  const dateFormat=new Intl.DateTimeFormat(locale,{calendar:'gregory',timeZone:'UTC',month:'long',year:'numeric'});
  const weekdayFormat=new Intl.DateTimeFormat(locale,{calendar:'gregory',timeZone:'UTC',weekday:'narrow'});
  const dateHeaderFormat=new Intl.DateTimeFormat(locale,{calendar:'gregory',timeZone:'UTC',weekday:'short',month:'short',day:'numeric'});
  const example=new Date(Date.UTC(2024,0,1));
  const template=dateFormat.formatToParts(example).map(p=>p.type==='month'?'{month}':p.type==='year'?'{year}':p.value).join('');
  const headerTemplate=dateHeaderFormat.formatToParts(example).map(p=>['weekday','month','day'].includes(p.type)?'{'+p.type+'}':p.value).join('');
  const dateOrder=new Intl.DateTimeFormat(locale,{calendar:'gregory',timeZone:'UTC',month:'short',day:'2-digit',year:'numeric'}).formatToParts(example).filter(p=>['month','day','year'].includes(p.type)).map(p=>({month:'M',day:'D',year:'Y'})[p.type]).join('');
  const months=Array.from({length:12},(_,i)=>dateFormat.formatToParts(new Date(Date.UTC(2024,i,1))).find(p=>p.type==='month')?.value||String(i+1));
  const weekdays=Array.from({length:7},(_,i)=>weekdayFormat.format(new Date(Date.UTC(2024,8,1+i))));
  const shortMonths=Array.from({length:12},(_,i)=>dateHeaderFormat.formatToParts(new Date(Date.UTC(2024,i,1))).find(p=>p.type==='month')?.value||String(i+1));
  const shortWeekdays=Array.from({length:7},(_,i)=>dateHeaderFormat.formatToParts(new Date(Date.UTC(2024,8,1+i))).find(p=>p.type==='weekday')?.value||'');
  const periodFormat=new Intl.DateTimeFormat(locale,{calendar:'gregory',timeZone:'UTC',hour:'numeric',minute:'2-digit',hour12:true});
  const hour24Format=new Intl.DateTimeFormat(locale,{calendar:'gregory',timeZone:'UTC',hour:'numeric',minute:'2-digit',hour12:false});
  const periods=[1,13].map(hour=>periodFormat.formatToParts(new Date(Date.UTC(2024,0,1,hour))).find(p=>p.type==='dayPeriod')?.value|| (hour<12?'AM':'PM'));
  const periodParts=periodFormat.formatToParts(new Date(Date.UTC(2024,0,1,1)));const periodIndex=periodParts.findIndex(p=>p.type==='dayPeriod'),hourIndex=periodParts.findIndex(p=>p.type==='hour'),minuteIndex=periodParts.findIndex(p=>p.type==='minute');const periodFirst=periodIndex>=0&&periodIndex<hourIndex;const sep=hourIndex>=0&&minuteIndex>hourIndex?periodParts.slice(hourIndex+1,minuteIndex).map(p=>p.value).join(''):':';const resolved12=periodFormat.resolvedOptions(),resolved24=hour24Format.resolvedOptions();
  const numberFormat=new Intl.NumberFormat(locale,{useGrouping:false});const digits=Array.from({length:10},(_,i)=>numberFormat.format(i));
  let weekStart=7,direction='ltr';try{const info=new Intl.Locale(locale);weekStart=(typeof info.getWeekInfo==='function'?info.getWeekInfo():info.weekInfo)?.firstDay||7;direction=(typeof info.getTextInfo==='function'?info.getTextInfo():info.textInfo)?.direction||(['Arab','Hebr','Thaa','Nkoo','Adlm','Syrc','Rohg','Samr','Mand'].includes(info.maximize().script)?'rtl':'ltr')}catch{}
  const androidFirst=weekStart%7+1;const bytes=encoder.encode([androidFirst,direction,template,headerTemplate,dateOrder,periodFirst?'first':'last',resolved12.hourCycle||'h12',resolved24.hourCycle||'h23',resolved12.hour==='2-digit'?'1':'0',resolved24.hour==='2-digit'?'1':'0',sep,...months,...weekdays,...digits,...shortMonths,...shortWeekdays,...periods].join('\n'));
  if(bytes.length<=2048){new Uint8Array(engine.memory.buffer,engine.bhoo_edit_buffer(),bytes.length).set(bytes);engine.bhoo_calendar_locale_commit(bytes.length);calendarLocaleKey=locale;}
}

function commitText(text) {
  const bytes=encoder.encode(text); if(bytes.length>2048)return false;
  new Uint8Array(engine.memory.buffer,engine.bhoo_edit_buffer(),bytes.length).set(bytes);
  engine.bhoo_edit_commit(bytes.length); queue(); return true;
}
function syncXmlAdapters(semantics) {
  const fields=semantics.filter(n=>n.role==='text field');
  const ids=new Set(fields.map(n=>n.id));
  for(const [id,input] of xmlAdapters)if(!ids.has(id)){input.remove();xmlAdapters.delete(id)}
  for(const node of fields){
    let input=xmlAdapters.get(node.id);const created=!input;
    const id=BigInt(node.id);
    const read=()=>decoder.decode(new Uint8Array(engine.memory.buffer,engine.bhoo_xml_text_pointer(id),engine.bhoo_xml_text_len(id)));
    if(!input){
      input=document.createElement('input');
      const kind=engine.bhoo_xml_input_type(id);
      input.type=kind===4?'password':kind===1?'email':kind===2?'tel':'text';
      input.inputMode=kind===3?'numeric':kind===5?'decimal':kind===2?'tel':kind===1?'email':'text';
      input.className='native-adapter';input.spellcheck=false;input.autocomplete='off';if(current.slug==='android-dialer-filter')input.dataset.dialer='1';if(['android-number-picker','android-date-picker-spinner','android-date-picker-combined','android-time-picker-spinner'].includes(current.slug))input.dataset.numberPicker='1';if(current.slug==='android-time-picker-clock')input.dataset.clockTime='1';if(current.slug==='android-search-view')input.dataset.searchView='1';
      if(['android-dialer-filter','android-edittext','android-autocomplete','android-multi-autocomplete','android-linear-layout'].includes(current.slug))input.dataset.canvasText='1';
      input.addEventListener('focus',()=>{events=events.filter(e=>!(e.flags&8));engine.bhoo_adapter_focus64(id,1);request()});
      input.addEventListener('blur',()=>{if(input.dataset.numberPicker==='1')engine.bhoo_xml_number_end(id,input.dataset.cancel==='1'?0:1);engine.bhoo_adapter_focus64(id,0);request()});
      const commitXmlInput=()=>{const bytes=encoder.encode(input.value);if(bytes.length>2048){input.value=read();notify('Text is limited to 2048 UTF-8 bytes.');return}new Uint8Array(engine.memory.buffer,engine.bhoo_edit_buffer(),bytes.length).set(bytes);engine.bhoo_xml_edit_commit(id,bytes.length);engine.bhoo_xml_edit_selection(id,input.selectionEnd??input.value.length);queue()};
      input.addEventListener('compositionstart',()=>{if(input.dataset.dialer==='1')input.dataset.composing='1'});
      input.addEventListener('compositionend',()=>{if(input.dataset.dialer==='1'){delete input.dataset.composing;commitXmlInput()}});
      input.addEventListener('input',()=>{if(input.dataset.composing!=='1')commitXmlInput()});
      input.addEventListener('select',()=>{engine.bhoo_xml_edit_selection(id,input.selectionEnd??input.value.length);request()});
      input.addEventListener('keydown',event=>{
        if(input.dataset.searchView==='1'&&['ArrowUp','ArrowDown','Escape'].includes(event.key)&&engine.bhoo_xml_autocomplete_open(id)){queue({key:keys[event.key],flags:16|(event.repeat?32:0)});event.preventDefault();return;}
        if(input.dataset.searchView==='1'&&event.key==='Enter'&&engine.bhoo_xml_autocomplete_highlighted(id)){queue({key:keys[event.key],flags:16});event.preventDefault();return;}
        if(input.dataset.searchView==='1'&&event.key==='Enter'){engine.bhoo_xml_search_submit(id);input.blur();canvas.focus({preventScroll:true});queue();event.preventDefault();return;}
        if(input.dataset.clockTime==='1'&&event.key==='Enter'){input.blur();canvas.focus({preventScroll:true});event.preventDefault();return;}
        if(input.dataset.numberPicker==='1'&&(event.key==='Enter'||event.key==='Escape')){if(event.key==='Escape')input.dataset.cancel='1';input.blur();canvas.focus({preventScroll:true});event.preventDefault();return;}
        if(['ArrowUp','ArrowDown','Enter','Escape'].includes(event.key)&&engine.bhoo_xml_autocomplete_open(id)){
          queue({key:keys[event.key],flags:16|(event.repeat?32:0)});event.preventDefault();return;
        }
        if(event.key==='Escape'){input.blur();canvas.focus();event.preventDefault()}
      });
      canvas.parentElement.append(input);xmlAdapters.set(node.id,input);
    }
    const [x,y,w,h]=node.bounds;
    Object.assign(input.style,{left:(canvas.offsetLeft+x+2)+'px',top:(canvas.offsetTop+y+2)+'px',width:Math.max(0,w-4)+'px',height:Math.max(0,h-4)+'px',opacity:'1',paddingTop:node.filled?'22px':'14px',paddingBottom:node.filled?'6px':'14px'});
    input.disabled=!node.enabled;input.setAttribute('aria-label',node.label);
    {const value=read();if(input.value!==value&&input.dataset.composing!=='1'){const old=input.value,start=input.selectionStart,end=input.selectionEnd,dir=input.selectionDirection,focused=document.activeElement===input;input.value=value;if(input.dataset.dialer==='1'&&focused&&start!==null&&end!==null){const map=pos=>Math.min(value.length,old.slice(0,pos).toUpperCase().length);input.setSelectionRange(map(start),map(end),dir||'none');}else if(['android-autocomplete','android-multi-autocomplete'].includes(current.slug)){const end=engine.bhoo_xml_selection_end(id);if(['text','search','url','tel','password'].includes(input.type))input.setSelectionRange(end,end)}}}
    if(input.dataset.searchView==='1'){input.placeholder=node.label;input.style.padding='0 8px';input.style.background='var(--preview)';if(engine.bhoo_xml_search_take_focus(id)&&document.activeElement!==input)input.focus({preventScroll:true});}
    if(input.dataset.clockTime==='1'){input.style.textAlign='center';input.style.padding='0';input.style.fontSize='20px';}
    if(input.dataset.numberPicker==='1'){input.style.textAlign='center';input.style.padding='0';input.style.fontSize='20px';if(created){input.focus({preventScroll:true});input.select();}}
  }
}
function syncAdapter(semantics) {
  if(['android-dialer-filter','android-edittext','android-autocomplete','android-multi-autocomplete','android-linear-layout','android-number-picker','android-date-picker-spinner','android-date-picker-combined','android-time-picker-spinner','android-time-picker-clock','android-search-view'].includes(current.slug) && selectedTab==='examples') { adapter?.remove();adapter=null;syncXmlAdapters(semantics);return; }
  clearXmlAdapters();
  const node=selectedTab==='examples'?semantics.find(n=>n.role==='text field'||n.id==='901'):null;
  if(!node){adapter?.remove();adapter=null;return;}
  const file=node.id==='901';
  const multiline=current.slug==='multiline-text-field';
  if(!adapter||adapter.dataset.family!==current.slug){
    adapter?.remove();adapter=document.createElement(file?'input':multiline?'textarea':'input');
    adapter.dataset.family=current.slug;adapter.className='native-adapter';
    if(!multiline)adapter.type=file?'file':'text';
    if(current.slug==='date-range-picker'){
      adapter.placeholder='Start date – End date';
      adapter.classList.add('date-range-adapter');
    }
    adapter.spellcheck=false; adapter.autocomplete='off';
    adapter.addEventListener('focus',()=>{events=events.filter(e=>!(e.flags&8));engine.bhoo_adapter_focus64(BigInt(node.id),1);request();});
    adapter.addEventListener('blur',()=>{engine.bhoo_adapter_focus64(BigInt(node.id),0);request();});
    adapter.addEventListener(file?'change':'input',()=>{
      const text=file?(adapter.files?.[0]?.name||''):adapter.value;
      if(!commitText(text)&&!file) {adapter.value=decoder.decode(new Uint8Array(engine.memory.buffer,engine.bhoo_text_pointer(),engine.bhoo_text_len()));notify('Text is limited to 2048 UTF-8 bytes.');}
    });
    adapter.addEventListener('keydown',event=>{if(event.key==='Escape'){adapter.blur();canvas.focus();event.preventDefault();}});
    canvas.parentElement.append(adapter);
  }
  const [x,y,w,h]=node.bounds;
  Object.assign(adapter.style,{left:(canvas.offsetLeft+x+2)+'px',top:(canvas.offsetTop+y+2)+'px',width:Math.max(0,w-4)+'px',height:Math.max(0,h-4)+'px',opacity:file?'0':'1'});
  adapter.style.paddingTop=node.filled?'22px':'14px';adapter.style.paddingBottom=node.filled?'6px':'14px';
  adapter.disabled=!node.enabled;adapter.required=Boolean(node.required);adapter.setAttribute('aria-label',node.label||'Choose file');
  adapter.setAttribute('aria-invalid',String(Boolean(node.invalid)));
  if(!file&&document.activeElement!==adapter)adapter.value=decoder.decode(new Uint8Array(engine.memory.buffer,engine.bhoo_text_pointer(),engine.bhoo_text_len()));
}
const debug = window.bhooDemo = {ready:false,frames:0,semantics:[],scene:0,animating:false,errors:[]};
try {settings.dark=localStorage.getItem('bhoo-theme')==='dark';} catch {}
function theme(){document.documentElement.dataset.theme=settings.dark?'dark':'light';$('#theme').setAttribute('aria-label',`Switch to ${settings.dark?'light':'dark'} theme`);request();}
function notify(message){$('#toast').textContent=message;$('#toast').hidden=false;clearTimeout(toastTimer);toastTimer=setTimeout(()=>$('#toast').hidden=true,2200);}
function escape(text){return String(text).replace(/[&<>"']/g,c=>({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[c]));}
function queue(event={}){
  const snapshot={x:pointer[0],y:pointer[1],flags:0,key:0,scroll:0,...event};
  snapshot.flags|=pointerKind;
  const last=events.at(-1);
  if(!(snapshot.flags&127)&&!snapshot.key&&!snapshot.scroll&&last&&!(last.flags&127)&&!last.key&&!last.scroll)events[events.length-1]=snapshot;
  else events.push(snapshot);
  request();
}
function request(){if(engine&&current?.scene!==null&&!raf&&!document.hidden&&embedVisible&&selectedTab==='examples')raf=requestAnimationFrame(render);}
let videoBridge=null,videoBridgePending=false,videoBridgeGeneration=0;
function stopVideoBridge(){videoBridgeGeneration++;videoBridge?.close();videoBridge=null;videoBridgePending=false;}
function ensureVideoBridge(){
  if(videoBridge||videoBridgePending||current?.slug!=='android-video-view')return;
  videoBridgePending=true;const generation=videoBridgeGeneration;
  import('./video-bridge.js').then(({createVideoBridge})=>{
    if(generation!==videoBridgeGeneration||current?.slug!=='android-video-view')return;
    videoBridge=createVideoBridge(engine,()=>current?.slug==='android-video-view'&&!document.hidden&&embedVisible,request,notify);
  }).catch(error=>{if(generation===videoBridgeGeneration)notify(`Video bridge unavailable: ${error.message}`);}).finally(()=>{if(generation===videoBridgeGeneration)videoBridgePending=false;});
}
function takeColor(view,at){return `rgba(${view.getUint8(at)},${view.getUint8(at+1)},${view.getUint8(at+2)},${view.getUint8(at+3)/255})`;}
function paint(packet){
  const v=new DataView(packet.buffer,packet.byteOffset,packet.byteLength);
  if(v.getUint32(0,true)!==0x42484f4f||v.getUint32(4,true)!==1)throw Error('Unsupported Bhoo render packet');
  const count=v.getUint32(8,true),active=Boolean(v.getUint32(12,true)),wake=v.getFloat64(16,true),semanticCount=v.getUint32(24,true);
  let at=32;const renderedText=[],ripples=[],paths=[];
  const rect=()=>{const b=[0,4,8,12].map(offset=>v.getFloat32(at+offset,true));at+=16;return b;};
  const text=()=>{const n=v.getUint32(at,true);at+=4;const value=decoder.decode(packet.subarray(at,at+n));at=(at+n+3)&~3;return value;};
  const w=canvas.clientWidth,h=canvas.clientHeight,dpr=Math.min(devicePixelRatio||1,2);
  const pw=Math.max(1,Math.round(w*dpr)),ph=Math.max(1,Math.round(h*dpr));
  if(canvas.width!==pw||canvas.height!==ph){canvas.width=pw;canvas.height=ph;}
  ctx.setTransform(dpr,0,0,dpr,0,0);ctx.font=embedded?'18px Arial, sans-serif':'18px -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif';ctx.textBaseline='alphabetic';
  for(let i=0;i<count;i++){
    const tag=v.getUint32(at,true);at+=4;
    if(tag===0){ctx.clearRect(0,0,w,h);ctx.fillStyle=takeColor(v,at);at+=4;ctx.fillRect(0,0,w,h);}
    else if(tag===1){const [x,y,bw,bh]=rect(),radius=v.getFloat32(at,true);at+=4;ctx.fillStyle=takeColor(v,at);at+=4;if(bw<=0||bh<=0)continue;ctx.beginPath();ctx.roundRect(x,y,bw,bh,Math.max(0,Math.min(radius,bw/2,bh/2)));ctx.fill();}
    else if(tag===2){const [x,y,bw,bh]=rect();ctx.fillStyle=takeColor(v,at);at+=4;const label=text();renderedText.push(label);if(bw<=0||bh<=0)continue;ctx.save();ctx.beginPath();ctx.rect(x,y,bw,bh);ctx.clip();ctx.fillText(label,x+12,y+(bh-18)*.5+17);ctx.restore();}
    else if(tag===9){
      const [x,y,bw,bh]=rect(),color=takeColor(v,at);at+=4;
      const size=v.getFloat32(at,true),bits=v.getUint32(at+4,true),padding=v.getFloat32(at+8,true),spacing=v.getFloat32(at+12,true);at+=16;
      const label=text();renderedText.push(label);if(bw<=0||bh<=0)continue;
      ctx.save();ctx.beginPath();ctx.rect(x,y,bw,bh);ctx.clip();ctx.fillStyle=color;ctx.font=`${bits&65535} ${size}px Arial, sans-serif`;ctx.letterSpacing=spacing+'px';
      const metrics=ctx.measureText(label),ascent=metrics.fontBoundingBoxAscent??size*.8,descent=metrics.fontBoundingBoxDescent??size*.2;
      const align=bits>>>16;ctx.textAlign=align===1?'center':align===2?'right':'left';ctx.textBaseline='alphabetic';
      ctx.fillText(label,align===1?x+bw/2:align===2?x+bw-padding:x+padding,y+(bh+ascent-descent)/2);ctx.restore();
    }
    else if(tag===10){
      const [x,y,bw,bh]=rect(),radius=v.getFloat32(at,true),dx=v.getFloat32(at+4,true),dy=v.getFloat32(at+8,true),blur=v.getFloat32(at+12,true),spread=v.getFloat32(at+16,true);at+=20;const color=takeColor(v,at);at+=4;
      ctx.save();ctx.shadowOffsetX=dx+10000;ctx.shadowOffsetY=dy;ctx.shadowBlur=blur;ctx.shadowColor=color;ctx.fillStyle="#000";
      ctx.beginPath();ctx.roundRect(x-10000-spread,y-spread,Math.max(0,bw+spread*2),Math.max(0,bh+spread*2),Math.max(0,radius+spread));ctx.fill();ctx.restore();
    }
    else if(tag===13){const origin=v.getFloat32(at,true),scale=v.getFloat32(at+4,true);at+=8;ctx.save();ctx.translate(0,origin);ctx.scale(1,scale);ctx.translate(0,-origin);}
    else if(tag===14){ctx.restore();}
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
    else if(tag===16){const [x,y,bw,bh]=rect(),uv=[0,4,8,12].map(o=>v.getFloat32(at+o,true));at+=16;const radius=v.getFloat32(at,true);at+=4;const alpha=v.getUint8(at+3)/255;at+=4;const id=v.getUint32(at,true),iw=v.getUint32(at+4,true),ih=v.getUint32(at+8,true),ptr=v.getUint32(at+12,true);at+=16;let entry=imageCache.get(id);if(!entry){const bytes=iw*ih*4;if(bytes>16*1024*1024||ptr+bytes>engine.memory.buffer.byteLength)throw Error('Invalid image buffer');while(imageCacheBytes+bytes>16*1024*1024&&imageCache.size){const [key,old]=imageCache.entries().next().value;imageCache.delete(key);imageCacheBytes-=old.bytes;}const surface=document.createElement('canvas');surface.width=iw;surface.height=ih;surface.getContext('2d').putImageData(new ImageData(new Uint8ClampedArray(engine.memory.buffer.slice(ptr,ptr+bytes)),iw,ih),0,0);entry={surface,bytes};imageCache.set(id,entry);imageCacheBytes+=bytes;}else{imageCache.delete(id);imageCache.set(id,entry);}ctx.save();ctx.globalAlpha*=alpha;ctx.beginPath();ctx.roundRect(x,y,bw,bh,Math.max(0,Math.min(radius,bw/2,bh/2)));ctx.clip();const flipX=uv[2]<uv[0],flipY=uv[3]<uv[1];ctx.translate(x+(flipX?bw:0),y+(flipY?bh:0));ctx.scale(flipX?-1:1,flipY?-1:1);ctx.drawImage(entry.surface,Math.min(uv[0],uv[2])*iw,Math.min(uv[1],uv[3])*ih,Math.abs(uv[2]-uv[0])*iw,Math.abs(uv[3]-uv[1])*ih,0,0,bw,bh);ctx.restore();}
    else throw Error(`Unknown drawing command ${tag}`);
  }
  const semantics=[];
  for(let i=0;i<semanticCount;i++){
    const id=v.getBigUint64(at,true).toString();at+=8;
    const part=v.getUint32(at,true),role=v.getUint32(at+4,true);at+=8;
    const bounds=rect(),flags=v.getUint32(at,true),checked=v.getUint32(at+4,true),value=v.getFloat32(at+8,true),min=v.getFloat32(at+12,true),max=v.getFloat32(at+16,true);at+=20;
    const label=text();semantics.push({id,part,role:roleNames[role]||'unknown',label,bounds,enabled:Boolean(flags&1),focused:Boolean(flags&2),selected:Boolean(flags&4),invalid:Boolean(flags&8),filled:Boolean(flags&16),required:Boolean(flags&32),checked:checked?['','unchecked','checked','mixed'][checked]:null,value:Number.isFinite(value)?value:null,min,max});
  }
  syncAdapter(semantics);
  syncLiveStatus(semantics);
  syncDialogAccessibility(semantics);
  syncCarouselAccessibility(semantics);
  syncTimePickerAccessibility(semantics);
  debug.paths=paths;
  debug.ripples=ripples;
  debug.semantics=semantics;debug.text=renderedText;debug.animating=active;debug.commands=count;debug.frames=++frames;
  $('#engine-status').textContent=active?'Animating':'Settled';$('#command-count').textContent=`${count} commands`;
  const meaningful=semantics.filter(n=>n.focused||n.checked||n.value!==null||n.selected).slice(0,5);
  const state=meaningful.length?meaningful.map(n=>`<div class="state-line"><span>${escape(n.label.slice(0,20))}</span><strong>${escape(n.checked??(n.value!==null?(n.max===1?Math.round(n.value*100)+'%':Math.round(n.value)):n.focused?'focused':'selected'))}</strong></div>`).join(''):'<p>Hover, click, or use your keyboard to explore.</p>';
  if(state!==lastState){$('#state-readout').innerHTML=state;lastState=state;}
  return {active,wake};
}
function render(time){
  raf=0;clearTimeout(timer);
  if(!engine||current?.scene===null||document.hidden||!embedVisible||selectedTab!=='examples')return;
  try{
    document.documentElement.dataset.motion=settings.reduced?'reduced':'full';
    const fromEvent=events.length>0;
    const input=events.shift()||{x:pointer[0],y:pointer[1],flags:pointerKind,key:0,scroll:0};
    const bits=Number(settings.disabled)|(Number(settings.reduced)<<1)|(Number(settings.dark)<<2)|(Number(settings.expressive)<<3)|(Number(sharedPalette)<<4)|(Number(embedded)<<5)|(settings.variant<<8);
    if(['android-calendar-view','android-date-picker','android-date-picker-spinner','android-date-picker-combined','android-number-picker','android-time-picker-spinner','android-time-picker-clock'].includes(current.slug))installCalendarLocale();
    if(['android-textclock','android-analogclock','android-digitalclock','android-calendar-view','android-date-picker','android-date-picker-spinner','android-date-picker-combined','android-time-picker-spinner','android-time-picker-clock'].includes(current.slug)) {
      const date=new Date();
      const cycle=new Intl.DateTimeFormat(undefined,{hour:'numeric'}).resolvedOptions().hourCycle;
      engine.bhoo_wall_clock(date.getTime()/1000,-date.getTimezoneOffset()*60,Number(cycle==='h23'||cycle==='h24'));
    }
    if(current.slug==='android-video-view')videoBridge?.sync();
    if(current.scene===10009){const date=new Date();engine.wisp_wall_time?.(date.getTime(),-date.getTimezoneOffset()*60);}
    const pointerOffset=engine.bhoo_frame(current.scene,canvas.clientWidth,canvas.clientHeight,time/1000,input.x,input.y,input.flags,input.key,input.scroll,bits);
    const packet=new Uint8Array(engine.memory.buffer,pointerOffset,engine.bhoo_packet_len());
    const result=paint(packet);$('#loading').hidden=true;
    if(current.slug==='android-video-view'){ensureVideoBridge();videoBridge?.consume();}
    if(showcaseMode){const action=engine.bhoo_showcase_action();if(action)sendPreview('showcase-action',{action});}
    if(embedded&&!embedReady){embedReady=true;sendPreview('ready',{component:current.slug});}
    if(result.active||events.length||fromEvent)request();
    else if(result.wake>=0)timer=setTimeout(request,Math.max(1,result.wake*1000-performance.now()));
  }catch(error){debug.errors.push(String(error));sendPreview('error',{message:String(error)});$('#loading').hidden=false;$('#loading').classList.add('error');$('#loading').textContent=`The Rust preview could not render: ${error.message}`;console.error(error);}
}
function nav(){
  const query=$('#search').value.toLowerCase().trim();
  const groups=new Map();
  for(const item of catalog.components){if(!`${item.name} ${item.category}`.toLowerCase().includes(query))continue;if(!groups.has(item.category))groups.set(item.category,[]);groups.get(item.category).push(item);}
  let html=`<a class="nav-link nav-overview ${current?.slug==='overview'?'active':''}" href="#/overview"><span>◈</span> Playground <span class="nav-count">${catalog.live_count} live</span></a>`;
  if(!query)html+=`<div class="nav-heading">Component library <span class="nav-count">${catalog.components.length}</span></div>`;
  for(const [category,items]of groups){html+=`<div class="nav-heading">${escape(category)}</div>`;for(const item of items)html+=`<a class="nav-link ${item.slug===current?.slug?'active':''}" href="#/${item.slug}" ${item.slug===current?.slug?'aria-current="page"':''}><span>${escape(item.name)}</span>${item.scene!==null?'<span class="nav-dot" title="Live partial implementation"></span>':'<span class="planned-label">planned</span>'}</a>`;}
  if(!groups.size)html+='<p class="loading-nav">No matching components.</p>';
  $('#navigation').innerHTML=html;
}
function openTab(tab){
  selectedTab=tab;
  for(const button of document.querySelectorAll('[data-tab]')){const active=button.dataset.tab===tab;button.setAttribute('aria-selected',String(active));button.tabIndex=active?0:-1;$('#panel-'+button.dataset.tab).hidden=!active;}
  cancelAnimationFrame(raf);raf=0;clearTimeout(timer);
  if(tab==='examples')request();else {videoBridge?.suspend();engine?.bhoo_pause();events=[];pointer=[NaN,NaN];adapter?.remove();adapter=null;clearXmlAdapters();}
}
function route(){
  if(!catalog)return;
  stopVideoBridge();
  const slug=catalog.single_component||decodeURIComponent(location.hash.replace(/^#\/?/,'')||'button');
  current=showcaseMode?{...catalog.overview,slug:'showcase-'+showcaseMode,name:'Bhoo showcase',scene:({rail:10000,toolbar:10001,hero:10002,heading:10003,code:10004,mobile:10005})[showcaseMode]}:slug==='overview'?catalog.overview:catalog.components.find(c=>c.slug===slug);
  if(!current){location.hash='/button';return;}
  adapter?.remove();adapter=null;clearXmlAdapters();
  debug.scene=current.scene;debug.component=current.slug;events=[];pointer=[NaN,NaN];
  if(engine&&current.scene!==null){engine.bhoo_reset(current.scene);calendarLocaleKey=null;}
  $('#title').textContent=current.name;$('#category').textContent=current.category;
  $('#description').textContent=current.summary;$('#implementation-badge').textContent=['state-layer-ripple','expressive-loading-indicator'].includes(current.slug)?'Live · Rust widget':current.scene<19?'Live · Partial implementation':'Live · Experimental composition';
  $('#demo-card').hidden=current.scene===null;$('#planned').hidden=current.scene!==null;
  $('#demo-coverage').replaceChildren();
  if(current.preview){const actual=document.createElement('p'),remaining=document.createElement('p');actual.textContent='In this preview: '+current.preview.demonstrates;remaining.textContent='Remaining scope: '+current.preview.remaining;remaining.className='muted';$('#demo-coverage').append(actual,remaining);}
  $('#state-contract').textContent=current.contract;$('#motion-contract').textContent=current.motion;$('#reduced-contract').textContent=current.reduced;
  $('#implementation-status').textContent=current.status+(current.preview?' Demonstrates: '+current.preview.demonstrates+' Remaining: '+current.preview.remaining:'');$('#code').textContent=current.code||'// No live Bhoo implementation for this family yet.\n// Read the state, motion and upstream reference contracts before implementation.';
  $('#code-note').textContent=current.scene>=19?'This experimental example is a Gallery composition in codes/apps/bhoo-web/src/gallery.rs, using Bhoo’s public primitives. Download the source below for the full state and helpers.':current.scene!==null?'This example uses the real Bhoo API. Its browser host compiles Rust to WebAssembly and paints draw commands with Canvas 2D. Native Wayland uses the GLES renderer.':'This is a planned family. No placeholder JavaScript widget is presented as an implemented Bhoo component.';
  $('#checks').replaceChildren(...current.checks.map(check=>{const li=document.createElement('li');li.textContent=check;return li;}));
  $('#reference-list').innerHTML=current.references.map(({label,url})=>`<a class="reference-link" href="${escape(url)}" target="_blank" rel="noopener"><strong>${escape(label)}</strong><span>Open reference ↗</span></a>`).join('')||'<p class="muted">Reference details are available in the repository TDD.</p>';
  $('#variant').disabled=![0,1,2].includes(current.scene);$('#loading').hidden=Boolean(engine);$('#loading').classList.remove('error');
  const index=catalog.components.findIndex(c=>c.slug===slug);
  for(const [selector,item]of [['#previous',index>0?catalog.components[index-1]:catalog.overview],['#next',catalog.components[(Math.max(index,0)+1)%catalog.components.length]]]){
    $(selector).href='#/'+item.slug;$(selector+' strong').textContent=item.name;
  }
  nav();openTab('examples');configureShowcase();document.title=`${current.name} — Bhoo`;document.body.classList.remove('sidebar-open');$('#menu-toggle').setAttribute('aria-expanded','false');
  window.scrollTo({top:0,behavior:'instant'});
}
canvas.addEventListener('pointermove',event=>{if(!event.isPrimary)return;pointerKind=event.pointerType==='touch'?128:event.pointerType==='pen'?256:0;const b=canvas.getBoundingClientRect();pointer=[event.clientX-b.left,event.clientY-b.top];queue();});
canvas.addEventListener('pointerdown',event=>{carouselKeyboardPending=false;if(event.button!==0||!event.isPrimary)return;pointerKind=event.pointerType==='touch'?128:event.pointerType==='pen'?256:0;canvas.focus({preventScroll:true});const b=canvas.getBoundingClientRect();pointer=[event.clientX-b.left,event.clientY-b.top];canvas.setPointerCapture(event.pointerId);queue({flags:1});event.preventDefault();});
canvas.addEventListener('pointerup',event=>{if(event.button!==0||!event.isPrimary)return;const b=canvas.getBoundingClientRect();pointer=[event.clientX-b.left,event.clientY-b.top];queue({flags:2});if(canvas.hasPointerCapture(event.pointerId))canvas.releasePointerCapture(event.pointerId);});
canvas.addEventListener('pointercancel',event=>{if(event.isPrimary)queue({flags:4});});
canvas.addEventListener('pointerleave',event=>{if(!event.isPrimary||event.pointerType==='touch'||canvas.hasPointerCapture(event.pointerId))return;pointer=[NaN,NaN];queue({flags:4});});
canvas.addEventListener('blur',event=>{if(!event.relatedTarget?.dataset?.bhooDialogControl&&!event.relatedTarget?.dataset?.bhooCarouselItem)queue({flags:8});});
for(const type of ['keydown','keyup'])canvas.addEventListener(type,event=>{
  if(type==='keydown')carouselKeyboardPending=event.key==='Tab';
  if(type==='keydown'&&(current?.slug==='select'||(['menu','popover','context-menu-integration','bottom-app-bar','toolbar','floating-toolbar'].includes(current?.slug)&&engine?.bhoo_overlay_open()))&&!event.ctrlKey&&!event.metaKey&&!event.altKey&&Array.from(event.key).length===1&&event.key!==' '){queue({key:256+event.key.codePointAt(0),flags:16|(event.repeat?32:0)});event.preventDefault();return;}
  if(event.key==='F10'&&event.shiftKey){queue({key:11,flags:16});event.preventDefault();return;}
  if(!keys[event.key])return;
  if(event.key==='Escape'&&type==='keyup'){event.preventDefault();return;}
  if(event.key==='Escape'&&!engine.bhoo_overlay_open()){canvas.blur();event.preventDefault();return;}
  queue({key:keys[event.key],flags:(type==='keydown'?16:0)|(event.repeat?32:0)|(event.shiftKey?64:0)});event.preventDefault();
});
canvas.addEventListener('wheel',event=>{const b=canvas.getBoundingClientRect(),x=event.clientX-b.left,y=event.clientY-b.top;const carousel=current?.slug==='carousel'&&debug.semantics.some(n=>n.id==='130'&&x>=n.bounds[0]&&y>=n.bounds[1]&&x<n.bounds[0]+n.bounds[2]&&y<n.bounds[1]+n.bounds[3]);const appbarScroll=(['top-app-bar','bottom-app-bar','toolbar','floating-toolbar'].includes(current?.slug))&&debug.semantics.some(n=>n.id===(['toolbar','floating-toolbar'].includes(current?.slug)?'900':'130')&&x>=n.bounds[0]&&y>=n.bounds[1]&&x<n.bounds[0]+n.bounds[2]&&y<n.bounds[1]+n.bounds[3]);const stackScroll=current?.slug==='android-stack-view'&&debug.semantics.some(n=>n.id==='163471922067871'&&x>=n.bounds[0]&&y>=n.bounds[1]&&x<n.bounds[0]+n.bounds[2]&&y<n.bounds[1]+n.bounds[3]);if(!['rail','code'].includes(showcaseMode)&&!carousel&&!appbarScroll&&!stackScroll&&![11,...catalog.components.filter(c=>['virtualized-grid','scrollbar','android-horizontal-scroll'].includes(c.slug)).map(c=>c.scene)].includes(current?.scene)&&!(['time-picker','floating-action-menu','toolbar','floating-toolbar','button-group'].includes(current?.slug)&&engine?.bhoo_overlay_open()))return;pointer=[x,y];const horizontal=current?.slug==='android-horizontal-scroll'&&Math.abs(event.deltaX)>Math.abs(event.deltaY);const delta=(horizontal||carousel&&Math.abs(event.deltaX)>Math.abs(event.deltaY))?event.deltaX:event.deltaY;queue({scroll:delta*(event.deltaMode===1?40:1),flags:horizontal?512:0});event.preventDefault();},{passive:false});
canvas.addEventListener('contextmenu',event=>{if(current.slug!=='context-menu-integration')return;event.preventDefault();const b=canvas.getBoundingClientRect();pointer=[event.clientX-b.left,event.clientY-b.top];queue({key:11,flags:16});});
new ResizeObserver(()=>request()).observe(canvas);
document.addEventListener('visibilitychange',()=>{if(document.hidden){videoBridge?.suspend();engine?.bhoo_pause();events=[];pointer=[NaN,NaN];cancelAnimationFrame(raf);raf=0;clearTimeout(timer);}else request();});
$('#search').addEventListener('input',nav);
document.addEventListener('keydown',event=>{if((event.key==='/'||(event.key==='k'&&(event.metaKey||event.ctrlKey)))&&!['INPUT','TEXTAREA','SELECT'].includes(document.activeElement.tagName)&&document.activeElement!==canvas){event.preventDefault();$('#search').focus();}});
for(const button of document.querySelectorAll('[data-tab]')){
  button.addEventListener('click',()=>openTab(button.dataset.tab));
  button.addEventListener('keydown',event=>{if(!['ArrowLeft','ArrowRight','Home','End'].includes(event.key))return;const tabs=[...document.querySelectorAll('[data-tab]')];let i=tabs.indexOf(button);i=event.key==='Home'?0:event.key==='End'?tabs.length-1:(i+(event.key==='ArrowRight'?1:tabs.length-1))%tabs.length;openTab(tabs[i].dataset.tab);tabs[i].focus();event.preventDefault();});
}
$('#reset').addEventListener('click',()=>{if(engine&&current?.scene!==null){stopVideoBridge();engine.bhoo_reset(current.scene);calendarLocaleKey=null;events=[];queue();notify('Example reset');}});
$('#disabled').addEventListener('change',event=>{settings.disabled=event.target.checked;queue({flags:4});});
$('#reduced').checked=settings.reduced;$('#reduced').addEventListener('change',event=>{motionOverride=true;settings.reduced=event.target.checked;request();});
motionMedia.addEventListener('change',event=>{if(!motionOverride){settings.reduced=event.matches;$('#reduced').checked=event.matches;request();}});
$('#motion-scheme').addEventListener('change',event=>{settings.expressive=event.target.value==='expressive';request();});
$('#variant').addEventListener('change',event=>{settings.variant=Number(event.target.value);request();});
$('#theme').addEventListener('click',()=>{settings.dark=!settings.dark;try{localStorage.setItem('bhoo-theme',settings.dark?'dark':'light');}catch{}theme();});
$('#copy-code').addEventListener('click',async()=>{try{await navigator.clipboard.writeText($('#code').textContent);notify('Rust example copied');}catch{notify('Select the code to copy it.');}});
$('#show-live').addEventListener('click',()=>location.hash='/button');
$('#menu-toggle').addEventListener('click',()=>{const open=document.body.classList.toggle('sidebar-open');$('#menu-toggle').setAttribute('aria-expanded',String(open));});
$('#mobile-shade').addEventListener('click',()=>{document.body.classList.remove('sidebar-open');$('#menu-toggle').setAttribute('aria-expanded','false');});
window.addEventListener('message',event=>{
  if(!embedded||event.origin!==location.origin||event.source!==parent||event.data?.source!=='bhoo-comparison'||event.data.run!==embedRun)return;
  if(event.data.type==='showcase-config'){showcaseState=event.data.config;configureShowcase();return;}
  if(event.data.type==='settings'){
    const s=event.data.settings;settings.disabled=!!s.disabled;settings.reduced=!!s.reduced;settings.dark=!!s.dark;settings.variant=s.variant;settings.expressive=false;
    sharedPalette=!!s.sharedTheme&&Array.isArray(s.colors)&&s.colors.length===22;
    if(sharedPalette&&engine)for(let i=0;i<Math.min(22,s.colors.length);i++){const [r,g,b,a]=s.colors[i];engine.bhoo_palette_set(i,(r|(g<<8)|(b<<16)|(a<<24))>>>0);}
    if(sharedPalette){document.documentElement.style.setProperty('--preview',`rgb(${s.colors[0].slice(0,3).join(',')})`);document.documentElement.style.setProperty('--text',`rgb(${s.colors[1].slice(0,3).join(',')})`);document.documentElement.style.setProperty('--field-primary',`rgb(${s.colors[5].slice(0,3).join(',')})`);}else{document.documentElement.style.removeProperty('--preview');document.documentElement.style.removeProperty('--text');document.documentElement.style.removeProperty('--field-primary');}
    debug.comparisonSettings={...s};theme();queue({flags:4});
  }
  if(event.data.type==='checkbox-state'&&current?.slug==='checkbox'&&engine){const s=event.data.state;events=events.filter(e=>!(e.flags&8));engine.bhoo_comparison_checkbox_set(Number(!!s.binary),s.tri,s.focused);request();}
  if(event.data.type==='visibility'){
    embedVisible=!!event.data.visible;
    if(!embedVisible){videoBridge?.suspend();engine?.bhoo_pause();events=[];pointer=[NaN,NaN];cancelAnimationFrame(raf);raf=0;clearTimeout(timer);}else request();
  }
});
window.addEventListener('hashchange',route);theme();
try{
  const [data,module]=await Promise.all([fetch('catalog.json').then(r=>{if(!r.ok)throw Error('Catalog missing. Run the web build script.');return r.json();}),fetch('bhoo.wasm').then(r=>{if(!r.ok)throw Error('WASM module missing. Run the web build script.');return r.arrayBuffer();}).then(b=>WebAssembly.instantiate(b,{}))]);
  catalog=data;debug.catalog=data;engine=module.instance.exports;
  ctx.save();for(let code=32;code<127;code++){const ch=String.fromCharCode(code);ctx.font='400 100px Arial, sans-serif';const normal=ctx.measureText(ch).width/100;ctx.font='500 100px Arial, sans-serif';engine.bhoo_font_advance(code,normal,ctx.measureText(ch).width/100);}ctx.restore();
  ctx.save();const offsets=[400,700].map(weight=>{ctx.font=`${weight} 100px Arial, sans-serif`;const m=ctx.measureText('Hg');return ((m.fontBoundingBoxAscent??80)-(m.fontBoundingBoxDescent??20))/200;});ctx.restore();engine.bhoo_font_baseline(...offsets);
  debug.ready=true;debug.wasmBytes=data.wasm_bytes;route();
}catch(error){debug.errors.push(String(error));sendPreview('error',{message:String(error)});$('#loading').classList.add('error');$('#loading').textContent=error.message;$('#implementation-badge').textContent='Preview unavailable';console.error(error);}
