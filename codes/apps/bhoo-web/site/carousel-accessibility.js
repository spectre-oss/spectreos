// Mirror visible Bhoo carousel items into accessible DOM controls. Rust remains
// responsible for navigation, activation, layout and selection.
export function createCarouselAccessibility(canvas, callbacks) {
 const regions=new Map();
 const hidden={position:'absolute',width:'1px',height:'1px',padding:'0',margin:'-1px',overflow:'hidden',clip:'rect(0, 0, 0, 0)',whiteSpace:'nowrap',border:'0',visibility:'visible'};
 const keyFor=n=>`${n.context??''}:${n.id}`;
 function remove(region){const focused=region.element.contains(document.activeElement);region.element.remove();if(focused)canvas.focus({preventScroll:true});}
 return nodes=>{
  const modal=nodes.some(n=>n.role==='dialog'&&n.enabled);
  const groups=modal?[]:nodes.filter(n=>n.role==='carousel'&&n.enabled);
  const wanted=new Set(groups.map(keyFor));
  for(const [key,region] of regions)if(!wanted.has(key)){remove(region);regions.delete(key);}
  for(const group of groups){
   const key=keyFor(group);let region=regions.get(key);
   if(!region){const element=document.createElement('section');element.setAttribute('role','region');element.setAttribute('aria-roledescription','carousel');element.setAttribute('aria-label',group.label);Object.assign(element.style,hidden);document.body.append(element);region={element,items:new Map()};regions.set(key,region);}
   region.element.setAttribute('aria-label',group.label);
   const min=BigInt(group.id)+1n,max=BigInt(group.id)+BigInt(Math.trunc(group.max??0));
   const items=nodes.filter(n=>n.role==='carousel item'&&n.context===group.context&&/^\d+$/.test(n.id)&&BigInt(n.id)>=min&&BigInt(n.id)<=max);
   const visible=new Set(items.map(n=>n.id));
   const focusedNode=items.find(n=>n.focused&&n.enabled);
   for(const n of items){let entry=region.items.get(n.id);
    if(!entry){const slide=document.createElement('div');slide.setAttribute('role','group');slide.setAttribute('aria-roledescription','slide');const button=document.createElement('button');button.type='button';button.dataset.bhooCarouselItem=n.id;slide.append(button);const next=Array.from(region.element.children).find(child=>BigInt(child.firstElementChild.dataset.bhooCarouselItem)>BigInt(n.id));region.element.insertBefore(slide,next??null);entry={slide,button};region.items.set(n.id,entry);
     button.addEventListener('focus',()=>callbacks.focus(n.id,n.context));
     button.addEventListener('click',()=>callbacks.activate(n.id,n.context));
     button.addEventListener('keydown',event=>{
      if(event.key==='Tab'&&callbacks.tab){event.stopPropagation();event.preventDefault();callbacks.tab(n.id,n.context,event.shiftKey);}
      else if(['ArrowLeft','ArrowRight','Home','End'].includes(event.key)){event.stopPropagation();event.preventDefault();callbacks.key(n.id,n.context,event.key);}
     });
    }
    entry.slide.setAttribute('aria-label',`${Math.trunc(n.value)} of ${Math.trunc(n.max)}`);entry.button.textContent=n.label;entry.button.disabled=!n.enabled;
   }
   const wasInside=region.element.contains(document.activeElement);
   const transfer=document.activeElement===canvas&&callbacks.shouldTransfer?.();
   if((wasInside||transfer)&&focusedNode){const target=region.items.get(focusedNode.id)?.button;if(target&&document.activeElement!==target){target.focus({preventScroll:true});if(transfer)callbacks.transferred?.();}}
   for(const [id,entry] of region.items)if(!visible.has(id)&&entry.button!==document.activeElement){entry.slide.remove();region.items.delete(id);}
  }
 };
}
