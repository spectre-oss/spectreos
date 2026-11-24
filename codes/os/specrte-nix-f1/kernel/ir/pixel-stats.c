/* SPDX-License-Identifier: GPL-2.0-only */
/* Aggregate Y10/Y10P data in RAM; never print individual pixels. */
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
static uint64_t sum, count, frame;
static unsigned low=1023, high;
static void report(void) {
 printf("frame=%llu samples=%llu min=%u max=%u mean=%.3f\n",
  (unsigned long long)frame++, (unsigned long long)count, low, high,
  (double)sum/count);
 sum=count=0; low=1023; high=0;
}
static void sample(unsigned v, unsigned frame_pixels) {
 if(v<low)low=v;
 if(v>high)high=v;
 sum+=v;count++;
 if(frame_pixels && count==frame_pixels)report();
}
int main(int argc,char **argv) {
 int packed=0;unsigned frame_pixels=0;
 for(int i=1;i<argc;i++) {
  if(!strcmp(argv[i],"--packed"))packed=1;
  else if(!strcmp(argv[i],"--frame-pixels") && i+1<argc) {
   char *end; unsigned long n=strtoul(argv[++i],&end,10);
   if(*end || !n || n>16777216)return 2;
   frame_pixels=(unsigned)n;
  } else return 2;
 }
 if(packed && frame_pixels%4)return 2;
 unsigned char b[5];size_t size=packed?5:2,n;
 while((n=fread(b,1,size,stdin))) {
  if(n!=size){fputs("incomplete sample group\n",stderr);return 1;}
  if(packed) {
   for(unsigned i=0;i<4;i++)sample(((unsigned)b[i]<<2)|((b[4]>>(2*i))&3),frame_pixels);
  } else {
   unsigned v=(unsigned)b[0]|((unsigned)b[1]<<8);
   if(v>1023){fputs("not unpacked 10-bit data\n",stderr);return 1;}
   sample(v,frame_pixels);
  }
 }
 if(ferror(stdin)||(!count&&!frame))return 1;
 if(frame_pixels && count){fputs("incomplete frame\n",stderr);return 1;}
 if(count)report();
 return 0;
}
