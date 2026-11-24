// Exercise the unpacked standalone webxdc through Chrome DevTools Protocol.
import assert from 'node:assert/strict';
const base=process.env.BHOO_GALLERY_URL||'http://127.0.0.1:8766';
const port=process.env.BHOO_CDP_PORT||'9223';
const tabs=await fetch(`http://127.0.0.1:${port}/json/list`).then(r=>r.json());
const ws=new WebSocket(tabs.find(t=>t.type==='page').webSocketDebuggerUrl);
await new Promise((resolve,reject)=>{ws.addEventListener('open',resolve,{once:true});ws.addEventListener('error',reject,{once:true})});
let id=0;const pending=new Map(),requests=[],exceptions=[];
ws.addEventListener('message',e=>{const m=JSON.parse(e.data);if(m.id){const p=pending.get(m.id);pending.delete(m.id);m.error?p.reject(m.error):p.resolve(m.result)}else if(m.method==='Network.requestWillBeSent')requests.push(m.params.request.url);else if(m.method==='Runtime.exceptionThrown')exceptions.push(m.params.exceptionDetails.text)});
const call=(method,params={})=>new Promise((resolve,reject)=>{const n=++id;pending.set(n,{resolve,reject});ws.send(JSON.stringify({id:n,method,params}))});
const evaluate=async expression=>{const r=await call('Runtime.evaluate',{expression,returnByValue:true,awaitPromise:true});if(r.exceptionDetails)throw Error(JSON.stringify(r.exceptionDetails));return r.result.value};
const delay=ms=>new Promise(r=>setTimeout(r,ms));async function until(s){for(let i=0;i<300;i++){if(await evaluate(s))return;await delay(30)}throw Error('Timeout: '+s)}

const fs=await import('node:fs/promises');await fs.mkdir('context/bhoo-app-tests',{recursive:true});
async function click(label){const n=await evaluate(`bhooApp.nodes.find(n=>n.label===${JSON.stringify(label)})`);assert(n,'Missing '+label);const x=n.bounds[0]+n.bounds[2]/2,y=n.bounds[1]+n.bounds[3]/2;await call('Input.dispatchMouseEvent',{type:'mousePressed',x,y,button:'left',clickCount:1});await call('Input.dispatchMouseEvent',{type:'mouseReleased',x,y,button:'left',clickCount:1});await delay(180);}
async function type(label,text){await click(label);await until(`document.querySelector('input,textarea')?.getAttribute('aria-label')===${JSON.stringify(label)}`);await evaluate(`(()=>{const e=document.querySelector('input,textarea');e.value=${JSON.stringify(text)};e.dispatchEvent(new Event('input',{bubbles:true}));})()`);await delay(100);}
async function shot(name){await fs.writeFile('context/bhoo-app-tests/'+name+'.png',Buffer.from((await call('Page.captureScreenshot',{format:'png'})).data,'base64'));}
async function reload(){await call('Page.reload',{ignoreCache:true});await delay(100);await until('window.bhooApp?.ready');}
try{
 await call('Page.enable');await call('Runtime.enable');await call('Network.enable');
 await call('Emulation.setDeviceMetricsOverride',{width:390,height:844,deviceScaleFactor:2,mobile:true});
 for(const app of ['fossify','deltachat','thunderbird','sunflower']){
  await call('Page.navigate',{url:process.env.BHOO_PACKAGED?base+'/'+app+'/index.html':base+'/app-test.html?app='+app});await until('window.bhooApp?.ready');await evaluate(`localStorage.removeItem('bhoo-app-test-v1-'+bhooApp.app)`);await reload();
  await shot(app+'-list');
  if(app==='thunderbird'){
   await click('Open navigation');await until('bhooApp.nodes.some(n=>n.label==="Inbox" && n.enabled)');
   assert((await evaluate('bhooApp.text')).includes('you@example.test'),'Navigation drawer did not open');
   await call('Input.dispatchKeyEvent',{type:'keyDown',key:'Escape',code:'Escape'});await call('Input.dispatchKeyEvent',{type:'keyUp',key:'Escape',code:'Escape'});await delay(250);
  }
  if(app==='sunflower'){
   await click('Plant list');await until('bhooApp.nodes.some(n=>n.label==="Apple")');await click('Apple');await click('Add to garden');await until('bhooApp.nodes.some(n=>n.label==="Remove from garden")');await shot(app+'-detail');await click('Back');await click('My garden');assert(await evaluate('bhooApp.nodes.some(n=>n.label==="Apple")'));await reload();assert(await evaluate('bhooApp.nodes.some(n=>n.label==="Apple")'));await click('Apple');await click('Remove from garden');await click('Back');assert(!(await evaluate('bhooApp.nodes.some(n=>n.label==="Apple")')));await click('Plant list');
  }else if(app==='thunderbird'){
   await click('Alex Morgan');await click('Star');await shot(app+'-detail');await click('Back');await click('Starred');assert(await evaluate('bhooApp.nodes.some(n=>n.label==="Alex Morgan")'));await click('Alex Morgan');await click('Archive');await click('Archive');await click('Alex Morgan');await click('Restore');await click('Inbox');await click('Compose');await type('To','tester@example.test');await type('Subject','Integration test');await type('Message body','A local email from Rust.\nSecond line.');await shot(app+'-compose');await click('Send locally');await until('bhooApp.nodes.some(n=>n.label==="tester@example.test")');await reload();await click('tester@example.test');assert((await evaluate('bhooApp.text')).join(' ').includes('A local email from Rust.'));await click('Reply');await until('bhooApp.text.some(t=>t.includes("Re: Integration test"))');await click('Back');
  }else{
   await click('Alex Morgan');await type('Message','Hello '+app+' 👻');await click('Send locally');await until(`bhooApp.text.some(t=>t.includes(${JSON.stringify('Hello '+app+' 👻')}))`);await shot(app+'-conversation');await click('Back');await reload();await click('Alex Morgan');assert((await evaluate('bhooApp.text')).join(' ').includes('Hello '+app+' 👻'));await type('Message','Saved draft');await click('Back');await reload();await click('Alex Morgan');assert((await evaluate('bhooApp.text')).join(' ').includes('Saved draft'));await click('Back');await click('New conversation');await click('Mina Lee');await until('bhooApp.nodes.some(n=>n.label==="Send locally")');await click('Back');
  }
  await click('Search');await type('Search locally','zz-no-results');assert((await evaluate('bhooApp.text')).join(' ').match(/No conversations|garden is waiting/));await type('Search locally','');await click(app==='thunderbird'?'Close search':'Search');
  const before=await evaluate('bhooApp.nodes.filter(n=>n.role==="list item").map(n=>n.label).join()');
  await call('Input.dispatchTouchEvent',{type:'touchStart',touchPoints:[{x:190,y:650,id:1}]});await delay(30);await call('Input.dispatchTouchEvent',{type:'touchMove',touchPoints:[{x:190,y:280,id:1}]});await delay(30);await call('Input.dispatchTouchEvent',{type:'touchEnd',touchPoints:[]});await delay(350);assert(!(await evaluate('bhooApp.nodes.some(n=>n.label==="Back")')),'Touch scroll activated a row');
  if(app==='thunderbird'){await click('More options');await click('About demo');await until('bhooApp.nodes.some(n=>n.label==="Close"&&n.enabled)');}else{await click('Demo information');await until('bhooApp.text.includes("No account, network or real delivery.")');}await click('Close');await delay(250);
  assert.deepEqual(await evaluate('bhooApp.errors'),[]);console.log('PASS '+app+': routes, actions, editing/search, local persistence, touch scroll, dialog');
  await call('Emulation.setDeviceMetricsOverride',{width:1200,height:900,deviceScaleFactor:1,mobile:false});await delay(200);await shot(app+'-desktop');assert.deepEqual(await evaluate('bhooApp.errors'),[]);await call('Emulation.setDeviceMetricsOverride',{width:390,height:844,deviceScaleFactor:2,mobile:true});
 }
 assert.deepEqual(exceptions,[]);assert(requests.filter(x=>/^https?:/.test(x)).every(x=>x.startsWith(base+'/')),'External runtime network request');
 console.log('PASS: four apps, mobile/desktop and local-only runtime resources');
}finally{ws.close()}
