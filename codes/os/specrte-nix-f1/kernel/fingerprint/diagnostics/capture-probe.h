/* SPDX-License-Identifier: GPL-2.0-only */
/* gfdip Milan HV IRQ protocol, corroborated against libgf_hal.so 0x3072c
 * and 0x30fec. Optional SAVE uses the isolated group and RAM filesystem broker.
 * Never export image/template data. */
#include <poll.h>
static uint32_t capture_u32(const void *p,size_t off) {
 uint32_t v;memcpy(&v,(const char *)p+off,sizeof(v));return v;
}
static void capture_put32(void *p,size_t off,uint32_t v) {memcpy((char *)p+off,&v,4);}
static int capture_probe(int tee,uint32_t session,struct shared request,int irq,int save_match,uint32_t expected_fid) {
 const size_t bytes=0x36a20;
 struct shared payload=alloc_shm_size(tee,bytes);
 if(payload.ptr==MAP_FAILED){perror("capture buffer");return -1;}
 struct gpio_v2_line_config cfg={.flags=GPIO_V2_LINE_FLAG_INPUT|GPIO_V2_LINE_FLAG_EDGE_RISING};
 int result=-1;unsigned events=0,samples=0;int matching=0;uint32_t saved_fid=0;
 if(ioctl(irq,GPIO_V2_LINE_SET_CONFIG_IOCTL,&cfg)<0){perror("IRQ events");goto out;}
 if(expected_fid){
  memset(payload.ptr,0,bytes);capture_put32(payload.ptr,96,1000);capture_put32(payload.ptr,112,1);
  if(invoke(tee,session,request,payload,1010,120))goto out;
  matching=1;saved_fid=expected_fid;
  puts("MATCH_READY: one attempt; no enrollment or credential changes");
 }
 struct timespec start,now;clock_gettime(CLOCK_MONOTONIC,&start);
 if(!expected_fid)puts("CAPTURE_READY: touch and lift the rear fingerprint sensor; capture window started");
 for(unsigned iteration=0;iteration<256 && !stop_requested;iteration++) {
  clock_gettime(CLOCK_MONOTONIC,&now);if(now.tv_sec-start.tv_sec>=100)break;
  struct pollfd pollfd={.fd=irq,.events=POLLIN};
  int ready=poll(&pollfd,1,500);
  if(ready<0){if(errno==EINTR)continue;perror("IRQ poll");goto out;}
  struct gpio_v2_line_values value={.mask=1};
  if(ioctl(irq,GPIO_V2_LINE_GET_VALUES_IOCTL,&value)<0){perror("IRQ level");goto out;}
  if(ready>0 && (pollfd.revents&POLLIN)) {
   struct gpio_v2_line_event event;
   if(read(irq,&event,sizeof(event))!=(ssize_t)sizeof(event)){perror("IRQ event");goto out;}
   events++;
  } else if(!(value.bits&1))continue;
  memset(payload.ptr,0,bytes);
  capture_put32(payload.ptr,0x367e1,0x2000000);
  capture_put32(payload.ptr,0x36810,0x200);
  capture_put32(payload.ptr,0x36a18,1);
  for(unsigned step=0;step<16 && !stop_requested;step++) {
   int sent=invoke(tee,session,request,payload,1016,bytes);
   uint32_t status=capture_u32(request.ptr,64);
   uint32_t flags=capture_u32(payload.ptr,96),mode=capture_u32(payload.ptr,100);
   printf("IRQ flags=0x%x mode=%u vendor=%u phase=%u\n",flags,mode,status,capture_u32(payload.ptr,0x36a18));
   if(sent && !status)goto out;
   if((flags&0x80) && mode==0 && (status==0 || status==0x415)) {
    uint16_t remaining;memcpy(&remaining,(char *)payload.ptr+0x36768,2);
    samples++;printf("ENROLL_PROGRESS remaining=%u accepted=%s\n",remaining,status==0?"yes":"duplicate");
    if(remaining==0){
     if(!save_match){puts("Capture sequence complete; cancelling without SAVE");result=0;goto out;}
     uint32_t first=capture_u32(payload.ptr,0x36760),second=capture_u32(payload.ptr,0x36764);
     if(first!=1000 || second==0){puts("Unexpected enrollment group/template fields; refusing SAVE");goto out;}
     memset(payload.ptr,0,bytes);capture_put32(payload.ptr,96,first);capture_put32(payload.ptr,100,second);
     if(invoke(tee,session,request,payload,1012,108)){puts("SAVE failed");goto out;}
     memset(payload.ptr,0,bytes);
     if(invoke(tee,session,request,payload,1015,180))goto out;
     unsigned count=0;int found=0;
     for(unsigned i=0;i<10;i++) {
      uint32_t fid=capture_u32(payload.ptr,140+4*i),gid=capture_u32(payload.ptr,100+4*i);
      if(fid)count++;
      if(fid==second && gid==first)found=1;
     }
     printf("SAVE_VERIFIED count=%u saved_template_listed=%s\n",count,found?"yes":"no");
     if(!found)goto out;
     if(save_match==2){result=0;goto out;}
     memset(payload.ptr,0,bytes);
     if(invoke(tee,session,request,payload,1009,96))goto out;
     memset(payload.ptr,0,bytes);capture_put32(payload.ptr,96,1000);capture_put32(payload.ptr,112,1);
     if(invoke(tee,session,request,payload,1010,120))goto out;
     saved_fid=second;matching=1;puts("MATCH_READY: waiting for fingerprint; no template bytes are exported");
     break;
    }
   }
   if(matching && (flags&0x80) && (mode==1 || mode==2)) {
    if(status==0 && capture_u32(payload.ptr,0x36760)==1000 && capture_u32(payload.ptr,0x36764)==saved_fid){puts("MATCH_RESULT accepted=yes identity=expected");result=0;}
    else if(status==0){puts("MATCH_RESULT identity mismatch");goto out;}
    else if(status==1006){
     if(((unsigned char *)payload.ptr)[108] || ((unsigned char *)payload.ptr)[109])
      puts("MATCH_RESULT inconclusive=fast_or_accidental_touch");
     else {puts("MATCH_RESULT accepted=no");if(expected_fid)result=0;}
    }
    else if(status==1064){
     puts("MATCH_RETRY: continuing vendor phase 2");
     capture_put32(payload.ptr,0x36a18,2);
    }
    else printf("MATCH_RESULT vendor_error=%u\n",status);
    if(expected_fid && status!=1064)goto out;
   }
   if(capture_u32(payload.ptr,0x36a18)!=2)break;
  }
 }
 printf("Capture window ended: IRQ events=%u sample reports=%u save_mode=%d\n",events,samples,save_match);
 if(!save_match)result=samples?0:-1;
out:
 explicit_bzero(payload.ptr,bytes);free_shm(&payload);return result;
}
