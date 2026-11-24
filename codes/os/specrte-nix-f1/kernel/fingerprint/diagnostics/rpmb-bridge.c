/* SPDX-License-Identifier: GPL-2.0-only */
/* Experimental QSEECOM RPMB broker. See README.md before running. */
#include <linux/tee.h>
#include <sys/mman.h>
#include "rpmb-vendor/rpmb_ufs.c"
struct rpmb_stats rpmb={.fd=-1,.fd_ufs_bsg=-1};
const struct rpmb_frame read_result_reg_frame={.request_response={0,5}};
void rpmb_init_wakelock(void){} void rpmb_wakelock(void){} void rpmb_wakeunlock(void){}
#define BUFSZ 28672
static int dispatch(unsigned char *buf){
 uint32_t q[6];memcpy(q,buf,sizeof(q));
 printf("RPMB request cmd=0x%x version/blocks=%u\n",q[0],q[1]);
 if(q[0]==0x101){uint32_t a[10]={q[0],q[1]>=2?2:1,0,rpmb.info.size,rpmb.info.rel_wr_count,9};memcpy(buf,a,q[1]>=2?40:20);return 0;}
 if(q[0]==0x102){
  uint32_t bytes=0;int status=-1;uint32_t n=q[1],len=q[2],off=q[3],cap=rpmb.info.rel_wr_count;
  unsigned char in[BUFSZ],out[BUFSZ];
  if(cap && n && n<=(BUFSZ-20)/512 && off>=24 && off<=BUFSZ && len<=BUFSZ-off && len>=((n+cap-1)/cap)*512){
   memcpy(in,buf+off,len);int valid=1;
   for(unsigned i=0;i<(n+cap-1)/cap;i++){unsigned type=in[i*512+510]*256+in[i*512+511];if(type!=2&&type!=4)valid=0;printf("RPMB read frame type=%u\n",type);}
   if(valid)status=rpmb_ufs_read((void *)in,n,(void *)out,&bytes);
   if(!status&&bytes<=BUFSZ-20){memcpy(buf+20,out,bytes);for(unsigned i=0;i<bytes/512;i++)printf("RPMB response type=%u result=%u\n",out[i*512+510]*256+out[i*512+511],out[i*512+508]*256+out[i*512+509]);}
   else bytes=0;
   explicit_bzero(in,sizeof(in));explicit_bzero(out,sizeof(out));
  }
  uint32_t a[5]={q[0],status,bytes,20,q[4]};memcpy(buf,a,20);return 0;
 }
 if(q[0]==0x103){
  uint32_t bytes=0,n=q[1],len=q[2],off=q[3],batch=q[5];int status=-1;
  unsigned char in[BUFSZ],out[512];
  if(n && n<=BUFSZ/512 && off>=24 && off<=BUFSZ && len<=BUFSZ-off && len==n*512 && batch && batch<=rpmb.info.rel_wr_count && n%batch==0){
   memcpy(in,buf+off,len);int valid=1;
   for(unsigned i=0;i<n;i++)if(in[i*512+510]!=0 || in[i*512+511]!=3)valid=0;
   if(valid)status=rpmb_ufs_write((void *)in,n,(void *)out,&bytes,batch);
   if(!status&&bytes==512){memcpy(buf+20,out,bytes);printf("RPMB authenticated write response result=%u\n",out[508]*256+out[509]);}else bytes=0;
   explicit_bzero(in,sizeof(in));explicit_bzero(out,sizeof(out));
  }
  uint32_t a[5]={q[0],status,bytes,20,q[4]};memcpy(buf,a,20);return 0;
 }
 if(q[0]==0x104){uint32_t a[4]={q[0],-1,0,16};memcpy(buf,a,16);return 0;}
 return -1;
}
int main(int argc,char **argv){
 if(argc>2 || (argc==2 && strcmp(argv[1],"--geometry"))){fputs("usage: rpmb-bridge [--geometry]\n",stderr);return 2;}
 setvbuf(stdout,NULL,_IONBF,0);
 if(rpmb_ufs_init())return 1;
 printf("RPMB geometry sectors=%u reliable_frames=%u type=%u\n",rpmb.info.size,rpmb.info.rel_wr_count,rpmb.info.dev_type);
 if(argc>1)return 0;
 int fd=open("/dev/teepriv0",O_RDWR);if(fd<0){perror("teepriv0");return 1;}
 struct tee_ioctl_shm_alloc_data alloc={.size=BUFSZ};int shm=ioctl(fd,TEE_IOC_SHM_ALLOC,&alloc);if(shm<0){perror("shm");return 1;}
 unsigned char *buf=mmap(NULL,BUFSZ,PROT_READ|PROT_WRITE,MAP_SHARED,shm,0);if(buf==MAP_FAILED)return 1;
 struct {struct tee_ioctl_open_session_arg a;struct tee_ioctl_param p[2];} s={0};s.a.num_params=2;s.p[0]=(struct tee_ioctl_param){.attr=TEE_IOCTL_PARAM_ATTR_TYPE_VALUE_INPUT,.a=8192};s.p[1]=(struct tee_ioctl_param){.attr=TEE_IOCTL_PARAM_ATTR_TYPE_MEMREF_INOUT,.b=BUFSZ,.c=alloc.id};
 struct tee_ioctl_buf_data bd={.buf_ptr=(uintptr_t)&s,.buf_len=sizeof(s)};
 if(ioctl(fd,TEE_IOC_OPEN_SESSION,&bd)||s.a.ret){perror("register");return 1;}puts("RPMB authenticated-storage listener registered");
 for(;;){
 struct {struct tee_iocl_supp_recv_arg a;struct tee_ioctl_param p[2];} r={0};r.a.num_params=2;bd=(struct tee_ioctl_buf_data){.buf_ptr=(uintptr_t)&r,.buf_len=sizeof(r)};
 if(ioctl(fd,TEE_IOC_SUPPL_RECV,&bd)){perror("recv");break;}
 int ok=r.a.func==8192&&!dispatch(buf);
 struct {struct tee_iocl_supp_send_arg a;struct tee_ioctl_param p;} t={0};t.a.num_params=1;t.a.ret=ok?0:1;t.p=(struct tee_ioctl_param){.attr=TEE_IOCTL_PARAM_ATTR_TYPE_VALUE_OUTPUT,.a=r.p[0].a};
 bd=(struct tee_ioctl_buf_data){.buf_ptr=(uintptr_t)&t,.buf_len=sizeof(t)};if(ioctl(fd,TEE_IOC_SUPPL_SEND,&bd)){perror("send");break;}
 }close(fd);return 1;
}
