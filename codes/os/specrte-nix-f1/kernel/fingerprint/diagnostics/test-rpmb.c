/* SPDX-License-Identifier: GPL-2.0-only */
/* Malformed requests must never reach an ioctl, especially KEY_PROVISION. */
#include <assert.h>
#include <stdarg.h>
static unsigned ioctl_count;
static int forbidden_ioctl(int fd, unsigned long request, ...) {
 (void)fd;(void)request;ioctl_count++;return -1;
}
#define ioctl forbidden_ioctl
#define main broker_main
#include "rpmb-bridge.c"
#undef main
#undef ioctl
static void reject(uint32_t cmd,uint32_t count,uint32_t len,uint32_t off,uint32_t batch,unsigned type){
 unsigned char buf[BUFSZ]={0};uint32_t q[6]={cmd,count,len,off,2,batch};memcpy(buf,q,sizeof(q));
 if(off<=BUFSZ-512){buf[off+510]=type>>8;buf[off+511]=type;}
 assert(dispatch(buf)==0);int32_t status;memcpy(&status,buf+4,4);assert(status!=0);assert(ioctl_count==0);
}
int main(void){
 rpmb.info.rel_wr_count=64;
 reject(0x103,1,512,24,1,1); /* permanent key programming */
 reject(0x103,1,512,24,1,4); /* read smuggled through write */
 reject(0x102,1,512,24,0,1); /* programming smuggled through read */
 reject(0x102,1,512,24,0,3); /* write smuggled through read */
 reject(0x103,1,512,UINT32_MAX,1,3);
 reject(0x103,UINT32_MAX,512,24,1,3);
 reject(0x103,1,511,24,1,3);
 reject(0x103,1,512,24,0,3);
 reject(0x103,1,512,24,65,3);
 reject(0x103,3,1536,24,2,3);
 reject(0x102,UINT32_MAX,512,24,0,4);
 reject(0x102,1,512,UINT32_MAX,0,4);
 reject(0x102,1,511,24,0,4);
 puts("PASS: 13 invalid/key-programming requests rejected without storage ioctl");return 0;
}
