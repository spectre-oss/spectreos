// Browser integration checks using Chrome DevTools Protocol and Node built-ins.
// Start a compatible Chromium with --remote-debugging-port=9223 first.
import fs from 'node:fs/promises';
import assert from 'node:assert/strict';
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
async function route(slug){await evaluate(`location.hash=${JSON.stringify('/'+slug)}`);await until(`window.bhooDemo?.component===${JSON.stringify(slug)}`);await delay(40);}
async function click(x,y){await call('Input.dispatchMouseEvent',{type:'mouseMoved',x,y});await call('Input.dispatchMouseEvent',{type:'mousePressed',button:'left',clickCount:1,x,y});await call('Input.dispatchMouseEvent',{type:'mouseReleased',button:'left',clickCount:1,x,y});await delay(70);}
async function point(x,y){const b=await evaluate(`(()=>{const b=document.querySelector('#preview').getBoundingClientRect();return{x:b.x,y:b.y}})()`);return [b.x+x,b.y+y];}
async function press(key,code,windowsVirtualKeyCode){await call('Input.dispatchKeyEvent',{type:'keyDown',key,code,windowsVirtualKeyCode});await call('Input.dispatchKeyEvent',{type:'keyUp',key,code,windowsVirtualKeyCode});await delay(60);}
async function clickNode(id){await evaluate("document.querySelector('#preview').scrollIntoView({block:'center',behavior:'instant'})");await delay(40);const p=await evaluate(`(()=>{const n=window.bhooDemo.semantics.find(n=>n.id===${JSON.stringify(id)});if(!n)throw Error('Missing node');const c=document.querySelector('#preview').getBoundingClientRect();return[c.x+n.bounds[0]+n.bounds[2]/2,c.y+n.bounds[1]+n.bounds[3]/2]})()`);await click(...p);}
const results=[];
try{
    await call('Runtime.enable');await call('Page.enable');
    await call('Emulation.setDeviceMetricsOverride',{width:1440,height:1100,deviceScaleFactor:1,mobile:false});
    await call('Page.navigate',{url:'about:blank'});
    await until("location.href==='about:blank' && !window.bhooDemo");
    await call('Page.navigate',{url:base+'/playground.html#/button'});
    await until('window.bhooDemo?.ready && window.bhooDemo?.frames>0');
    await call('Emulation.setDeviceMetricsOverride',{width:1440,height:1100,deviceScaleFactor:1,mobile:false});await delay(80);
    assert.equal(await evaluate("document.querySelectorAll('#navigation .nav-link').length"),await evaluate("window.bhooDemo.catalog.components.length + 1"));
    assert.equal(await evaluate('window.bhooDemo.errors.length'),0);results.push('All component pages + overview; real WASM started');
    await click(...await point(80,42));
    await until("window.bhooDemo.text.includes('Activations: 1')");results.push('Rust button click changes actual rendered state');
    await evaluate("document.querySelector('#disabled').click()");await delay(50);
    await click(...await point(80,42));
    assert(await evaluate("window.bhooDemo.text.includes('Activations: 1')"));
    assert(await evaluate('window.bhooDemo.semantics.every(n=>!n.enabled)'));results.push('Disabled state blocks activation');
    await evaluate("document.querySelector('#disabled').click()");
    await route('checkbox');await click(...await point(70,40));
    assert.equal(await evaluate("window.bhooDemo.semantics.find(n=>n.id==='1').checked"),'checked');results.push('Checkbox uses Rust checked state');
    await route('switch');await click(...await point(180,350));await press('Tab','Tab',9);await press(' ','Space',32);
    assert.equal(await evaluate("window.bhooDemo.semantics.find(n=>n.id==='1').checked"),'unchecked');results.push('Keyboard Tab/Space reaches and toggles Rust switch');
    await evaluate("document.querySelector('#reduced').checked || document.querySelector('#reduced').click()");await delay(80);
    assert.equal(await evaluate('window.bhooDemo.animating'),false);results.push('Reduced motion settles active effects');
    await route('virtualized-list');const [x,y]=await point(80,140);
    await call('Input.dispatchMouseEvent',{type:'mouseWheel',x,y,deltaY:600,deltaX:0});
    await until("window.bhooDemo.text.some(t=>t.includes('offset: 600px'))");
    assert((await evaluate('window.bhooDemo.semantics.length'))<20);results.push('100,000-row list scrolls and virtualizes');
    await route('text-field');
    assert.equal(await evaluate("document.querySelector('#demo-card').hidden"),false);
    await evaluate("document.querySelector('.native-adapter').focus()");
    await call('Input.insertText',{text:'Bhoo'});
    await until("window.bhooDemo.text.includes('4 characters')");
    results.push('Editable field commits browser input into Rust state');
    await route('dialog');await click(...await point(80,42));
    await until("window.bhooDemo.semantics.some(n=>n.id==='600'&&n.enabled)");
    await press('Escape','Escape',27);
    await until("!window.bhooDemo.semantics.some(n=>n.id==='600')");
    await until("window.bhooDemo.semantics.some(n=>n.id==='100'&&n.focused)");
    results.push('Modal Escape dismisses and restores trigger focus');
    await route('menu');await click(...await point(80,42));
    await until("window.bhooDemo.semantics.some(n=>n.id==='600')");
    await press('ArrowDown','ArrowDown',40);await press('Enter','Enter',13);
    await until("window.bhooDemo.text.includes('Selected: Rename')");
    results.push('Menu arrow navigation and Enter select a real item');
    const families=await evaluate('window.bhooDemo.catalog.components.map(c=>({slug:c.slug,scene:c.scene}))');
    assert(families.length >= 83);assert(families.every(c=>Number.isInteger(c.scene)));
    for(const item of families){
      await route(item.slug);
      assert.equal(await evaluate('window.bhooDemo.errors.length'),0,item.slug);
      assert((await evaluate('window.bhooDemo.commands'))>1,item.slug+' has drawing commands');
      assert.equal(await evaluate("document.querySelector('#demo-card').hidden"),false,item.slug);
      await evaluate("document.querySelector('#disabled').click()");await delay(30);
      assert(await evaluate('window.bhooDemo.semantics.every(n=>!n.enabled)'),item.slug+' disables controls');
      await evaluate("document.querySelector('#disabled').click()");
    }
    results.push('All pages render actual Rust scenes and honor disabled state');
    await route('stepper');
    assert(await evaluate("window.bhooDemo.semantics.some(n=>n.id==='101'&&!n.enabled)"));
    await evaluate("document.querySelector('.native-adapter').focus()");await call('Input.insertText',{text:'Spectre'});await delay(60);
    await clickNode('111');await until("window.bhooDemo.text.includes('Choose your profile')");
    results.push('Linear stepper validates before unlocking the next stage');
    await route('data-table');await clickNode('101');
    assert(await evaluate("(()=>{const t=window.bhooDemo.text;return t.indexOf('SpectreOS')<t.indexOf('Launcher')})()"));
    results.push('Size column sorts actual row data');
    await route('file-chooser-integration');
    const documentNode=await call('DOM.getDocument');const fileNode=await call('DOM.querySelector',{nodeId:documentNode.root.nodeId,selector:'.native-adapter[type=file]'});
    await call('DOM.setFileInputFiles',{nodeId:fileNode.nodeId,files:[process.cwd()+'/README.md']});
    await until("window.bhooDemo.text.includes('README.md')");results.push('Real file adapter commits filename without reading contents');
    await route('expansion-panel-accordion');
    await evaluate("document.querySelector('#reduced').checked && document.querySelector('#reduced').click()");
    await clickNode('100');assert(await evaluate('window.bhooDemo.animating'));
    await clickNode('100');await delay(1000);assert.equal(await evaluate('window.bhooDemo.animating'),false);
    await clickNode('100');await evaluate("document.querySelector('#reduced').click()");await delay(70);assert.equal(await evaluate('window.bhooDemo.animating'),false);
    results.push('Spatial disclosure reverses, settles, and snaps when reduced motion is enabled');
    await route('state-layer-ripple');
    await evaluate("document.querySelector('#reduced').checked && document.querySelector('#reduced').click()");
    const inkPoint=await evaluate("(()=>{const n=window.bhooDemo.semantics.find(n=>n.id==='100'),c=document.querySelector('#preview').getBoundingClientRect();return[c.x+n.bounds[0]+15,c.y+n.bounds[1]+18]})()");
    await call('Input.dispatchMouseEvent',{type:'mouseMoved',x:inkPoint[0],y:inkPoint[1]});
    await call('Input.dispatchMouseEvent',{type:'mousePressed',button:'left',clickCount:1,x:inkPoint[0],y:inkPoint[1]});
    await until('window.bhooDemo.ripples.length===1');
    const earlyInk=await evaluate('window.bhooDemo.ripples[0]');
    assert(earlyInk.center[0]<earlyInk.bounds[0]+earlyInk.bounds[2]/2,'Ink should begin near pointer');
    await fs.writeFile('context/bhoo-web-ripple.png',Buffer.from((await call('Page.captureScreenshot',{format:'png'})).data,'base64'));
    await delay(700);const heldInk=await evaluate('window.bhooDemo.ripples[0]');assert(heldInk.radius>earlyInk.radius);assert.equal(await evaluate('window.bhooDemo.animating'),false);
    await call('Input.dispatchMouseEvent',{type:'mouseReleased',button:'left',clickCount:1,x:inkPoint[0],y:inkPoint[1]});
    await until('window.bhooDemo.ripples.length===0 && !window.bhooDemo.animating');
    results.push('Pointer-origin radial ink grows, holds without polling, fades and returns to idle');
    await route('state-layer-ripple');
    await call('Emulation.setTouchEmulationEnabled',{enabled:true,maxTouchPoints:1});
    const touchPoint=await evaluate("(()=>{const n=window.bhooDemo.semantics.find(n=>n.id==='100'),c=document.querySelector('#preview').getBoundingClientRect();return[c.x+n.bounds[0]+20,c.y+n.bounds[1]+18]})()");
    await call('Input.dispatchTouchEvent',{type:'touchStart',touchPoints:[{x:touchPoint[0],y:touchPoint[1],id:1}]});await delay(30);
    assert.equal(await evaluate('window.bhooDemo.ripples.length'),0);
    await call('Input.dispatchTouchEvent',{type:'touchEnd',touchPoints:[]});
    await until('window.bhooDemo.ripples.length===1');await until('window.bhooDemo.ripples.length===0 && !window.bhooDemo.animating');
    await call('Emulation.setTouchEmulationEnabled',{enabled:false});results.push('Touch press waits; a quick tap still produces the minimum ink feedback');
    await route('expressive-loading-indicator');await clickNode('100');await until('window.bhooDemo.paths.length===3');
    const oldMorph=await evaluate('JSON.stringify(window.bhooDemo.paths[0].firstCurve)');await delay(150);
    assert.notEqual(await evaluate('JSON.stringify(window.bhooDemo.paths[0].firstCurve)'),oldMorph);
    await fs.writeFile('context/bhoo-web-loading.png',Buffer.from((await call('Page.captureScreenshot',{format:'png'})).data,'base64'));
    await evaluate("document.querySelector('#reduced').click()");await delay(100);
    const staticMorph=await evaluate('JSON.stringify(window.bhooDemo.paths)');const staticFrames=await evaluate('window.bhooDemo.frames');await delay(180);
    assert.equal(await evaluate('JSON.stringify(window.bhooDemo.paths)'),staticMorph);assert.equal(await evaluate('window.bhooDemo.frames'),staticFrames);
    results.push('Matched seven-shape loading paths animate, then freeze and idle under reduced motion');
    await route('switch');await evaluate("document.querySelector('#motion-scheme').value='expressive';document.querySelector('#motion-scheme').dispatchEvent(new Event('change'))");
    await clickNode('1');await delay(80);assert.equal(await evaluate('window.bhooDemo.animating'),false);
    results.push('Expressive scheme respects reduced motion');
    await route('tooltip');await evaluate("document.querySelector('#preview').focus()");
    const tooltipPoint=await evaluate("(()=>{const n=window.bhooDemo.semantics.find(n=>n.id==='1'),c=document.querySelector('#preview').getBoundingClientRect();return[c.x+n.bounds[0]+30,c.y+n.bounds[1]+18]})()");
    await call('Input.dispatchMouseEvent',{type:'mouseMoved',x:tooltipPoint[0],y:tooltipPoint[1]});
    await until("window.bhooDemo.semantics.some(n=>n.role==='tooltip')");await press('Escape','Escape',27);
    await until("!window.bhooDemo.semantics.some(n=>n.role==='tooltip')");await delay(650);
    assert.equal(await evaluate("window.bhooDemo.semantics.some(n=>n.role==='tooltip')"),false);
    assert.equal(await evaluate("document.activeElement===document.querySelector('#preview')"),true);
    results.push('Escape dismisses a tooltip without reopening it or leaving the preview');
    await route('virtualized-grid');const gridPoint=await point(80,140);
    await call('Input.dispatchMouseEvent',{type:'mouseWheel',x:gridPoint[0],y:gridPoint[1],deltaY:600,deltaX:0});
    await until("window.bhooDemo.text.some(t=>t.includes('Row offset: 600px'))");
    assert((await evaluate('window.bhooDemo.semantics.length'))<30);results.push('100,000-cell grid stays virtualized');
    await evaluate("document.querySelector('#tab-states').click()");assert.equal(await evaluate("document.querySelector('#panel-states').hidden"),false);
    await evaluate("document.querySelector('#tab-guidelines').click()");assert((await evaluate("document.querySelectorAll('#reference-list a').length"))>0);results.push('State/motion specs and reference links available');
    await evaluate("document.querySelector('#search').value='switch';document.querySelector('#search').dispatchEvent(new Event('input'))");
    assert.equal(await evaluate("document.querySelectorAll('#navigation .nav-link').length"),2);results.push('Catalog search filters navigation');
    await evaluate("document.querySelector('#search').value='';document.querySelector('#search').dispatchEvent(new Event('input'))");
    await route('button');
    await evaluate("document.documentElement.dataset.theme==='dark' && document.querySelector('#theme').click()");await delay(500);
    const before=await evaluate('window.bhooDemo.frames');await delay(250);assert.equal(await evaluate('window.bhooDemo.frames'),before);results.push('Idle preview stops scheduling frames');
    await fs.writeFile('context/bhoo-web-desktop.png',Buffer.from((await call('Page.captureScreenshot',{format:'png'})).data,'base64'));
    await call('Emulation.setDeviceMetricsOverride',{width:390,height:844,deviceScaleFactor:1,mobile:true});await delay(200);
    for(const item of families){await route(item.slug);assert(await evaluate('document.documentElement.scrollWidth<=innerWidth'),item.slug+' mobile overflow');assert.equal(await evaluate('window.bhooDemo.errors.length'),0,item.slug);}
    results.push('All previews render at 390px without horizontal overflow');
    await route('date-picker');await delay(80);
    await fs.writeFile('context/bhoo-web-calendar-mobile.png',Buffer.from((await call('Page.captureScreenshot',{format:'png',captureBeyondViewport:true})).data,'base64'));
    await route('button');

    await evaluate("document.querySelector('#menu-toggle').click()");assert(await evaluate("document.body.classList.contains('sidebar-open')"));
    await evaluate("document.querySelector('#mobile-shade').click()");assert(!(await evaluate("document.body.classList.contains('sidebar-open')")));results.push('Mobile navigation opens and dismisses');
    await fs.writeFile('context/bhoo-web-mobile.png',Buffer.from((await call('Page.captureScreenshot',{format:'png'})).data,'base64'));
    assert.equal(exceptions.length,0);assert.equal(await evaluate('window.bhooDemo.errors.length'),0);
    console.log(results.map((result,i)=>`PASS ${i+1}: ${result}`).join('\n'));
    console.log('Screenshots: context/bhoo-web-desktop.png, context/bhoo-web-mobile.png');
}finally{ws.close();}
