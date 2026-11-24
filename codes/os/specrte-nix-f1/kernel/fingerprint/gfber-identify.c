/* SPDX-License-Identifier: GPL-2.0-only */
/* Beryllium class-1 diagnostics: identify, enumerate, and enrollment preflight.
 * Enrollment preflight is always cancelled; this probe never sends SAVE. */
#include <errno.h>
#include <fcntl.h>
#include <linux/gpio.h>
#include <linux/tee.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/ioctl.h>
#include <sys/mman.h>
#include <time.h>
#include <unistd.h>

struct shared { int fd; int id; void *ptr; };
static struct shared alloc_shm(int tee)
{
 struct tee_ioctl_shm_alloc_data a = {.size=4096};
 struct shared m = {.fd=-1, .ptr=MAP_FAILED};
 m.fd=ioctl(tee, TEE_IOC_SHM_ALLOC, &a);
 if(m.fd<0) return m;
 m.id=a.id;
 m.ptr=mmap(NULL,4096,PROT_READ|PROT_WRITE,MAP_SHARED,m.fd,0);
 return m;
}
static void free_shm(struct shared *m)
{
 if(m->ptr!=MAP_FAILED) munmap(m->ptr,4096);
 if(m->fd>=0) close(m->fd);
}
static int open_session(int tee,struct shared *m,uint32_t *id)
{
 struct {struct tee_ioctl_open_session_arg a; struct tee_ioctl_param p;} r={0};
 strcpy(m->ptr,"gfber");
 r.a.num_params=1;
 r.p=(struct tee_ioctl_param){.attr=TEE_IOCTL_PARAM_ATTR_TYPE_MEMREF_INPUT,.b=6,.c=m->id};
 struct tee_ioctl_buf_data b={.buf_ptr=(uintptr_t)&r,.buf_len=sizeof(r)};
 if(ioctl(tee,TEE_IOC_OPEN_SESSION,&b)<0) return -1;
 if(r.a.ret){fprintf(stderr,"Open returned %u\n",r.a.ret);return -1;}
 *id=r.a.session;
 return 0;
}
static void close_session(int tee,uint32_t id)
{
 struct tee_ioctl_close_session_arg a={.session=id};
 if(ioctl(tee,TEE_IOC_CLOSE_SESSION,&a)<0) perror("close session");
}
static int line(int chip,unsigned n,int output)
{
 struct gpio_v2_line_request r={0};
 r.offsets[0]=n;r.num_lines=1;
 strcpy(r.consumer,"gfber-identify");
 r.config.flags=output?GPIO_V2_LINE_FLAG_OUTPUT:GPIO_V2_LINE_FLAG_INPUT;
 if(ioctl(chip,GPIO_V2_GET_LINE_IOCTL,&r)<0)return -1;
 return r.fd;
}
static int set(int fd,int value)
{
 struct gpio_v2_line_values v={.mask=1,.bits=value};
 return ioctl(fd,GPIO_V2_LINE_SET_VALUES_IOCTL,&v);
}
static int invoke(int tee,uint32_t ts,struct shared m,struct shared payload,unsigned command,unsigned size)
{
 memset(m.ptr,0,4096);
 uint32_t *req=m.ptr;
 req[0]=command-999;req[1]=1;req[4]=command;req[6]=size;
 struct timespec now;clock_gettime(CLOCK_REALTIME,&now);
 uint64_t ms=((uint64_t)now.tv_sec%86400)*1000+now.tv_nsec/1000000;
 memcpy((char *)m.ptr+8,&ms,sizeof(ms));
 struct {struct tee_ioctl_invoke_arg a;struct tee_ioctl_param p[4];} call={0};
 call.a.session=ts;call.a.num_params=4;
 call.p[0]=(struct tee_ioctl_param){.attr=TEE_IOCTL_PARAM_ATTR_TYPE_MEMREF_INPUT,.b=64,.c=m.id};
 call.p[1]=(struct tee_ioctl_param){.attr=TEE_IOCTL_PARAM_ATTR_TYPE_MEMREF_OUTPUT,.a=64,.b=64,.c=m.id};
 call.p[2]=(struct tee_ioctl_param){.attr=TEE_IOCTL_PARAM_ATTR_TYPE_VALUE_INPUT,.a=20,.b=4};
 call.p[3]=(struct tee_ioctl_param){.attr=TEE_IOCTL_PARAM_ATTR_TYPE_MEMREF_INOUT,.b=size,.c=payload.id};
 struct tee_ioctl_buf_data b={.buf_ptr=(uintptr_t)&call,.buf_len=sizeof(call)};
 printf("Sending vendor class 1 command %u (%u-byte payload)\n",command,size);
 if(ioctl(tee,TEE_IOC_INVOKE,&b)<0){perror("invoke");return -1;}
 printf("Transport ret=%u origin=%u\n",call.a.ret,call.a.ret_origin);
 uint32_t vendor_status;
 memcpy(&vendor_status,(char *)m.ptr+64,sizeof(vendor_status));
 printf("Vendor status=%u\n",vendor_status);
 if(call.a.ret||vendor_status)return -1;
 return 0;
}
int main(int argc,char **argv)
{
 int list_only=argc==2&&!strcmp(argv[1],"--list");
 int enroll_probe=argc==2&&!strcmp(argv[1],"--enroll-probe");
 int enrollment_probe=list_only || enroll_probe || (argc==2&&!strcmp(argv[1],"--enrollment-probe"));
 int power_off=argc==2&&!strcmp(argv[1],"--power-off");
 if(argc>2||(argc==2&&!power_off&&!enrollment_probe)){fputs("usage: gfber-identify [--power-off|--list|--enrollment-probe|--enroll-probe]\n",stderr);return 2;}
 int rc=1,chip=-1,pwr=-1,rst=-1,irq=-1;
 int loader=open("/dev/teepriv0",O_RDWR),tee=open("/dev/tee0",O_RDWR);
 int loaded=0,opened=0;
 uint32_t ls=0,ts=0;
 struct shared lm={.fd=-1,.ptr=MAP_FAILED},m=lm,payload=lm;
 setvbuf(stdout,NULL,_IOLBF,0);
 if(loader<0||tee<0){perror("TEE open");goto out;}
 struct tee_ioctl_version_data v={0};
 if(ioctl(tee,TEE_IOC_VERSION,&v)<0||v.impl_id!=5){fputs("Not QSEECOM\n",stderr);goto out;}
 lm=alloc_shm(loader);m=alloc_shm(tee);payload=alloc_shm(tee);
 if(lm.ptr==MAP_FAILED||m.ptr==MAP_FAILED||payload.ptr==MAP_FAILED){perror("shared memory");goto out;}
 if(open_session(loader,&lm,&ls)<0){
  if(errno!=EINVAL){perror("load gfber");goto out;}
  /* Observed on the first load after boot; one retry, no error masking. */
  fputs("Initial gfber load returned EINVAL; retrying once\n",stderr);
  usleep(50000);
  if(open_session(loader,&lm,&ls)<0){perror("load gfber retry");goto out;}
 }
 loaded=1;
 if(open_session(tee,&m,&ts)<0){perror("open gfber");goto out;}
 opened=1;
 chip=open("/dev/gpiochip2",O_RDONLY);
 struct gpiochip_info ci={0};
 if(chip<0||ioctl(chip,GPIO_GET_CHIPINFO_IOCTL,&ci)<0||strcmp(ci.label,"3400000.pinctrl")){
  fputs("Unexpected GPIO chip\n",stderr);goto out;
 }
 irq=line(chip,121,0);rst=line(chip,37,1);pwr=line(chip,94,1);
 if(irq<0||rst<0||pwr<0){perror("GPIO request");goto out;}
 if(!power_off){
 if(set(pwr,1)<0){perror("power");goto out;}usleep(10000);
 if(set(rst,1)<0){perror("reset");goto out;}usleep(10000);
 }
 for(unsigned command=1000;command<=1001;command++){
 unsigned size=command==1000?356:168;
 memset(payload.ptr,0,4096);
 if(invoke(tee,ts,m,payload,command,size))goto out;
 if(command==1001){
  uint32_t rows,cols,otp_size;
  unsigned nonzero=0;
  memcpy(&rows,(char *)payload.ptr+100,4);
  memcpy(&cols,(char *)payload.ptr+104,4);
  memcpy(&otp_size,(char *)payload.ptr+164,4);
  for(unsigned i=132;i<164;i++)nonzero|=((unsigned char *)payload.ptr)[i];
  printf("Sensor geometry=%ux%u; OTP bytes=%u; OTP data=%s\n",rows,cols,otp_size,nonzero?"present":"zero");
  /* Do not print the device's OTP contents. */
  if(power_off ? (otp_size||nonzero) : (otp_size!=32||!nonzero)){
   fputs("Unexpected powered/power-off sensor response\n",stderr);goto out;
  }
 }

 }
 if(enrollment_probe){
  memset(payload.ptr,0,4096);
  if(invoke(tee,ts,m,payload,1005,100))goto out;
  memset(payload.ptr,0,4096);
  uint32_t group=1000;memcpy((char *)payload.ptr+96,&group,4);
  if(invoke(tee,ts,m,payload,1014,100))goto out;
  memset(payload.ptr,0,4096);
  if(invoke(tee,ts,m,payload,1015,180))goto out;
  unsigned count=0;
  for(unsigned i=0;i<10;i++){
   uint32_t fid;memcpy(&fid,(char *)payload.ptr+140+4*i,4);
   if(fid)count++;
  }
  printf("Group 1000 template count=%u\n",count);
  if(list_only){rc=0;goto out;}
  memset(payload.ptr,0,4096);
  if(invoke(tee,ts,m,payload,1006,104))goto out;
  uint64_t challenge;memcpy(&challenge,(char *)payload.ptr+96,8);
  printf("Enrollment challenge=%s\n",challenge?"present":"absent");
  if(enroll_probe){
   if(!challenge){fputs("Missing enrollment challenge\n",stderr);goto out;}
   memset(payload.ptr,0,4096);
   memcpy((char *)payload.ptr+96,&group,4);
   memcpy((char *)payload.ptr+106,&challenge,8);
   int started=invoke(tee,ts,m,payload,1007,176);
   memset(payload.ptr,0,4096);
   int cancelled=invoke(tee,ts,m,payload,1009,96);
   if(started||cancelled)goto out;
   puts("Enrollment start accepted; cancelled without capturing or saving");
  }
 }
 rc=0;
out:
 if(rst>=0){set(rst,0);close(rst);}
 if(pwr>=0){set(pwr,0);struct gpio_v2_line_config c={.flags=GPIO_V2_LINE_FLAG_INPUT};ioctl(pwr,GPIO_V2_LINE_SET_CONFIG_IOCTL,&c);close(pwr);}
 if(irq>=0)close(irq);
 if(chip>=0)close(chip);
 if(opened)close_session(tee,ts);
 if(loaded)close_session(loader,ls);
 free_shm(&payload);free_shm(&m);free_shm(&lm);
 if(tee>=0)close(tee);
 if(loader>=0)close(loader);
 return rc;
}
