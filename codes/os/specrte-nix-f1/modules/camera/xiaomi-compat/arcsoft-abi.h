/* Private ABI recovered from this ROM's HDR node and engine, not a public SDK.
 * See docs/camera-xiaomi-compat.md for binary hashes and REA evidence.
 * This header is valid only for the pinned ARM32 libraries. */
#ifndef SPECTRE_ARCSOFT_ABI_H
#define SPECTRE_ARCSOFT_ABI_H
#include <stddef.h>
#include <stdint.h>

typedef struct {
    uint32_t format;
    int32_t width, height;
    uint8_t *plane[4];
    int32_t pitch[4];
} ArcImage;

typedef struct {
    int32_t count;
    ArcImage image[3];
} ArcInputs;

typedef struct {
    int32_t tone_length, brightness, saturation, contrast;
} ArcParams;

typedef struct { float word[6]; } ArcAE;

_Static_assert(sizeof(void *) == 4, "Original camera ABI requires ARM32");
_Static_assert(sizeof(ArcImage) == 44, "ASVL_OFFSCREEN size");
_Static_assert(offsetof(ArcImage, plane) == 12, "ASVL plane offset");
_Static_assert(offsetof(ArcImage, pitch) == 28, "ASVL pitch offset");
_Static_assert(sizeof(ArcInputs) == 136, "ARC HDR input size");
_Static_assert(sizeof(ArcParams) == 16, "ARC HDR parameters");
_Static_assert(sizeof(ArcAE) == 24, "HDR node copies first 24 AE bytes");
#endif
