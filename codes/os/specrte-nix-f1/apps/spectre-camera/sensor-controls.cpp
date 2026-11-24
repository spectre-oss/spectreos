// Sensor diagnostic utility, separate from the libcamera application backend.
#include <linux/videodev2.h>
#include <sys/ioctl.h>
#include <fcntl.h>
#include <unistd.h>
#include <cerrno>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <map>
#include <string>
int main(int argc,char **argv) {
    if (argc!=2 && argc!=5) { fprintf(stderr,"Usage: spectre-sensor-controls /dev/v4l-subdevN [--set NAME VALUE]\n"); return 2; }
    int fd=open(argv[1],O_RDWR|O_CLOEXEC);
    if (fd<0) { perror("Open sensor"); return 1; }
    std::map<std::string,unsigned> ids={{"exposure",V4L2_CID_EXPOSURE},{"analogue_gain",V4L2_CID_ANALOGUE_GAIN},
        {"digital_gain",V4L2_CID_DIGITAL_GAIN},{"test_pattern",V4L2_CID_TEST_PATTERN},{"vblank",V4L2_CID_VBLANK},
        {"hflip",V4L2_CID_HFLIP},{"vflip",V4L2_CID_VFLIP}};
    if (argc==5) {
        char *end=nullptr; long value=strtol(argv[4],&end,10);
        auto it=ids.find(argv[3]);
        if (strcmp(argv[2],"--set") || it==ids.end() || !end || *end) { close(fd); return 2; }
        v4l2_queryctrl range={}; range.id=it->second;
        if (ioctl(fd,VIDIOC_QUERYCTRL,&range)<0 || value<range.minimum || value>range.maximum) {
            fprintf(stderr,"Unsupported control or value out of range\n"); close(fd); return 2;
        }
        v4l2_control control={}; control.id=it->second; control.value=value;
        if (ioctl(fd,VIDIOC_S_CTRL,&control)<0) { perror("Set sensor control"); close(fd); return 1; }
    }
    for (const auto &[name,id]:ids) {
        v4l2_queryctrl query={}; query.id=id;
        v4l2_control control={}; control.id=id;
        if (ioctl(fd,VIDIOC_QUERYCTRL,&query)==0 && ioctl(fd,VIDIOC_G_CTRL,&control)==0)
            printf("%s=%d min=%d max=%d default=%d\n",name.c_str(),control.value,query.minimum,query.maximum,query.default_value);
    }
    close(fd); return 0;
}
