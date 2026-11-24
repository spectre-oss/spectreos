// Translate non-focusable Rust status semantics into a polite live region.
export function createLiveStatus() {
  const region=document.createElement('div');
  region.setAttribute('role','status');region.setAttribute('aria-live','polite');region.setAttribute('aria-atomic','true');
  Object.assign(region.style,{position:'absolute',width:'1px',height:'1px',padding:'0',margin:'-1px',overflow:'hidden',clip:'rect(0, 0, 0, 0)',whiteSpace:'nowrap',border:'0',visibility:'visible'});
  document.body.append(region);
  let previous='', generation=0;
  return nodes=>{
    const status=nodes.find(n=>n.role==='status');
    const key=status?`${status.context??''}:${status.id}:${status.label}`:'';
    if(key===previous)return;
    previous=key;const current=++generation;
    region.textContent='';
    if(status)setTimeout(()=>{if(current===generation)region.textContent=status.label;},30);
  };
}
