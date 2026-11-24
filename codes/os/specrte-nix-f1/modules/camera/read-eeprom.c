/* Read only: stock IMX363 EEPROM, 8-bit address 0xa8 / Linux 7-bit 0x54.
 * The two-byte write message selects a read address; no EEPROM data is written.
 * Default 256-byte prefix; --full reads the stock descriptor's 8192-byte map.
 * Never probes other addresses or writes EEPROM contents.
 */
#include <errno.h>
#include <fcntl.h>
#include <linux/i2c-dev.h>
#include <linux/i2c.h>
#include <stdio.h>
#include <string.h>
#include <sys/ioctl.h>
#include <unistd.h>
int main(int argc, char **argv) {
    if ((argc != 2 && argc != 3) || (argc == 3 && strcmp(argv[2], "--full"))) {
        fprintf(stderr,"Usage: %s /dev/i2c-CAMERA_BUS [--full] > eeprom.bin\n",argv[0]); return 2;
    }
    int fd=open(argv[1],O_RDWR|O_CLOEXEC);
    if(fd<0) {perror("open camera I2C");return 1;}
    unsigned char bytes[8192];
    const unsigned length = argc == 3 ? sizeof bytes : 256;
    for(unsigned offset=0;offset<length;offset+=8) {
        unsigned char address[2]={offset>>8,offset&255};
        struct i2c_msg messages[2]={{.addr=0x54,.flags=0,.len=2,.buf=address},
                                   {.addr=0x54,.flags=I2C_M_RD,.len=8,.buf=bytes+offset}};
        struct i2c_rdwr_ioctl_data request={.msgs=messages,.nmsgs=2};
        if(ioctl(fd,I2C_RDWR,&request)!=2) {perror("EEPROM read");close(fd);return 1;}
    }
    close(fd);
    return fwrite(bytes,1,length,stdout)==length ? 0 : 1;
}
