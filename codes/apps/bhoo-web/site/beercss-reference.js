import ui from './beercss/beer.min.js';
import {names} from './beercss-shapes.js';
const params=new URLSearchParams(location.search),run=Number(params.get('run'));
const send=(type,data={})=>parent.postMessage({source:'bhoo-angular',type,run,...data},location.origin==='null'?'*':location.origin);
const shapes=params.get('fixture')==='shapes';
const rotation=params.get('fixture')==='rotation';
const media=params.get('fixture')==='media';
const wave=params.get('fixture')==='wave';
if(shapes){
  document.body.classList.add('shape-fixture');
  const fixture=document.querySelector('.fixture');fixture.replaceChildren();
  const heading=document.createElement('p');heading.textContent='Expressive shapes';fixture.append(heading);
  const grid=document.createElement('div');grid.className='shape-grid';fixture.append(grid);
  for(const name of names){const item=document.createElement('div');item.className='shape-item';const icon=document.createElement('div');icon.className='shape';icon.style.maskImage=`url("beercss/${name}.svg")`;const label=document.createElement('span');label.textContent=name;item.append(icon,label);grid.append(item);}
}
if(rotation){
  document.body.classList.add('shape-fixture');const fixture=document.querySelector('.fixture');fixture.replaceChildren();const heading=document.createElement('p');heading.textContent='Rotating shapes';fixture.append(heading);const grid=document.createElement('div');grid.className='rotation-grid';fixture.append(grid);
  for(const [speed,label] of [['slow-rotate','Slow · 24s'],['rotate','Normal · 12s'],['fast-rotate','Fast · 6s']]){const item=document.createElement('div');item.className='rotation-item';const icon=document.createElement('div');icon.className='shape arrow '+speed;const caption=document.createElement('span');caption.textContent=label;item.append(icon,caption);grid.append(item);}
  const detail=document.createElement('p');detail.textContent='Rotation pauses when hidden or reduced motion is enabled.';fixture.append(detail);
}
if(media){
  document.body.classList.add('shape-fixture');const fixture=document.querySelector('.fixture');fixture.replaceChildren();const heading=document.createElement('p');heading.textContent='Image media';fixture.append(heading);const grid=document.createElement('div');grid.className='media-grid';fixture.append(grid);
  for(const [style,label] of [['circle','Circle'],['round','Rounded']]){const item=document.createElement('div');item.className='media-item';const image=document.createElement('img');image.className=style;image.src='beercss/demo-media.png';image.alt='';const caption=document.createElement('span');caption.textContent=label;item.append(image,caption);grid.append(item);}
  const image=document.createElement('img');image.className='responsive round media-wide';image.src='beercss/demo-media.png';image.alt='';fixture.append(image);const note=document.createElement('p');note.className='media-note';note.textContent='Caller-supplied RGBA image · cover fit';fixture.append(note);
}
if(wave){
  document.body.classList.add('shape-fixture','wave-fixture');const fixture=document.querySelector('.fixture');fixture.replaceChildren();const heading=document.createElement('p');heading.textContent='Hover or focus the button.';fixture.append(heading);const button=document.createElement('button');button.className='wave';button.textContent='Hover wave';fixture.append(button);const count=document.createElement('p');count.className='wave-count';count.textContent='Activations: 0';fixture.append(count);let n=0;button.addEventListener('click',()=>{count.textContent='Activations: '+ ++n;});
}
window.angularReference={ready:true,exists:true,errors:[],version:'5.0.3',template:shapes?'<div class="shape" style="mask-image: url(shape.svg)"></div>':rotation?'<div class="shape arrow slow-rotate"></div>\n<div class="shape arrow rotate"></div>\n<div class="shape arrow fast-rotate"></div>':media?'<img class="circle" src="demo-media.png">\n<img class="round" src="demo-media.png">\n<img class="responsive round" src="demo-media.png">':wave?'<button class="wave">Hover wave</button>':'<div class="page active left">Page one</div>\n<div class="page right">Page two</div>\nui("#two");'};
for(const button of document.querySelectorAll('[data-target]'))button.addEventListener('click',()=>ui('#'+button.dataset.target));
window.addEventListener('message',event=>{const d=event.data;if(event.source!==parent||d?.source!=='bhoo-comparison'||d.run!==run)return;if(d.type==='settings'){document.body.classList.toggle('dark',!!d.settings.dark);document.documentElement.dataset.reduced=String(!!d.settings.reduced);for(const button of document.querySelectorAll('button'))button.disabled=!!d.settings.disabled;}});
window.addEventListener('error',e=>send('error',{message:e.message}));
send('ready',{version:'5.0.3'});
