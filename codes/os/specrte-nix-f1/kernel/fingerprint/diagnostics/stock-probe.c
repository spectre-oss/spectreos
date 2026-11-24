/* SPDX-License-Identifier: GPL-2.0-only */
/* Minimal stock QSEECOM ABI parity test. No listeners, credential or template
 * commands. Only signed gfber loading, version query and wrapped-key request.
 * Uses published LineageOS sdm845 UAPI headers fetched by build-stock-probe.sh. */
#define _GNU_SOURCE
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <fcntl.h>
#include <errno.h>
#include <unistd.h>
#include <sys/ioctl.h>
#include <sys/mman.h>
#include <sys/stat.h>
#include "vendor-qseecom-uapi.h"
#include "vendor-ion-uapi.h"
struct buffer {int ion,fd,handle;void *p;size_t size;};
static void release(struct buffer *b){
 if(b->p!=MAP_FAILED){explicit_bzero(b->p,b->size);munmap(b->p,b->size);}
 if(b->fd>=0)close(b->fd);
 if(b->ion>=0){if(b->handle){struct ion_handle_data h={b->handle};ioctl(b->ion,ION_IOC_FREE,&h);}close(b->ion);}
}
static int allocate(struct buffer *b,size_t size,unsigned heap){
 *b=(struct buffer){.ion=-1,.fd=-1,.p=MAP_FAILED,.size=(size+4095)&~(size_t)4095};
 b->ion=open("/dev/ion",O_RDWR|O_CLOEXEC);if(b->ion<0){perror("open ion");return -1;}
 struct ion_allocation_data a={.len=b->size,.align=4096,.heap_id_mask=1u<<heap,.flags=ION_FLAG_CACHED};
 if(ioctl(b->ion,ION_IOC_ALLOC,&a)<0){perror("ION alloc");return -1;}b->handle=a.handle;
 struct ion_fd_data f={.handle=a.handle,.fd=-1};if(ioctl(b->ion,ION_IOC_SHARE,&f)<0){perror("ION share");return -1;}b->fd=f.fd;
 b->p=mmap(NULL,b->size,PROT_READ|PROT_WRITE,MAP_SHARED,b->fd,0);if(b->p==MAP_FAILED){perror("ION mmap");return -1;}
 memset(b->p,0,b->size);return 0;
}
static int query(int fd,const char *name){
 struct qseecom_qseos_app_load_query q={0};snprintf(q.app_name,sizeof(q.app_name),"%s",name);
 errno=0;int r=ioctl(fd,QSEECOM_IOCTL_APP_LOADED_QUERY_REQ,&q),e=errno;
 printf("Lookup %s: syscall=%d errno=%d app=%u arch=%u\n",name,r,e,q.app_id,q.app_arch);
 return r<0&&e==EEXIST&&q.app_id?1:r==0?0:-1;
}
int main(int argc,char **argv){
 const char *name="gfber";int resident_only=0;
 if(argc==3 && !strcmp(argv[1],"--resident") &&
    (!strcmp(argv[2],"gfdip") || !strcmp(argv[2],"gfber"))){name=argv[2];resident_only=1;}
 else if(argc!=1){fprintf(stderr,"Usage: %s [--resident gfdip|gfber]\n",argv[0]);return 2;}
 setvbuf(stdout,NULL,_IONBF,0);alarm(30);int rc=1,gf=-1,km=-1,loaded=0,img=-1;
 struct buffer image={.ion=-1,.fd=-1,.p=MAP_FAILED},shared=image;
 puts("Stock ABI probe: no persistent storage or enrollment commands");
 gf=open("/dev/qseecom",O_RDWR|O_CLOEXEC);km=open("/dev/qseecom",O_RDWR|O_CLOEXEC);
 if(gf<0||km<0){perror("open qseecom");goto out;}
 struct qseecom_qseos_version_req v={0};if(ioctl(km,QSEECOM_IOCTL_GET_QSEOS_VERSION_REQ,&v)<0){perror("QSEE version");goto out;}
 printf("QSEE version=%u\n",v.qseos_version);
 int exists=query(gf,name);if(exists<0)goto out;
 if(!exists){
  if(resident_only){puts("Requested resident trustlet absent; refusing to load");goto out;}
  img=open("/gfber.img",O_RDONLY|O_CLOEXEC);struct stat st;
  if(img<0||fstat(img,&st)||st.st_size!=1562194){puts("Missing/unexpected gfber image");goto out;}
  if(allocate(&image,(size_t)st.st_size,19))goto out;
  size_t off=0;while(off<(size_t)st.st_size){ssize_t n=read(img,(char*)image.p+off,(size_t)st.st_size-off);if(n<=0){perror("read image");goto out;}off+=(size_t)n;}
  struct qseecom_load_img_req l={.mdt_len=7004,.img_len=(uint32_t)st.st_size,.ifd_data_fd=image.fd,.app_arch=1};strcpy(l.img_name,"gfber");
  if(ioctl(gf,QSEECOM_IOCTL_LOAD_APP_REQ,&l)<0){perror("load gfber");goto out;}
  loaded=1;printf("Goodix loaded: app=%u\n",l.app_id);
 }
 if(query(km,"keymaster64")!=1){puts("Resident Keymaster unavailable; refusing replacement");goto out;}
 if(allocate(&shared,4096,27))goto out;
 struct qseecom_set_sb_mem_param_req mem={.ifd_data_fd=shared.fd,.virt_sb_base=shared.p,.sb_len=4096};
 if(ioctl(km,QSEECOM_IOCTL_SET_MEM_PARAM_REQ,&mem)<0){perror("set shared memory");goto out;}
 const uint32_t cmds[]={0x200,0x205};
 for(unsigned i=0;i<2;i++){
  memset(shared.p,0,4096);uint32_t *q=shared.p;q[0]=cmds[i];if(i)q[1]=2;
  struct qseecom_send_cmd_req c={.cmd_req_buf=shared.p,.cmd_req_len=64,.resp_buf=(char*)shared.p+64,.resp_len=256};
  if(ioctl(km,QSEECOM_IOCTL_SEND_CMD_REQ,&c)<0){perror("send command");goto out;}
  int32_t status;memcpy(&status,c.resp_buf,4);printf("Keymaster command=%u status=%d\n",cmds[i],status);
  if(!i && status==0){uint32_t *a=c.resp_buf;printf("Keymaster API=%u TA=%u.%u\n",a[1],a[2],a[3]);}
  explicit_bzero(shared.p,4096);
 }
 rc=0;
out:
 if(loaded && ioctl(gf,QSEECOM_IOCTL_UNLOAD_APP_REQ)<0)perror("unload gfber");
 if(km>=0)close(km);
 if(gf>=0)close(gf);
 if(img>=0)close(img);
 release(&shared);release(&image);puts("Probe finished; no token or envelope logged");return rc;
}
