/* SPDX-License-Identifier: GPL-2.0-only */
/* Compare stock-sized and cache-line-padded requests. No credential creation,
 * template operations, or key import. Never log the wrapped response. */
#define main enrollment_preflight_main
#include "gatekeeper-preflight.c"
#undef main
int main(void)
{
 int rc=1,loader=open("/dev/teepriv0",O_RDWR),fd=open("/dev/tee0",O_RDWR);
 uint32_t gf=0,km=0;int loaded=0,opened=0;
 struct shared l={.fd=-1,.ptr=MAP_FAILED},m=l;
 if(loader<0||fd<0){perror("TEE open");goto done;}
 l=alloc_shm(loader);m=alloc_shm(fd);
 if(l.ptr==MAP_FAILED||m.ptr==MAP_FAILED)goto done;
 if(open_session(loader,&l,&gf)<0){usleep(50000);if(open_session(loader,&l,&gf)<0){perror("Goodix load");goto done;}}
 loaded=1;
 strcpy(m.ptr,"keymaster64");
 struct {struct tee_ioctl_open_session_arg a;struct tee_ioctl_param p;} r={0};
 r.a.num_params=1;r.p=(struct tee_ioctl_param){.attr=TEE_IOCTL_PARAM_ATTR_TYPE_MEMREF_INPUT,.b=12,.c=m.id};
 struct tee_ioctl_buf_data b={.buf_ptr=(uintptr_t)&r,.buf_len=sizeof(r)};
 if(ioctl(fd,TEE_IOC_OPEN_SESSION,&b)<0||r.a.ret){perror("Keymaster open");goto done;}
 km=r.a.session;opened=1;
 /* Type zero is an invalid-input control; expected status is -38. */
 const unsigned auth_types[]={2,1,4,0};
 for(unsigned auth=0;auth<sizeof(auth_types)/sizeof(auth_types[0]);auth++)
 for(unsigned profile=0;profile<3;profile++){
  unsigned request_size=profile?64:8,response_offset=profile==2?64:2048;
  memset(m.ptr,0,4096);uint32_t *q=m.ptr;q[0]=0x205;q[1]=auth_types[auth];
  struct {struct tee_ioctl_invoke_arg a;struct tee_ioctl_param p[2];} c={0};
  c.a.session=km;c.a.num_params=2;
  c.p[0]=(struct tee_ioctl_param){.attr=TEE_IOCTL_PARAM_ATTR_TYPE_MEMREF_INPUT,.b=request_size,.c=m.id};
  c.p[1]=(struct tee_ioctl_param){.attr=TEE_IOCTL_PARAM_ATTR_TYPE_MEMREF_OUTPUT,.a=response_offset,.b=256,.c=m.id};
  struct tee_ioctl_buf_data call={.buf_ptr=(uintptr_t)&c,.buf_len=sizeof(c)};
  printf("Auth type=%u; request bytes=%u; response offset=%u\n",auth_types[auth],request_size,response_offset);
  if(ioctl(fd,TEE_IOC_INVOKE,&call)<0){perror("Keymaster invoke");goto done;}
  int32_t status;memcpy(&status,(char *)m.ptr+response_offset,4);
  printf("Keymaster command=517 transport=%u status=%d\n",c.a.ret,status);
  /* Public envelope metadata only, never the key/envelope bytes. The TA
   * initializes these fields before encapsulation, but its dispatcher clears
   * the entire response on failure. Zero fields do not localize failures. */
  uint32_t envelope_offset,envelope_size;
  memcpy(&envelope_offset,(char *)m.ptr+response_offset+4,4);
  memcpy(&envelope_size,(char *)m.ptr+response_offset+8,4);
  printf("Envelope header offset=%u size=%u\n",envelope_offset,envelope_size);
  explicit_bzero(m.ptr,4096);
 }
 rc=0;
done:
 if(m.ptr!=MAP_FAILED)explicit_bzero(m.ptr,4096);
 if(opened)close_session(fd,km);
 if(loaded)close_session(loader,gf);
 free_shm(&m);free_shm(&l);if(fd>=0)close(fd);if(loader>=0)close(loader);
 return rc;
}
