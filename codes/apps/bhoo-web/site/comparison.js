import {angularFor,angularUrl} from './comparison-catalog.js';
const $=s=>document.querySelector(s);
const frames={angular:$('#angular'),bhoo:$('#bhoo')};
const media=matchMedia('(prefers-reduced-motion: reduce)');
const settings={disabled:false,reduced:media.matches,dark:false,variant:1,sharedTheme:true};
let catalog,current,reference,colors=null,run=0,activeTab='examples',motionOverride=false;
const ready={angular:false,bhoo:false};
const debug=window.bhooComparison={ready:false,errors:[],settings,reference:null,component:null,run:0};
try{settings.dark=localStorage.getItem('bhoo-comparison-theme')==='dark';}catch{}
const send=(name,type,data={})=>frames[name].contentWindow?.postMessage({source:'bhoo-comparison',type,run,...data},location.origin==='null'?'*':location.origin);
function fail(name,message){debug.errors.push({name,message,component:current?.slug});const el=$('#'+name+'-status');if(el){el.textContent='Failed';el.className='runtime-status failed';}$('#error').textContent=message;$('#error').hidden=false;}
function pushSettings(){
  document.documentElement.dataset.theme=settings.dark?'dark':'light';
  $('#theme').setAttribute('aria-label',`Switch to ${settings.dark?'light':'dark'} theme`);
  for(const name of Object.keys(frames))send(name,'settings',{settings:{...settings,colors}});
}
function status(name){
  ready[name]=true;$('#'+name+'-status').textContent=name==='angular'&&reference.kind==='none'?'No equivalent':'Ready';$('#'+name+'-status').className='runtime-status ready';
  debug.ready=ready.angular&&ready.bhoo;
}
window.addEventListener('message',event=>{
  if(event.origin!==location.origin)return;
  const data=event.data;
  const name=event.source===frames.angular.contentWindow?'angular':event.source===frames.bhoo.contentWindow?'bhoo':null;
  if(!name||data?.source!==(name==='angular'?'bhoo-angular':'bhoo-preview')||data.run!==run)return;
  if(name==='angular'&&data.type==='checkbox-state'&&current?.slug==='checkbox'&&$('#link-checkbox').checked){send('bhoo','checkbox-state',{state:data.state});return;}
  if(data.type==='error'){fail(name,data.message);return;}
  if(name==='angular'&&(data.type==='palette'||data.type==='ready')){colors=data.colors;send('bhoo','settings',{settings:{...settings,colors}});}
  if(data.type==='ready'){
    status(name);send(name,'settings',{settings:{...settings,colors}});
    send(name,'visibility',{visible:activeTab==='examples'&&!document.hidden});
    if(name==='angular')$('#angular-code').textContent=frames.angular.contentWindow.angularReference?.template||'// No direct Angular equivalent.';
  }
});
function escape(text){return String(text).replace(/[&<>"']/g,c=>({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[c]));}
function nav(){
  const query=$('#search').value.trim().toLowerCase();let category='';let html='';
  for(const item of catalog.components){
    if(!`${item.name} ${item.category}`.toLowerCase().includes(query))continue;
    if(category!==item.category){category=item.category;html+=`<div class="nav-category">${escape(category)}</div>`;}
    const ref=angularFor(item.slug);
    html+=`<a class="nav-link ${item.slug===current?.slug?'active':''}" href="#/${item.slug}" ${item.slug===current?.slug?'aria-current="page"':''}><span>${escape(item.name)}</span><span class="nav-indicator ${ref.kind}" title="${ref.kind==='direct'?'Direct Angular equivalent':ref.kind==='related'?'Related Angular pattern':'No Angular equivalent'}"></span></a>`;
  }
  $('#navigation').innerHTML=html||'<p>No matching components.</p>';
}
function openTab(tab){
  activeTab=tab;
  for(const button of document.querySelectorAll('[data-tab]')){const active=button.dataset.tab===tab;button.setAttribute('aria-selected',String(active));button.tabIndex=active?0:-1;$('#panel-'+button.dataset.tab).hidden=!active;}
  for(const name of Object.keys(frames))send(name,'visibility',{visible:tab==='examples'&&!document.hidden});
}
function reset(){
  debug.ready=false;ready.angular=false;ready.bhoo=false;colors=null;debug.run=++run;
  for(const name of Object.keys(frames)){$('#'+name+'-status').textContent='Loading';$('#'+name+'-status').className='runtime-status';}
  frames.angular.src=reference.kind==='beer'?`beercss.html?fixture=${encodeURIComponent(reference.fixture)}&run=${run}`:`angular.html?fixture=${encodeURIComponent(reference.fixture)}&run=${run}`;
  frames.bhoo.src=`playground.html?embed=1&run=${run}#/${current.slug}`;
  $('#error').hidden=true;
}
function route(){
  if(!catalog)return;
  let slug;try{slug=decodeURIComponent(location.hash.replace(/^#\/?/,'')||'button');}catch{slug='button';}
  if(catalog.single_component)slug=catalog.single_component;
  if(slug==='overview'){location.replace('playground.html#/overview');return;}
  current=catalog.components.find(c=>c.slug===slug);if(!current){location.hash='/button';return;}
  reference=angularFor(slug);debug.component=slug;debug.reference=reference;
  $('#title').textContent=current.name;$('#category').textContent=current.category;$('#description').textContent=catalog.update_description||current.summary;
  const beer=reference.kind==='beer';
  $('.angular-icon').textContent=beer?'B':'A';$('.angular-panel header strong').textContent=beer?'Beer CSS':'Angular Material';$('.angular-panel header div span').textContent=beer?'5.0.3 · official components':'22.2.6 · official components';
  $('#angular').title=beer?'Live Beer CSS reference':'Live Angular Material reference';
  $('.section-title p').textContent=beer?'Real Beer CSS on the left. Real Bhoo Rust / WASM on the right.':'Real Angular Material on the left. Real Bhoo Rust / WASM on the right.';
  $('#coverage').textContent=beer?'Beer CSS equivalent · unverified parity':reference.kind==='direct'?'Angular equivalent · unverified parity':reference.kind==='related'?'Related Angular pattern':'No Angular equivalent';
  $('#example-title').textContent=current.name+' comparison';$('#fixture-note').textContent=reference.note;
  $('#pair-warning').textContent=reference.kind==='none'?reference.note:reference.sameContent?'Matching labels and initial values do not imply matching pixels.':'The fixtures differ in layout or content. Overlay is exploratory; do not use a whole-pane difference as a component match score.';
  $('#official').hidden=!reference.doc;if(reference.doc)$('#official').href=beer?reference.doc:angularUrl(reference.doc);
  $('#variant').disabled=slug!=='button';
  $('#demonstrates').textContent=current.preview?'Demonstrates: '+current.preview.demonstrates:current.status;
  $('#remaining').textContent=current.preview?'Remaining: '+current.preview.remaining:'Component and motion parity require further validation.';
  $('#playground').href='playground.html#/'+slug;
  $('#state-contract').textContent=current.contract;$('#motion-contract').textContent=current.motion;$('#reduced-contract').textContent=current.reduced;
  $('#code').textContent=current.code;$('#angular-code').textContent='Loading reference fixture…';
  $('#references').replaceChildren(...current.references.map(ref=>{const a=document.createElement('a');a.href=ref.url;a.textContent=ref.label+' ↗';a.target='_blank';a.rel='noopener';return a;}));
  $('#link-checkbox-label').hidden=slug!=='checkbox';
  document.title=current.name+(beer?' — Beer CSS / Bhoo':' — Angular / Bhoo');nav();openTab('examples');reset();closeNav();window.scrollTo({top:0,behavior:'instant'});
}
function view(){
  const mode=$('#view').value,panel=$('.angular-panel'),viewport=$('#angular-viewport'),mix=Number($('#mix').value);
  $('#previews').dataset.view=mode;$('#overlay-controls').hidden=mode==='side';
  viewport.style.opacity=mode==='overlay'?String(mix/100):'1';
  // Clip the complete Angular panel so its border and background cannot cover Bhoo's half.
  panel.style.clipPath=mode==='wipe'?`inset(0 ${100-mix}% 0 0)`:'none';
  $('#previews').style.setProperty('--wipe',mix+'%');
  $('#mix-label').textContent=mode==='wipe'?'Angular coverage':'Angular opacity';$('#mix-value').textContent=mix+'%';
  // Hidden duplicate content must not receive Tab focus in inspection mode.
  for(const frame of Object.values(frames)){frame.inert=mode!=='side';frame.tabIndex=mode==='side'?0:-1;}
}
function dimensions(){const width=Number($('#width').value);document.documentElement.style.setProperty('--viewport-width',width+'px');$('#dimensions').textContent=`Each viewport: ${width} × 640 CSS px · 100% scale`;debug.width=width;}
function closeNav(){document.body.classList.remove('sidebar-open');$('#menu').setAttribute('aria-expanded','false');$('#shade').hidden=true;}
$('#link-checkbox').addEventListener('change',()=>{if($('#link-checkbox').checked)send('angular','checkbox-state-request');});
$('#menu').addEventListener('click',()=>{const open=document.body.classList.toggle('sidebar-open');$('#menu').setAttribute('aria-expanded',String(open));$('#shade').hidden=!open;});$('#shade').addEventListener('click',closeNav);
$('#search').addEventListener('input',nav);$('#reset').addEventListener('click',reset);$('#view').addEventListener('change',view);$('#mix').addEventListener('input',view);$('#width').addEventListener('change',dimensions);
for(const [id,key] of [['disabled','disabled'],['reduced','reduced'],['shared-theme','sharedTheme']]){
  $('#'+id).checked=settings[key];$('#'+id).addEventListener('change',event=>{settings[key]=event.target.checked;if(key==='reduced')motionOverride=true;pushSettings();});
}
$('#variant').addEventListener('change',event=>{settings.variant=Number(event.target.value);pushSettings();});
$('#theme').addEventListener('click',()=>{settings.dark=!settings.dark;try{localStorage.setItem('bhoo-comparison-theme',settings.dark?'dark':'light');}catch{}pushSettings();});
media.addEventListener('change',event=>{if(!motionOverride){settings.reduced=event.matches;$('#reduced').checked=settings.reduced;pushSettings();}});
for(const button of document.querySelectorAll('[data-tab]')){
  button.addEventListener('click',()=>openTab(button.dataset.tab));
  button.addEventListener('keydown',event=>{if(!['ArrowLeft','ArrowRight','Home','End'].includes(event.key))return;const tabs=[...document.querySelectorAll('[data-tab]')],i=tabs.indexOf(button);const target=event.key==='Home'?0:event.key==='End'?tabs.length-1:(i+(event.key==='ArrowRight'?1:tabs.length-1))%tabs.length;openTab(tabs[target].dataset.tab);tabs[target].focus();event.preventDefault();});
}
window.addEventListener('hashchange',route);
window.addEventListener('keydown',event=>{if(event.key==='/'&&!['INPUT','TEXTAREA','SELECT'].includes(document.activeElement.tagName)){event.preventDefault();$('#search').focus();}if(event.key==='Escape')closeNav();});
window.addEventListener('error',event=>fail('host',event.message));
document.addEventListener('visibilitychange',()=>{for(const name of Object.keys(frames))send(name,'visibility',{visible:!document.hidden&&activeTab==='examples'});});
try{const response=await fetch('catalog.json');if(!response.ok)throw Error('Catalog missing. Run python3 tools/serve-bhoo-web.py');catalog=await response.json();if(catalog.single_component)document.documentElement.dataset.singleComponent='true';debug.catalog=catalog;$('#count').textContent=catalog.components.length;dimensions();view();pushSettings();route();}catch(error){fail('host',String(error));}
