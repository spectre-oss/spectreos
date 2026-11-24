// Mirror the time picker's visible Rust listbox into the browser accessibility tree.
export function createTimePickerAccessibility(canvas, callbacks) {
 const lists=new Map();
 const hidden={position:'absolute',width:'1px',height:'1px',padding:'0',margin:'-1px',overflow:'hidden',clip:'rect(0, 0, 0, 0)',whiteSpace:'nowrap',border:'0'};
 const key=n=>`${n.context??''}:${n.id}`;
 return nodes=>{
  const groups=nodes.filter(n=>n.role==='list'&&n.id==='510'&&n.enabled&&callbacks.allow(n.context));
  const wanted=new Set(groups.map(key));
  for(const [k,entry] of lists)if(!wanted.has(k)){const focused=document.activeElement===entry.element;entry.element.remove();lists.delete(k);if(focused)canvas.focus({preventScroll:true});}
  for(const group of groups){
   const k=key(group);let entry=lists.get(k);
   if(!entry){const element=document.createElement('div');element.setAttribute('role','listbox');element.setAttribute('aria-label','Select time');element.tabIndex=0;Object.assign(element.style,hidden);document.body.append(element);entry={element,options:new Map()};lists.set(k,entry);
    element.addEventListener('focus',()=>callbacks.focus(group.id,group.context));
    element.addEventListener('keydown',event=>{if(['ArrowUp','ArrowDown','Home','End','Enter','Escape'].includes(event.key)){event.preventDefault();event.stopPropagation();callbacks.key(group.id,group.context,event.key);}else if(event.key==='Tab')event.stopPropagation();});
   }
   const options=nodes.filter(n=>n.role==='list item'&&n.context===group.context&&/^\d+$/.test(n.id)&&Number(n.id)>=600&&Number(n.id)<648);
   const visible=new Set(options.map(n=>n.id));
   for(const n of options){let option=entry.options.get(n.id);if(!option){option=document.createElement('div');option.id=`bhoo-time-${group.context??'preview'}-${n.id}`;option.setAttribute('role','option');option.addEventListener('click',()=>callbacks.activate(n.id,n.context));entry.element.append(option);entry.options.set(n.id,option);}option.textContent=n.label;option.setAttribute('aria-selected',String(n.selected));}
   for(const [id,option] of entry.options)if(!visible.has(id)){option.remove();entry.options.delete(id);}
   // Preserve content order when the virtual window moves backwards.
   for(const n of options){const option=entry.options.get(n.id);entry.element.append(option);}
   const selected=options.find(n=>n.selected);if(selected)entry.element.setAttribute('aria-activedescendant',entry.options.get(selected.id).id);else entry.element.removeAttribute('aria-activedescendant');
  }
 };
}
