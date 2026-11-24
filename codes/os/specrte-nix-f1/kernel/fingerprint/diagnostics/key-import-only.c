/* SPDX-License-Identifier: GPL-2.0-only */
/* Import the wrapped authentication key into gfdip. No Gatekeeper credential,
 * enrollment, capture, save, or template operation. Do not log key material. */
#define FINGERPRINT_TA "gfdip"
#define main unused_preflight_main
#include "gatekeeper-preflight.c"
#undef main
int main(void) {
 int rc=1, loader=open("/dev/teepriv0",O_RDWR), fd=open("/dev/tee0",O_RDWR);
 uint32_t loaded=0,gf=0,km=0; int have_load=0,have_gf=0,have_km=0;
 struct shared l={.fd=-1,.ptr=MAP_FAILED},m=l,p=l;
 setvbuf(stdout,NULL,_IOLBF,0);
 if(loader<0||fd<0){perror("TEE open");goto done;}
 l=alloc_shm(loader);m=alloc_shm(fd);p=alloc_shm(fd);
 if(l.ptr==MAP_FAILED||m.ptr==MAP_FAILED||p.ptr==MAP_FAILED)goto done;
 if(open_session(loader,&l,&loaded)<0){usleep(50000);if(open_session(loader,&l,&loaded)<0)goto done;}
 have_load=1;
 if(open_session(fd,&m,&gf)<0)goto done;
 have_gf=1;
 strcpy(m.ptr,"keymaster64");
 struct {struct tee_ioctl_open_session_arg a;struct tee_ioctl_param p;} r={0};
 r.a.num_params=1;r.p=(struct tee_ioctl_param){.attr=TEE_IOCTL_PARAM_ATTR_TYPE_MEMREF_INPUT,.b=12,.c=m.id};
 struct tee_ioctl_buf_data b={.buf_ptr=(uintptr_t)&r,.buf_len=sizeof(r)};
 if(ioctl(fd,TEE_IOC_OPEN_SESSION,&b)<0||r.a.ret)goto done;
 km=r.a.session;have_km=1;
 memset(m.ptr,0,4096);uint32_t *q=m.ptr;q[0]=0x205;q[1]=2;
 if(km_call(fd,km,m,8))goto done;
 uint32_t *a=(void *)((char *)m.ptr+2048);
 if(a[1]<12||a[1]>256||a[2]==0||a[2]>256-a[1]){puts("Invalid envelope bounds");goto done;}
 memset(p.ptr,0,4096);memcpy(p.ptr,a,256);
 rc=invoke(fd,gf,m,p,109,256)?1:0;
 puts(rc?"Wrapped key import rejected":"Wrapped key import accepted; enrollment not attempted");
 done:
 if(p.ptr!=MAP_FAILED)explicit_bzero(p.ptr,4096);
 if(m.ptr!=MAP_FAILED)explicit_bzero(m.ptr,4096);
 if(have_km)close_session(fd,km);
 if(have_gf)close_session(fd,gf);
 if(have_load)close_session(loader,loaded);
 free_shm(&p);free_shm(&m);free_shm(&l);
 if(fd>=0)close(fd);
 if(loader>=0)close(loader);
 return rc;
}
