'use strict';
(async()=>{
  const config=await fetch('review.json').then(r=>r.json());
  const $=id=>document.getElementById(id);
  for(const key of ['title','purpose','changes','expected'])$(key).textContent=config[key];
  $('limits').textContent=config.limits?'Scope: '+config.limits:'';
  for(const step of config.steps){const li=document.createElement('li');li.textContent=step;$('steps').append(li);}
  const key='bhoo-review-'+config.test_id;
  let submitted=false;
  function disable(value){$('ok').disabled=value;$('reject').disabled=value;$('details').disabled=value;}
  function pending(){submitted=true;disable(true);$('status').textContent='Review queued. Waiting for the bot to save it. Keep this message until it is removed.';}
  function observe(update){
    const p=update.payload;
    if(!p||p.test_id!==config.test_id)return;
    if(p.protocol==='bhoo-test-review/v1'&&p.token===config.token)pending();
    if(p.protocol==='bhoo-test-review-ack/v1'&&p.saved){submitted=true;disable(true);localStorage.removeItem(key);$('status').textContent='Your '+(p.decision==='ok'?'approval':'rejection')+' was saved. This test message can now be removed.';}
  }
  if(!window.webxdc){disable(true);$('status').textContent='Open this test in Delta Chat to submit your review.';return;}
  const old=localStorage.getItem(key);if(old)pending();
  await window.webxdc.setUpdateListener(observe,0);
  async function submit(decision){
    if(submitted)return;
    const details=$('details').value.trim();
    if(decision==='rejected'&&!details){$('status').textContent='Please explain the problem before rejecting.';$('details').focus();return;}
    const review_id=window.crypto?.randomUUID?.()||Date.now().toString(36)+'-'+Math.random().toString(36).slice(2);
    const payload={protocol:'bhoo-test-review/v1',test_id:config.test_id,token:config.token,review_id,decision,details};
    try{pending();await window.webxdc.sendUpdate({payload},'Bhoo test review');localStorage.setItem(key,JSON.stringify(payload));}
    catch(error){submitted=false;disable(false);$('status').textContent='Could not queue your review. Please try again.';}
  }
  $('ok').addEventListener('click',()=>submit('ok'));
  $('reject').addEventListener('click',()=>submit('rejected'));
})().catch(()=>{document.getElementById('status').textContent='Could not load this test. Reopen the message to try again.';});
