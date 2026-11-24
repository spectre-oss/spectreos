// Bridge Bhoo dialog semantics to DOM controls using host-supplied callbacks.
export function createDialogAccessibility(canvas, callbacks=null) {
  const container=callbacks?.separate?document.createElement('div'):callbacks?canvas.parentElement:canvas;
  if(callbacks?.separate)document.body.append(container);
  const original={role:container.getAttribute('role'),label:container.getAttribute('aria-label'),description:container.getAttribute('aria-describedby')};
  const description=document.createElement('span');description.id='bhoo-dialog-description';
  const hidden={position:'absolute',width:'1px',height:'1px',padding:'0',margin:'-1px',overflow:'hidden',clip:'rect(0, 0, 0, 0)',whiteSpace:'nowrap',border:'0'};
  Object.assign(description.style,hidden);container.append(description);
  const controls=new Map();let previous='';
  const restore=(name,value)=>value===null?container.removeAttribute(name):container.setAttribute(name,value);
  const clear=()=>{const hadFocus=[...controls.values()].includes(document.activeElement);for(const button of controls.values())button.remove();controls.clear();if(hadFocus)canvas.focus({preventScroll:true});};
  return nodes=>{
    const dialog=[...nodes].reverse().find(n=>n.role==='dialog'&&n.enabled);
    const details=dialog&&nodes.find(n=>n.role==='dialog description'&&n.id===dialog.id&&n.context===dialog.context);
    const key=dialog?`${dialog.context??''}:${dialog.id}:${dialog.label}:${details?.label??''}`:'';
    if(key!==previous){if(previous&&key)clear();previous=key;
      if(dialog){container.setAttribute('role','dialog');container.setAttribute('aria-modal','true');container.setAttribute('aria-label',dialog.label);description.textContent=details?.label??'';if(details)container.setAttribute('aria-describedby',description.id);else container.removeAttribute('aria-describedby');}
      else {clear();restore('role',original.role);restore('aria-label',original.label);restore('aria-describedby',original.description);container.removeAttribute('aria-modal');description.textContent='';}
    }
    if(!dialog||!callbacks)return;
    // The modal overlay marks background controls disabled; only expose its
    // currently enabled buttons. Nested dialog ownership remains unsupported.
    const buttons=nodes.filter(n=>n.context===dialog.context&&n.enabled&&n.role==='button'&&/^\d+$/.test(n.id)&&Number(n.id)<=0xffffffff);
    const wanted=new Set(buttons.map(n=>n.id));
    for(const [id,button] of controls)if(!wanted.has(id)){button.remove();controls.delete(id);}
    for(const n of buttons){let button=controls.get(n.id);
      if(!button){button=document.createElement('button');button.type='button';button.dataset.bhooDialogControl=n.id;Object.assign(button.style,hidden);
        button.addEventListener('focus',()=>callbacks.focus(n.id,n.context));
        button.addEventListener('click',()=>callbacks.activate(n.id,n.context));
        button.addEventListener('keydown',event=>{
          if(event.key==='Escape'){event.stopPropagation();event.preventDefault();callbacks.escape(n.context);}
          else if(event.key==='Tab'){event.stopPropagation();const available=[...controls.values()];if(!available.length)return;event.preventDefault();const index=available.indexOf(button),next=(index+(event.shiftKey?-1:1)+available.length)%available.length;available[next].focus();}
          else if(event.key==='Enter'||event.key===' ')event.stopPropagation();
        });
        controls.set(n.id,button);container.append(button);
      }
      button.textContent=n.label;
    }
    if(document.activeElement===canvas&&controls.size)controls.values().next().value.focus({preventScroll:true});
  };
}
