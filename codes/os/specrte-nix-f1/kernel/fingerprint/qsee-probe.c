/* SPDX-License-Identifier: GPL-2.0-only */
/* Only discover QSEECOM, optionally load an OEM trustlet. No TA commands. */
#include <errno.h>
#include <fcntl.h>
#include <linux/tee.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/ioctl.h>
#include <sys/mman.h>
#include <unistd.h>
#define QSEECOM_IMPL 5
static int discover(const char *prefix) {
 char path[64];
 for(int n=0;n<16;n++){
  snprintf(path,sizeof(path),"/dev/%s%d",prefix,n);
  int fd=open(path,O_RDWR|O_CLOEXEC);if(fd<0)continue;
  struct tee_ioctl_version_data v={0};
  if(ioctl(fd,TEE_IOC_VERSION,&v)==0){
   printf("%s impl=%u caps=%u generic=%u\n",path,v.impl_id,v.impl_caps,v.gen_caps);
   if(v.impl_id==QSEECOM_IMPL)return fd;
  }
  close(fd);
 }
 return -1;
}
int main(int argc,char **argv){
 setvbuf(stdout,NULL,_IOLBF,0);
 int lookup = argc == 3 && !strcmp(argv[2], "--lookup");
 if(argc>3 || (argc==3 && !lookup) || (argc>=2 && strcmp(argv[1],"gfber") && strcmp(argv[1],"fpcber"))){
  fprintf(stderr,"usage: %s [gfber|fpcber [--lookup]]\n",argv[0]);return 2;
 }
 int fd=discover(argc>=2 && !lookup ? "teepriv" : "tee");
 if(fd<0){fprintf(stderr,"No QSEECOM TEE device\n");return 1;}
 if(argc==1){close(fd);return 0;}
 struct tee_ioctl_shm_alloc_data shm={.size=64};
 int memfd=ioctl(fd,TEE_IOC_SHM_ALLOC,&shm);
 if(memfd<0){perror("SHM_ALLOC");close(fd);return 1;}
 void *ptr=mmap(NULL,shm.size,PROT_READ|PROT_WRITE,MAP_SHARED,memfd,0);
 if(ptr==MAP_FAILED){perror("mmap");close(memfd);close(fd);return 1;}
 memset(ptr,0,shm.size);strcpy(ptr,argv[1]);
 struct {struct tee_ioctl_open_session_arg arg;struct tee_ioctl_param p;} req={0};
 req.arg.num_params=1;
 req.p.attr=TEE_IOCTL_PARAM_ATTR_TYPE_MEMREF_INPUT;
 req.p.b=strlen(argv[1])+1;req.p.c=shm.id;
 struct tee_ioctl_buf_data buf={.buf_ptr=(uintptr_t)&req,.buf_len=sizeof(req)};
 int rc=ioctl(fd,TEE_IOC_OPEN_SESSION,&buf);
 if(rc<0)perror(lookup ? "LOOKUP" : "LOAD");
 else if(req.arg.ret){fprintf(stderr,"LOAD ret=%u origin=%u\n",req.arg.ret,req.arg.ret_origin);rc=-1;}
 else {
  printf("%s %s, session=%u; no sensor commands sent\n",lookup ? "Opened resident" : "Loaded",argv[1],req.arg.session);
  struct tee_ioctl_close_session_arg closearg={.session=req.arg.session};
  if(ioctl(fd,TEE_IOC_CLOSE_SESSION,&closearg)<0){perror("CLOSE_SESSION");rc=-1;}
  else puts("Session closed");
 }
 munmap(ptr,shm.size);close(memfd);close(fd);return rc<0;
}
