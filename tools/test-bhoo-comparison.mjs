// Browser integration checks using Chrome DevTools Protocol and Node built-ins.
// Start a compatible Chromium with --remote-debugging-port=9223 first.
import fs from 'node:fs/promises';
import assert from 'node:assert/strict';
import {comparePng,encodePng,decodePng,cropPng} from './bhoo-png-diff.mjs';
const base=process.env.BHOO_WEB_URL||'http://127.0.0.1:8766';
const port=process.env.BHOO_CDP_PORT||'9223';
const tabs=await fetch(`http://127.0.0.1:${port}/json/list`).then(r=>r.json());
const tab=tabs.find(t=>t.type==='page');assert(tab,'No browser page available');
const ws=new WebSocket(tab.webSocketDebuggerUrl);
await new Promise((resolve,reject)=>{ws.addEventListener('open',resolve,{once:true});ws.addEventListener('error',reject,{once:true});});
let serial=0;const pending=new Map(),exceptions=[];
ws.addEventListener('message',event=>{
    const message=JSON.parse(event.data);
    if(message.id){const request=pending.get(message.id);pending.delete(message.id);if(message.error)request.reject(Error(JSON.stringify(message.error)));else request.resolve(message.result);}
    else if(message.method==='Runtime.exceptionThrown')exceptions.push(message.params.exceptionDetails.text);
});
const call=(method,params={})=>new Promise((resolve,reject)=>{const id=++serial;pending.set(id,{resolve,reject});ws.send(JSON.stringify({id,method,params}));});
const evaluate=async expression=>{
    const result=await call('Runtime.evaluate',{expression,returnByValue:true,awaitPromise:true});
    if(result.exceptionDetails)throw Error(JSON.stringify(result.exceptionDetails));
    return result.result.value;
};
const delay=ms=>new Promise(resolve=>setTimeout(resolve,ms));
async function until(expression){for(let i=0;i<160;i++){if(await evaluate(expression))return;await delay(25);}throw Error('Timeout: '+expression);}
const artifact='context/bhoo-comparison';
let viewport={width:1600,height:1200,deviceScaleFactor:1,mobile:false};
async function configureViewport(next=viewport){viewport=next;await call('Emulation.setDeviceMetricsOverride',viewport);await delay(80);}
const results=[],comparisons=[],network=[];
const surfacePixel=image=>image.rgba.subarray((4*image.width+4)*4,(4*image.width+4)*4+4);
const aw="document.querySelector('#angular').contentWindow";
const bw="document.querySelector('#bhoo').contentWindow";
const ad="document.querySelector('#angular').contentDocument";
async function route(slug){await evaluate(`location.hash=${JSON.stringify('/'+slug)}`);await until(`window.bhooComparison?.ready&&window.bhooComparison.component===${JSON.stringify(slug)}`);await configureViewport();}
async function set(id,value){await evaluate(`(()=>{const e=document.querySelector(${JSON.stringify('#'+id)});${typeof value==='boolean'?`if(e.checked!==${value})e.click();`:`e.value=${JSON.stringify(value)};e.dispatchEvent(new Event('change'));`}})()`);await delay(100);}
async function reset(){const previous=await evaluate('window.bhooComparison.run');await evaluate("document.querySelector('#reset').click()");await until(`window.bhooComparison.ready&&window.bhooComparison.run>${previous}`);await configureViewport();}
async function mouse(type,x,y){await call('Input.dispatchMouseEvent',{type,x,y,...(type==='mouseMoved'?{}:{button:'left',clickCount:1})});}
async function click(x,y){await mouse('mouseMoved',x,y);await mouse('mousePressed',x,y);await mouse('mouseReleased',x,y);await delay(90);}
async function point(name,selector){return evaluate(`(()=>{const frame=document.querySelector('#${name}');frame.scrollIntoView({block:'center',behavior:'instant'});const r=frame.getBoundingClientRect();${name==='angular'?`const b=frame.contentDocument.querySelector(${JSON.stringify(selector)}).getBoundingClientRect();return[r.x+b.x+b.width/2,r.y+b.y+b.height/2];`:`const b=frame.contentWindow.bhooDemo.semantics.find(n=>n.id===${JSON.stringify(selector)}).bounds;return[r.x+b[0]+b[2]/2,r.y+b[1]+b[3]/2];`}})()`);}
async function hit(name,selector){const p=await point(name,selector);await delay(30);await click(...p);}
async function press(key,code,windowsVirtualKeyCode,modifiers=0){await call('Input.dispatchKeyEvent',{type:'keyDown',key,code,windowsVirtualKeyCode,modifiers});await call('Input.dispatchKeyEvent',{type:'keyUp',key,code,windowsVirtualKeyCode,modifiers});await delay(70);}
async function capture(name){
  await evaluate(`document.querySelector('#${name}').scrollIntoView({block:'center',inline:'center',behavior:'instant'})`);await delay(50);
  const rect=await evaluate(`(()=>{const b=document.querySelector('#${name}').getBoundingClientRect();return{x:b.x,y:b.y,width:b.width,height:b.height}})()`);
  // Cropping a normal viewport screenshot avoids Chrome's beyond-viewport scrollbar/layout changes.
  const result=await call('Page.captureScreenshot',{format:'png'});
  return cropPng(Buffer.from(result.data,'base64'),rect.x,rect.y,rect.width,rect.height);
}
async function record(name,left=null,right=null){
  left??=await capture('angular');right??=await capture('bhoo');
  const {png,...metrics}=comparePng(left,right);
  await fs.writeFile(`${artifact}/${name}-angular.png`,left);await fs.writeFile(`${artifact}/${name}-bhoo.png`,right);await fs.writeFile(`${artifact}/${name}-diff.png`,png);
  const ref=await evaluate('window.bhooComparison.reference');
  comparisons.push({name,...metrics,referenceKind:ref.kind,sameContent:ref.sameContent,verdict:'Not certified; whole-pane difference includes layout, text, anti-aliasing and fixture differences.'});
}
ws.addEventListener('message',event=>{const m=JSON.parse(event.data);if(m.method==='Network.requestWillBeSent'&&/^https?:/.test(m.params.request.url))network.push(m.params.request.url);});
try{
  await fs.mkdir(artifact,{recursive:true});
  const sample=Buffer.from([255,0,0,255,0,255,0,255]);const samplePng=encodePng(2,1,sample);assert.deepEqual(decodePng(samplePng).rgba,sample);assert.equal(comparePng(samplePng,samplePng).changedPixels,0);results.push('PNG decoder/encoder round trip and exact-identity comparison');
  await call('Runtime.enable');await call('Page.enable');await call('Network.enable');
  await call('Page.navigate',{url:'about:blank'});await until("location.href==='about:blank'&&!window.bhooComparison");
  await call('Page.navigate',{url:base+'/#/button'});await until('window.bhooComparison?.ready');
  await configureViewport({width:1600,height:1200,deviceScaleFactor:1,mobile:false});await delay(100);
  await evaluate("window.bhooComparison.settings.dark && document.querySelector('#theme').click()");await delay(150);
  assert.equal(await evaluate("document.querySelectorAll('#navigation .nav-link').length"),await evaluate("window.bhooComparison.catalog.components.length"));
  assert.equal(await evaluate(`${ad}.querySelector('angular-reference').getAttribute('ng-version')`),'22.2.5');
  assert.equal(await evaluate(`${aw}.angularReference.version`),'22.2.6');
  assert(await evaluate(`${bw}.bhooDemo.wasmBytes>200000`));
  results.push('Genuine pinned Angular Material and Rust/WASM, catalog navigation entries');
  await set('reduced',true);await record('button-default');
  assert.equal(await evaluate(`${bw}.document.querySelector('#preview').clientWidth`),440);
  assert.equal(await evaluate(`${ad}.documentElement.clientWidth`),440);
  const a0=decodePng(await capture('angular')),b0=decodePng(await capture('bhoo'));
  assert.deepEqual(surfacePixel(a0),surfacePixel(b0));results.push('Equal 440×640 viewports and actual shared Material surface pixels');
  await hit('angular','button');await hit('bhoo','10');
  await until(`${ad}.body.textContent.includes('Activations: 1')&&${bw}.bhooDemo.text.includes('Activations: 1')`);
  results.push('Trusted pointer clicks activate Angular and Rust independently');
  await set('disabled',true);
  assert(await evaluate(`[...${ad}.querySelectorAll('button')].every(b=>b.disabled)`));
  assert(await evaluate(`${bw}.bhooDemo.semantics.every(n=>!n.enabled)`));
  await hit('angular','button');await hit('bhoo','10');
  assert(await evaluate(`${ad}.body.textContent.includes('Activations: 1')&&${bw}.bhooDemo.text.includes('Activations: 1')`));
  await record('button-disabled');await set('disabled',false);results.push('Shared disabled state changes both renderers and blocks both activations');
  await reset();await set('reduced',false);
  let p=await point('angular','button');await mouse('mouseMoved',...p);await delay(250);const hoveredA=await capture('angular');
  p=await point('bhoo','10');await mouse('mouseMoved',...p);await delay(250);const hoveredB=await capture('bhoo');await record('button-hover',hoveredA,hoveredB);
  p=await point('angular','button');await mouse('mousePressed',...p);await delay(100);const pressedA=await capture('angular');await mouse('mouseReleased',...p);
  p=await point('bhoo','10');await mouse('mousePressed',...p);await delay(100);assert(await evaluate(`${bw}.bhooDemo.ripples.length>0`));const pressedB=await capture('bhoo');await mouse('mouseReleased',...p);await record('button-pressed',pressedA,pressedB);results.push('Separate hover and held-press captures, including actual Rust ripple');
  await set('reduced',true);await reset();
  await evaluate(`${ad}.querySelector('button').focus()`);await press('Tab','Tab',9);await press('Tab','Tab',9,8);const focusedA=await capture('angular');
  await evaluate(`${bw}.document.querySelector('#preview').focus()`);await press('Tab','Tab',9);assert(await evaluate(`${bw}.bhooDemo.semantics.some(n=>n.focused)`));const focusedB=await capture('bhoo');await record('button-focus',focusedA,focusedB);results.push('Keyboard focus can be inspected in both implementations');
  for(const variant of ['2','3','4','5']){await set('variant',variant);assert.equal(await evaluate(`${aw}.angularReference.settings.variant`),Number(variant));assert.equal(await evaluate(`${bw}.bhooDemo.comparisonSettings.variant`),Number(variant));}await set('variant','1');results.push('Button variant settings reach both implementations');
  await evaluate("document.querySelector('#theme').click()");await delay(150);
  assert.equal(await evaluate(`${ad}.documentElement.dataset.theme`),'dark');assert.equal(await evaluate(`${bw}.document.documentElement.dataset.theme`),'dark');
  const da=decodePng(await capture('angular')),db=decodePng(await capture('bhoo'));assert.deepEqual(surfacePixel(da),surfacePixel(db));assert.notDeepEqual(surfacePixel(da),surfacePixel(a0));await record('button-dark');
  await set('shared-theme',false);assert.equal(await evaluate(`${bw}.bhooDemo.comparisonSettings.sharedTheme`),false);const original=decodePng(await capture('bhoo'));assert.notDeepEqual(surfacePixel(original),surfacePixel(db));await set('shared-theme',true);await evaluate("document.querySelector('#theme').click()");await delay(100);results.push('Shared light/dark palette pixels and optional original Bhoo palette');
  await set('view','overlay');await evaluate("document.querySelector('#mix').value='35';document.querySelector('#mix').dispatchEvent(new Event('input'))");
  assert.equal(await evaluate("document.querySelector('#angular-viewport').style.opacity"),'0.35');assert(await evaluate("document.querySelector('#bhoo').inert"));
  assert(await evaluate("(()=>{const a=document.querySelector('#angular').getBoundingClientRect(),b=document.querySelector('#bhoo').getBoundingClientRect();return a.x===b.x&&a.y===b.y})()"));
  await set('view','wipe');assert.equal(await evaluate("document.querySelector('.angular-panel').style.clipPath"),'inset(0px 65% 0px 0px)');await set('view','side');
  for(const width of ['360','600','440']){await set('width',width);assert.equal(await evaluate(`${bw}.document.querySelector('#preview').clientWidth`),Number(width));assert.equal(await evaluate(`${ad}.documentElement.clientWidth`),Number(width));}results.push('Overlay registration, wipe clipping, inert inspection frames and shared widths');
  await route('checkbox');await set('link-checkbox',false);await hit('angular','mat-checkbox input');await hit('bhoo','1');await until(`${bw}.bhooDemo.semantics.find(n=>n.id==='1').checked==='checked'`);assert(await evaluate(`${ad}.querySelector('mat-checkbox input').checked`));await record('checkbox-checked');results.push('Checked state reached with actual input in both implementations');
  await route('switch');await hit('angular','mat-slide-toggle button');await hit('bhoo','1');await until(`${bw}.bhooDemo.semantics.find(n=>n.id==='1').checked==='unchecked'`);assert.equal(await evaluate(`${ad}.querySelector('mat-slide-toggle button').getAttribute('aria-checked')`),'false');await record('switch-off');results.push('Switch toggles to the same logical state');
  await route('menu');await hit('angular','button');await hit('bhoo','100');await until(`${ad}.querySelector('[role="menu"]')&&${bw}.bhooDemo.semantics.some(n=>n.id==='600')`);await record('menu-open');
  await evaluate(`${ad}.querySelector('[role="menuitem"]').focus()`);await press('Escape','Escape',27);await evaluate(`${bw}.document.querySelector('#preview').focus()`);await press('Escape','Escape',27);await until(`!${ad}.querySelector('[role="menu"]')&&!${bw}.bhooDemo.semantics.some(n=>n.id==='600')`);results.push('Real menus open, render and dismiss on Escape in both implementations');
  await route('dialog');await hit('angular','button');await hit('bhoo','100');await until(`${ad}.querySelector('mat-dialog-container')&&${bw}.bhooDemo.semantics.some(n=>n.id==='600')`);await record('dialog-open');
  await evaluate(`${ad}.querySelector('mat-dialog-container button').focus()`);await press('Escape','Escape',27);await evaluate(`${bw}.document.querySelector('#preview').focus()`);await press('Escape','Escape',27);await until(`!${ad}.querySelector('mat-dialog-container')&&!${bw}.bhooDemo.semantics.some(n=>n.id==='600')`);results.push('Real dialogs open and dismiss on Escape in both implementations');
  const families=await evaluate('window.bhooComparison.catalog.components.map(c=>c.slug)');const counts={direct:0,related:0,none:0,beer:0};
  for(const slug of families){
    await route(slug);const ref=await evaluate('window.bhooComparison.reference');counts[ref.kind]++;
    assert.equal(await evaluate(`${aw}.angularReference.errors.length`),0,slug+' Angular errors');assert.equal(await evaluate(`${bw}.bhooDemo.errors.length`),0,slug+' Rust errors');
    assert.equal(await evaluate(`${aw}.angularReference.exists`),ref.kind!=='none',slug+' honest equivalent');
    if(ref.kind==='direct')await record(slug+'-baseline');
    await set('disabled',true);assert(await evaluate(`${bw}.bhooDemo.semantics.every(n=>!n.enabled)`),slug+' Rust disabled');await set('disabled',false);
  }
  results.push(`All routes: ${counts.direct} direct Angular equivalents, ${counts.related} related patterns, ${counts.none} explicit missing references; zero runtime errors`);
  await route('expressive-loading-indicator');await set('reduced',false);await delay(100);await evaluate("document.querySelector('#tab-overview').click()");await delay(100);const before=await evaluate(`${bw}.bhooDemo.frames`);await delay(180);assert.equal(await evaluate(`${bw}.bhooDemo.frames`),before);await evaluate("document.querySelector('#tab-api').click()");assert(await evaluate("document.querySelector('#code').textContent.length>10"));await evaluate("document.querySelector('#tab-examples').click()");await set('reduced',true);results.push('Documentation tabs show source/references and suspend hidden Rust animation');
  await evaluate("document.querySelector('#search').value='switch';document.querySelector('#search').dispatchEvent(new Event('input'))");assert.equal(await evaluate("document.querySelectorAll('#navigation .nav-link').length"),1);await evaluate("document.querySelector('#search').value='';document.querySelector('#search').dispatchEvent(new Event('input'))");results.push('Component search filters the comparison catalog');
  await route('button');await configureViewport({width:390,height:844,deviceScaleFactor:1,mobile:true});await delay(150);
  for(const slug of families){await route(slug);assert(await evaluate('document.documentElement.scrollWidth<=innerWidth'),slug+' mobile page overflow');}
  await evaluate("document.querySelector('#menu').click()");assert(await evaluate("document.body.classList.contains('sidebar-open')"));await evaluate("document.querySelector('#shade').click()");assert.equal(await evaluate("document.body.classList.contains('sidebar-open')"),false);results.push('All mobile routes contain overflow within preview scroll areas; navigation opens/dismisses');
  await route('button');await fs.writeFile(`${artifact}/mobile.png`,Buffer.from((await call('Page.captureScreenshot',{format:'png',captureBeyondViewport:true})).data,'base64'));
  await configureViewport({width:1600,height:1200,deviceScaleFactor:1,mobile:false});await evaluate('window.scrollTo(0,0)');await delay(100);await fs.writeFile(`${artifact}/desktop.png`,Buffer.from((await call('Page.captureScreenshot',{format:'png'})).data,'base64'));
  assert.equal(await evaluate('window.bhooComparison.errors.length'),0);assert.equal(exceptions.length,0);assert(network.every(url=>new URL(url).origin===new URL(base).origin));results.push('No unhandled exceptions or external runtime requests');
  await fs.writeFile(`${artifact}/results.json`,JSON.stringify({results,counts,comparisons},null,2)+'\n');
  const rows=comparisons.map(item=>`<section><h2>${item.name}</h2><p>${item.referenceKind}; ${item.sameContent?'matching labels and starting values':'different fixture content/layout'}. ${(item.changedFraction*100).toFixed(2)}% of whole-pane pixels differ by more than ${item.threshold}/255 in an RGB channel. This is not a component match score.</p><div>${['angular','bhoo','diff'].map(name=>`<figure><figcaption>${name}</figcaption><img loading="lazy" src="${item.name}-${name}.png" width="440" alt="${item.name} ${name}"></figure>`).join('')}</div></section>`).join('');
  await fs.writeFile(`${artifact}/index.html`,`<!doctype html><meta charset="utf-8"><title>Bhoo / Angular visual evidence</title><style>body{font:14px/1.7 Arial;margin:32px;color:#26202b}section{border-top:1px solid #ddd;padding-top:16px;margin-top:32px}section>div{display:flex;gap:16px;overflow:auto}figure{margin:0}img{border:1px solid #ddd;max-width:440px}h1,h2{font-weight:500}p{max-width:1000px}figcaption{font-weight:bold;text-transform:capitalize}</style><h1>Bhoo / Angular visual evidence</h1><p>Angular Material 22.2.6 and actual Rust/WASM, equal 440×640 CSS px viewports. Baselines use shared Material colors, Arial as the requested font family, light theme and reduced motion. Chromium may use a system fallback font. Red marks differences. Images exercise the actual Rust text metrics, geometry and layout. Different fixtures and animation phase prevent these images from certifying parity.</p><p>Trusted pointer and keyboard interactions produce hover, pressed, focus, checked and open-state images. The capture is sequential; phase-aligned animation parity is not established. Tests passed: ${results.length} groups.</p>${rows}`);
  console.log(results.map((r,i)=>`PASS ${i+1}: ${r}`).join('\n'));console.log(`Visual evidence: ${artifact}/index.html (${comparisons.length} pairs + heatmaps)`);
}finally{ws.close();}
