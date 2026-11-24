#define _GNU_SOURCE
#include <time.h>
static int fail_clock, fail_utc;
static int test_clock_gettime(clockid_t id, struct timespec *ts)
{
    return fail_clock ? -1 : clock_gettime(id, ts);
}
static struct tm *test_gmtime_r(const time_t *t, struct tm *tm)
{
    return fail_utc ? NULL : gmtime_r(t, tm);
}
#define clock_gettime test_clock_gettime
#define gmtime_r test_gmtime_r
#define TIME_LISTENER_TEST
#include "time-listener.c"
#include <assert.h>
int main(void)
{
    for (uint32_t cmd = 0x300; cmd <= 0x307; cmd++) {
        size_t size = cmd == 0x303 ? 44 : cmd == 0x304 ? 12 :
                      (cmd == 0x302 || cmd == 0x305) ? 16 : 8;
        for (size_t len = 0; len <= 64; len++) {
            unsigned char b[64], before[64];
            memset(b, 0xa5, sizeof(b)); memcpy(b, &cmd, 4);
            memcpy(before, b, sizeof(b));
            assert(dispatch_time(b, len) == (len < size ? -1 : 0));
            assert(!memcmp(b + (len < size ? 0 : size),
                           before + (len < size ? 0 : size),
                           sizeof(b) - (len < size ? 0 : size)));
            if (len >= size) {
                uint32_t actual; memcpy(&actual, b, 4); assert(actual == 0);
                if (cmd == 0x300 || cmd == 0x301 || cmd == 0x307) {
                    int32_t status; memcpy(&status, b + 4, 4); assert(status == -1);
                }
            }
        }
    }
    for (uint32_t cmd = 0x302; cmd <= 0x305; cmd++) {
        unsigned char b[44] = {0};
        memcpy(b, &cmd, 4); fail_clock = 1;
        assert(dispatch_time(b, sizeof(b)) == 0);
        if (cmd == 0x304) {
            uint64_t ms; memcpy(&ms, b + 4, 8); assert(ms == UINT64_MAX);
        } else {
            int32_t status;
            memcpy(&status, b + (cmd == 0x303 ? 40 : 12), 4);
            assert(status == -1);
        }
    }
    fail_clock = 0; fail_utc = 1;
    unsigned char b[44] = {0};
    uint32_t cmd = 0x303; int32_t status;
    memcpy(b, &cmd, 4);
    assert(dispatch_time(b, sizeof(b)) == 0);
    memcpy(&status, b + 40, 4); assert(status == -1);
    puts("520 bounds cases and 5 time-provider failure cases passed");
    return 0;
}
