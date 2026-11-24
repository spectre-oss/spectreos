#define _POSIX_C_SOURCE 200809L
#include "arcsoft-abi.h"
#include <dlfcn.h>
#include <errno.h>
#include <fcntl.h>
#include <math.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/resource.h>
#include <sys/stat.h>
#include <time.h>
#include <unistd.h>

/* One invocation, one job. Never loaded into the Qt/libcamera process. */
static int fail(const char *stage, int code)
{
    fprintf(stdout, "{\"ok\":false,\"stage\":\"%s\",\"code\":%d}\n", stage, code);
    return 1;
}

static void *symbol(void *lib, const char *name)
{
    void *p = dlsym(lib, name);
    if (!p) {
        fprintf(stderr, "Missing symbol %s: %s\n", name, dlerror());
        exit(fail("symbol", 1));
    }
    return p;
}

static long number(const char *s, long low, long high)
{
    char *end;
    errno = 0;
    long n = strtol(s, &end, 10);
    if (errno || !*s || *end || n < low || n > high)
        exit(fail("integer-argument", 1));
    return n;
}

static float real_number(const char *s)
{
    char *end;
    errno = 0;
    float n = strtof(s, &end);
    if (errno || !*s || *end || !isfinite(n) || n < 0 || n > 100000)
        exit(fail("ae-argument", 1));
    return n;
}

static int image_alloc(ArcImage *im, int width, int height, uint32_t format)
{
    const int stride = (width + 63) & ~63;
    size_t bytes = (size_t)stride * height * 3 / 2;
    *im = (ArcImage){.format = format, .width = width, .height = height};
    if (posix_memalign((void **)&im->plane[0], 64, bytes))
        return -1;
    memset(im->plane[0], 0, bytes);
    im->plane[1] = im->plane[0] + (size_t)stride * height;
    im->pitch[0] = im->pitch[1] = stride;
    return 0;
}

static int image_read(ArcImage *im, const char *path)
{
    int fd = open(path, O_RDONLY | O_CLOEXEC | O_NOFOLLOW);
    if (fd < 0) return -1;
    struct stat st;
    if (fstat(fd, &st) || !S_ISREG(st.st_mode) ||
        st.st_size != (off_t)im->width * im->height * 3 / 2) {
        close(fd);
        return -1;
    }
    for (int p = 0; p < 2; ++p) {
        int rows = p ? im->height / 2 : im->height;
        for (int y = 0; y < rows; ++y) {
            size_t done = 0;
            while (done < (size_t)im->width) {
                ssize_t n = read(fd, im->plane[p] + (size_t)y * im->pitch[p] + done,
                                 im->width - done);
                if (n < 0 && errno == EINTR) continue;
                if (n <= 0) { close(fd); return -1; }
                done += n;
            }
        }
    }
    close(fd);
    return 0;
}

static int image_write(const ArcImage *im, const char *path)
{
    int fd = open(path, O_WRONLY | O_CREAT | O_EXCL | O_CLOEXEC | O_NOFOLLOW, 0600);
    if (fd < 0) return -1;
    for (int p = 0; p < 2; ++p) {
        int rows = p ? im->height / 2 : im->height;
        for (int y = 0; y < rows; ++y) {
            size_t done = 0;
            while (done < (size_t)im->width) {
                ssize_t n = write(fd, im->plane[p] + (size_t)y * im->pitch[p] + done,
                                  im->width - done);
                if (n < 0 && errno == EINTR) continue;
                if (n <= 0) { close(fd); unlink(path); return -1; }
                done += n;
            }
        }
    }
    int rc = fsync(fd);
    if (close(fd)) rc = -1;
    if (rc) unlink(path);
    return rc;
}

static double milliseconds(void)
{
    struct timespec t;
    clock_gettime(CLOCK_MONOTONIC, &t);
    return t.tv_sec * 1000.0 + t.tv_nsec / 1000000.0;
}

int main(int argc, char **argv)
{
    /* private CLI: probe LIB; hdr LIB FORMAT W H IN0 IN1 IN2 OUT AE0..5 */
    int probe = argc == 3 && !strcmp(argv[1], "probe");
    if (!probe && (argc != 16 || strcmp(argv[1], "hdr")))
        return fail("usage", 1);
    struct rlimit no_core = {0, 0}, memory = {1024UL * 1024 * 1024, 1024UL * 1024 * 1024};
    if (setrlimit(RLIMIT_CORE, &no_core) || setrlimit(RLIMIT_AS, &memory))
        return fail("resource-limits", errno);
    alarm(90);
    ArcInputs inputs = {.count = 3};
    ArcAE ae = {0};
    if (!probe) {
        int w = number(argv[4], 64, 4032), h = number(argv[5], 64, 4032);
        if ((w & 1) || (h & 1) || (size_t)w * h > 4032UL * 3024)
            return fail("dimensions", 1);
        uint32_t format = !strcmp(argv[3], "nv12") ? 0x801 :
                          !strcmp(argv[3], "nv21") ? 0x802 : 0;
        if (!format) return fail("pixel-format", 1);
        for (int i = 0; i < 6; ++i) ae.word[i] = real_number(argv[10 + i]);
        for (int i = 0; i < 3; ++i) {
            if (image_alloc(&inputs.image[i], w, h, format)) return fail("allocate", 1);
            if (image_read(&inputs.image[i], argv[6 + i])) return fail("input", i);
        }
    }
    void *lib = dlopen(argv[2], RTLD_NOW | RTLD_LOCAL);
    if (!lib) { fprintf(stderr, "%s\n", dlerror()); return fail("load", 1); }
    int (*init)(void **, int, int) = symbol(lib, "ARC_HDR_Init");
    int (*uninit)(void **) = symbol(lib, "ARC_HDR_Uninit");
    int (*defaults)(ArcParams *) = symbol(lib, "ARC_HDR_GetDefaultParam");
    int (*preprocess)(void *, ArcInputs *, ArcAE *, int) = symbol(lib, "ARC_HDR_PreProcess");
    int (*process)(void *, ArcParams *, ArcImage *) = symbol(lib, "ARC_HDR_Process");
    void *handle = NULL;
    ArcParams params = {0};
    double start = milliseconds();
    int rc = init(&handle, 0, 0); /* rear camera, as in the original CHI node */
    if (rc || !handle) return fail("init", rc ? rc : -1);
    rc = defaults(&params);
    if (rc) { uninit(&handle); return fail("defaults", rc); }
    if (probe) {
        rc = uninit(&handle);
        if (rc) return fail("uninit", rc);
        printf("{\"ok\":true,\"backend\":\"xiaomi-arcsoft-hdr\",\"abi\":\"arm32\","
               "\"default_parameters\":[%d,%d,%d,%d],\"elapsed_ms\":%.3f}\n",
               params.tone_length, params.brightness, params.saturation, params.contrast,
               milliseconds() - start);
    } else {
        for (int i = 0; i < 3; ++i) {
            rc = preprocess(handle, &inputs, &ae, i);
            if (rc) { uninit(&handle); return fail("preprocess", rc); }
        }
        /* Xiaomi writes the result into its final input frame. */
        rc = process(handle, &params, &inputs.image[2]);
        int final_rc = uninit(&handle);
        /* Xiaomi also accepts 0x1a. Surface it, never silently report success. */
        if (rc) return fail("process", rc);
        if (final_rc) return fail("uninit", final_rc);
        if (image_write(&inputs.image[2], argv[9])) return fail("output", errno);
        printf("{\"ok\":true,\"backend\":\"xiaomi-arcsoft-hdr\",\"frames\":3,"
               "\"width\":%d,\"height\":%d,\"elapsed_ms\":%.3f}\n",
               inputs.image[2].width, inputs.image[2].height, milliseconds() - start);
    }
    for (int i = 0; i < 3; ++i) free(inputs.image[i].plane[0]);
    dlclose(lib);
    return 0;
}
