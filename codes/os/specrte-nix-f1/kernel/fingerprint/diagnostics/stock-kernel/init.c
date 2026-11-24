#define _GNU_SOURCE
#include <arpa/inet.h>
#include <dirent.h>
#include <errno.h>
#include <fcntl.h>
#include <linux/reboot.h>
#include <net/if.h>
#include <poll.h>
#include <signal.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/ioctl.h>
#include <sys/mount.h>
#include <sys/socket.h>
#include <sys/stat.h>
#include <sys/syscall.h>
#include <sys/sysmacros.h>
#include <sys/wait.h>
#include <time.h>
#include <unistd.h>

/* PID 1 has no shell or command endpoint. All files written are RAM/configfs. */
static void dir(const char *p) { if (mkdir(p,0755) && errno!=EEXIST) perror(p); }
static int put(const char *p,const char *s) { int f=open(p,O_WRONLY|O_CLOEXEC); if(f<0){perror(p);return -1;} size_t n=strlen(s); ssize_t r=write(f,s,n);close(f);return r==(ssize_t)n?0:-1; }
static void devnode(const char *sys,const char *name) { unsigned a,b;FILE *f=fopen(sys,"r");if(!f)return;if(fscanf(f,"%u:%u",&a,&b)==2 && mknod(name,S_IFCHR|0600,makedev(a,b)) && errno!=EEXIST)perror(name);fclose(f); }
static long now(void){struct timespec t;clock_gettime(CLOCK_MONOTONIC,&t);return t.tv_sec;}
static void reboot_bootloader(void){syscall(SYS_reboot,LINUX_REBOOT_MAGIC1,LINUX_REBOOT_MAGIC2,LINUX_REBOOT_CMD_RESTART2,"bootloader");for(;;)pause();}
static int network(void){
 const char *g="/config/usb_gadget/diag";char p[256];dir("/config/usb_gadget/diag");
 #define PATH(s) (snprintf(p,sizeof(p),"%s/%s",g,s),p)
 if(put(PATH("idVendor"),"0x18d1")||put(PATH("idProduct"),"0x4ee3"))return -1;
 dir(PATH("strings/0x409"));put(PATH("strings/0x409/serialnumber"),"poco-readonly-diagnostic");put(PATH("strings/0x409/manufacturer"),"Diagnostic");put(PATH("strings/0x409/product"),"Poco kernel probe");
 dir(PATH("configs/c.1"));dir(PATH("functions/ncm.usb0"));put(PATH("functions/ncm.usb0/dev_addr"),"02:00:00:00:42:01");put(PATH("functions/ncm.usb0/host_addr"),"02:00:00:00:42:02");
 if(symlink("../../functions/ncm.usb0",PATH("configs/c.1/ncm.usb0")) && errno!=EEXIST)return -1;
 DIR *d=opendir("/sys/class/udc");if(!d)return -1;struct dirent *e;char udc[256]={0};while((e=readdir(d)))if(e->d_name[0]!='.'){snprintf(udc,sizeof(udc),"%s",e->d_name);break;}closedir(d);if(!udc[0]||put(PATH("UDC"),udc))return -1;
 int s=socket(AF_INET,SOCK_DGRAM|SOCK_CLOEXEC,0);if(s<0)return -1;struct ifreq r={0};strcpy(r.ifr_name,"usb0");struct sockaddr_in *a=(void*)&r.ifr_addr;a->sin_family=AF_INET;inet_pton(AF_INET,"172.16.42.1",&a->sin_addr);int ok=ioctl(s,SIOCSIFADDR,&r);inet_pton(AF_INET,"255.255.255.0",&a->sin_addr);ok|=ioctl(s,SIOCSIFNETMASK,&r);r.ifr_flags=IFF_UP|IFF_RUNNING;ok|=ioctl(s,SIOCSIFFLAGS,&r);close(s);return ok;
}
int main(void){
#ifdef DIAGNOSTIC_CHILD
 if(getppid()!=1){
#else
 if(getpid()!=1){
#endif
 fputs("Must run as isolated initramfs PID 1\n",stderr);return 1;}
 long deadline=now()+180;signal(SIGPIPE,SIG_IGN);
 /* Independent watchdog survives a stalled probe or USB setup. */
 pid_t w=fork();if(!w){sleep(180);reboot_bootloader();}if(w<0)reboot_bootloader();
 dir("/proc");dir("/sys");dir("/dev");dir("/run");dir("/config");
 if(mount("proc","/proc","proc",MS_NOSUID|MS_NODEV|MS_NOEXEC,NULL)||mount("sysfs","/sys","sysfs",MS_NOSUID|MS_NODEV|MS_NOEXEC,NULL)||mount("tmpfs","/dev","tmpfs",MS_NOSUID,"mode=0755,size=1m")||mount("tmpfs","/run","tmpfs",MS_NOSUID|MS_NODEV|MS_NOEXEC,"mode=0700,size=2m")||mount("configfs","/config","configfs",MS_NOSUID|MS_NODEV|MS_NOEXEC,NULL))reboot_bootloader();
 mknod("/dev/null",S_IFCHR|0600,makedev(1,3));mknod("/dev/console",S_IFCHR|0600,makedev(5,1));
 int log=open("/run/diagnostic.log",O_CREAT|O_RDWR|O_APPEND|O_CLOEXEC,0600);if(log<0)reboot_bootloader();dup2(log,1);dup2(log,2);setvbuf(stdout,NULL,_IONBF,0);setvbuf(stderr,NULL,_IONBF,0);
 puts("Minimal stock kernel diagnostic; RAM only; automatic bootloader restart in 180 seconds.");
 int net=-1;for(int i=0;i<20 && net;i++){net=network();if(net)sleep(1);}printf("USB NCM setup: %d\n",net);
 for(int i=0;i<20;i++){devnode("/sys/class/qseecom/qseecom/dev","/dev/qseecom");devnode("/sys/class/misc/ion/dev","/dev/ion");if(!access("/dev/qseecom",F_OK)&&!access("/dev/ion",F_OK))break;sleep(1);}
 puts("Probe scheduled after 15 seconds; USB log server starting first.");
 pid_t probe=fork();if(!probe){sleep(15);int n=open("/dev/null",O_RDONLY);if(n>=0)dup2(n,0);execl("/probe","/probe",NULL);perror("probe exec");_exit(127);}long probe_deadline=now()+40;
 int server=socket(AF_INET,SOCK_STREAM|SOCK_CLOEXEC|SOCK_NONBLOCK,0);struct sockaddr_in addr={.sin_family=AF_INET,.sin_port=htons(8080)};inet_pton(AF_INET,"172.16.42.1",&addr.sin_addr);
 if(server<0||bind(server,(void*)&addr,sizeof(addr))||listen(server,2))perror("log server");
 while(now()<deadline){int status;pid_t r=waitpid(probe,&status,WNOHANG);if(probe>0&&r==probe){printf("Probe exit status: %d\n",status);probe=-1;}if(probe>0&&now()>probe_deadline){kill(probe,SIGKILL);puts("Probe timed out");probe_deadline=deadline;}
  struct pollfd p={.fd=server,.events=POLLIN};if(poll(&p,1,250)<=0)continue;int c=accept4(server,NULL,NULL,SOCK_CLOEXEC|SOCK_NONBLOCK);if(c<0)continue;
  /* Fixed response only. No URL parser, file selector or command processing. */
  char req[1024];struct pollfd cp={.fd=c,.events=POLLIN};poll(&cp,1,100);(void)read(c,req,sizeof(req));
  char data[65536];ssize_t n=pread(log,data,sizeof(data),0);if(n<0)n=0;char hdr[180];int hn=snprintf(hdr,sizeof(hdr),"HTTP/1.0 200 OK\r\nContent-Type: text/plain\r\nContent-Length: %zd\r\nConnection: close\r\n\r\n",n);(void)write(c,hdr,(size_t)hn);for(ssize_t off=0;off<n;){ssize_t a=write(c,data+off,(size_t)(n-off));if(a<=0)break;off+=a;}close(c);
 }
 reboot_bootloader();
}
