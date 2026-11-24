// Decode an offline video with the browser, while Bhoo paints every visible pixel.
export function createVideoBridge(engine, isActive, request, report) {
  const video=document.createElement('video');
  const surface=document.createElement('canvas');
  const context=surface.getContext('2d',{willReadFrequently:true});
  let closed=false,callback=0,sourceUrl=null;
  video.preload='auto';video.muted=true;video.playsInline=true;
  function active(){return !closed&&isActive();}
  function sync(){
    if(!active())return;
    const duration=Number.isFinite(video.duration)?video.duration:0;
    let buffered=0;
    if(duration>0&&video.buffered.length)buffered=Math.round(Math.max(0,Math.min(1,video.buffered.end(video.buffered.length-1)/duration))*100);
    engine.bhoo_video_host_status(Number(video.readyState>=1&&video.videoWidth>0),video.videoWidth,video.videoHeight,video.currentTime*1000,duration*1000,Number(!video.paused&&!video.ended),buffered);
  }
  function capture(){
    if(!active()||video.readyState<2||!video.videoWidth||!video.videoHeight)return;
    try {
      const scale=Math.min(1,320/video.videoWidth,180/video.videoHeight);
      const width=Math.max(1,Math.round(video.videoWidth*scale)),height=Math.max(1,Math.round(video.videoHeight*scale));
      if(surface.width!==width||surface.height!==height){surface.width=width;surface.height=height;}
      context.drawImage(video,0,0,width,height);
      const pixels=context.getImageData(0,0,width,height).data;
      const pointer=engine.bhoo_video_frame_buffer(width,height);
      if(!pointer)throw Error('Video frame exceeds the buffer limit');
      new Uint8Array(engine.memory.buffer,pointer,pixels.length).set(pixels);
      if(!engine.bhoo_video_frame_commit(width,height))throw Error('Invalid decoded video frame');
      sync();request();
    } catch(error) {report(`Video frame error: ${error.message}`);close();}
  }
  function schedule(){
    if(!active()||video.paused||video.ended||callback)return;
    if(video.requestVideoFrameCallback)callback=video.requestVideoFrameCallback(()=>{callback=0;capture();schedule();});
    else callback=requestAnimationFrame(()=>{callback=0;capture();schedule();});
  }
  function onPlaying(){sync();capture();schedule();}
  function onFrame(){sync();capture();}
  function onState(){sync();request();}
  video.addEventListener('loadeddata',onFrame);video.addEventListener('seeked',onFrame);
  video.addEventListener('playing',onPlaying);video.addEventListener('pause',onState);video.addEventListener('ended',onState);video.addEventListener('durationchange',onState);video.addEventListener('progress',onState);
  video.addEventListener('error',()=>{if(active())report('The bundled video could not be decoded on this device.');});
  function close(){
    if(closed)return;closed=true;
    if(callback){if(video.cancelVideoFrameCallback)video.cancelVideoFrameCallback(callback);else cancelAnimationFrame(callback);callback=0;}
    video.pause();video.removeAttribute('src');video.load();if(sourceUrl){URL.revokeObjectURL(sourceUrl);sourceUrl=null;}
  }
  function consume(){
    if(!active())return;
    const action=engine.bhoo_video_take_action();
    if(action===1){const promise=video.play();if(promise?.catch)promise.catch(()=>{if(active())report('Video playback could not start.');});}
    else if(action===2)video.pause();
    else if(action===3&&Number.isFinite(video.duration))video.currentTime=Math.max(0,Math.min(video.duration,engine.bhoo_video_action_value()/1000));
  }
  sync();
  // A local Blob remains seekable even when the archive host cannot serve byte ranges.
  fetch('bhoo-test-video.webm').then(response=>{if(!response.ok)throw Error(`HTTP ${response.status}`);return response.blob();}).then(blob=>{if(closed)return;sourceUrl=URL.createObjectURL(blob);video.src=sourceUrl;video.load();}).catch(error=>{if(active())report(`Video load failed: ${error.message}`);});
  return {consume,close,sync,suspend:()=>video.pause()};
}
