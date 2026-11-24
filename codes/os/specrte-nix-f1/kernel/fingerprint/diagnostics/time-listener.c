/* SPDX-License-Identifier: GPL-2.0-only */
/* Diagnostic legacy QSEE time listener. No storage or clock-setting operations.
 * ABI: Qualcomm minkipc 5e3c6951d13162a262ea65ea8e8351228f092b58,
 * listeners/libtimeservice/time_msg.h; checked against MIUI V12.0.3.0
 * lib64/libdrmtime.so (BuildID dada25d6249ac2aa0d82bfd463b53956).
 * Stock UTC/time-ms use time_genoff with CLOCK_REALTIME fallback; this
 * diagnostic implements that fallback only. It is not a trusted clock.
 */
#define _GNU_SOURCE
#include <errno.h>
#include <fcntl.h>
#include <linux/tee.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>
#include <sys/ioctl.h>
#include <sys/mman.h>
#include <time.h>
#include <unistd.h>
#define TIME_ID 11
#define TIME_BUFSZ 20480
static int dispatch_time(unsigned char *buf, size_t len)
{
    uint32_t cmd;
    unsigned char reply[44] = {0};
    struct timespec now = {0};
    int32_t status = -1;
    size_t size;
    if (len < 4) return -1;
    memcpy(&cmd, buf, 4);
    size = cmd == 0x303 ? 44 : cmd == 0x304 ? 12 :
           (cmd == 0x302 || cmd == 0x305) ? 16 : 8;
    if (len < size) return -1;
    /* MIUI uses a separate zeroed reply; its first word remains zero. */
    if (cmd == 0x302 || cmd == 0x305) {
        if (!clock_gettime(CLOCK_REALTIME, &now) && now.tv_sec >= 0 &&
            (uint64_t)now.tv_sec <= UINT32_MAX) {
            uint32_t sec = now.tv_sec, ns = cmd == 0x302 ? 0 : now.tv_nsec;
            memcpy(reply + 4, &sec, 4);
            memcpy(reply + 8, &ns, 4);
            status = 0;
        }
        memcpy(reply + 12, &status, 4);
    } else if (cmd == 0x303) {
        struct tm utc;
        if (!clock_gettime(CLOCK_REALTIME, &now) && gmtime_r(&now.tv_sec, &utc)) {
            int32_t fields[9] = {utc.tm_sec, utc.tm_min, utc.tm_hour,
                utc.tm_mday, utc.tm_mon, utc.tm_year, utc.tm_wday,
                utc.tm_yday, utc.tm_isdst};
            memcpy(reply + 4, fields, sizeof(fields));
            status = 0;
        }
        memcpy(reply + 40, &status, 4);
    } else if (cmd == 0x304) {
        uint64_t ms = UINT64_MAX;
        /* MIUI returns whole UTC seconds times 1000, not monotonic time. */
        if (!clock_gettime(CLOCK_REALTIME, &now) && now.tv_sec >= 0 &&
            (uint64_t)now.tv_sec <= UINT32_MAX) ms = (uint64_t)now.tv_sec * 1000;
        memcpy(reply + 4, &ms, 8);
    } else {
        if (cmd == 0x306) status = 0;
        memcpy(reply + 4, &status, 4);
    }
    memcpy(buf, reply, size);
    return 0;
}
#ifndef TIME_LISTENER_TEST
int main(void)
{
    int result = 1;
    setvbuf(stdout, NULL, _IONBF, 0);
    int fd = open("/dev/teepriv0", O_RDWR | O_CLOEXEC);
    if (fd < 0) { perror("teepriv0"); return 1; }
    struct tee_ioctl_shm_alloc_data alloc = {.size = TIME_BUFSZ};
    int shm = ioctl(fd, TEE_IOC_SHM_ALLOC, &alloc);
    if (shm < 0) { perror("shm"); close(fd); return 1; }
    unsigned char *buf = mmap(NULL, TIME_BUFSZ, PROT_READ | PROT_WRITE, MAP_SHARED, shm, 0);
    if (buf == MAP_FAILED) { perror("mmap"); close(shm); close(fd); return 1; }
    struct {struct tee_ioctl_open_session_arg a; struct tee_ioctl_param p[2];} s = {0};
    s.a.num_params = 2;
    s.p[0] = (struct tee_ioctl_param){.attr = TEE_IOCTL_PARAM_ATTR_TYPE_VALUE_INPUT, .a = TIME_ID};
    s.p[1] = (struct tee_ioctl_param){.attr = TEE_IOCTL_PARAM_ATTR_TYPE_MEMREF_INOUT, .b = TIME_BUFSZ, .c = alloc.id};
    struct tee_ioctl_buf_data bd = {.buf_ptr = (uintptr_t)&s, .buf_len = sizeof(s)};
    if (ioctl(fd, TEE_IOC_OPEN_SESSION, &bd)) { perror("register"); goto out; }
    if (s.a.ret) { fprintf(stderr, "register status=0x%x\n", s.a.ret); goto out; }
    puts("Read-only time listener 11 registered");
    for (;;) {
        struct {struct tee_iocl_supp_recv_arg a; struct tee_ioctl_param p[2];} r = {0};
        r.a.num_params = 2;
        bd = (struct tee_ioctl_buf_data){.buf_ptr = (uintptr_t)&r, .buf_len = sizeof(r)};
        if (ioctl(fd, TEE_IOC_SUPPL_RECV, &bd)) { perror("recv"); break; }
        uint32_t cmd;
        memcpy(&cmd, buf, 4);
        int ok = r.a.func == TIME_ID && !dispatch_time(buf, TIME_BUFSZ);
        printf("Time request cmd=0x%x transport_ok=%d\n", cmd, ok);
        struct {struct tee_iocl_supp_send_arg a; struct tee_ioctl_param p;} t = {0};
        t.a.num_params = 1; t.a.ret = ok ? 0 : 1;
        t.p = (struct tee_ioctl_param){.attr = TEE_IOCTL_PARAM_ATTR_TYPE_VALUE_OUTPUT, .a = r.p[0].a};
        bd = (struct tee_ioctl_buf_data){.buf_ptr = (uintptr_t)&t, .buf_len = sizeof(t)};
        if (ioctl(fd, TEE_IOC_SUPPL_SEND, &bd)) { perror("send"); break; }
    }
    struct tee_ioctl_close_session_arg close_arg = {.session = s.a.session};
    if (ioctl(fd, TEE_IOC_CLOSE_SESSION, &close_arg)) perror("close session");
out:
    close(fd); munmap(buf, TIME_BUFSZ); close(shm);
    return result;
}
#endif
