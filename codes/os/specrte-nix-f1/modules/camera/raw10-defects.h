/* SPDX-License-Identifier: MIT */
#pragma once
#include "raw10-reduce.h"
#include "defect-profile.h"
#include <algorithm>
#include <array>
#include <cstring>
namespace spectre {
// Replace only calibrated coordinates, using immutable same-colour neighbours.
// Never infer a defect from ordinary scene brightness; preserve packed padding.
inline bool correctRaw10Defects(const uint8_t *src, size_t length, unsigned stride,
                               uint8_t *dst, unsigned width, unsigned height,
                               const DefectPoint *points, size_t count)
{
    if (!src || !dst || src==dst || !width || !height || width%4 ||
        stride<size_t(width)*5/4 || length<size_t(stride)*height || (!points && count))
        return false;
    for (size_t i=0;i<count;++i)
        if (points[i].x<2 || unsigned(points[i].x)+2>=width ||
            points[i].y<2 || unsigned(points[i].y)+2>=height) return false;
    std::memcpy(dst,src,size_t(stride)*height);
    for (size_t i=0;i<count;++i) {
        const int x=points[i].x,y=points[i].y;
        std::array<unsigned,8> samples; unsigned n=0;
        for (int dy=-2;dy<=2;dy+=2)
            for (int dx=-2;dx<=2;dx+=2) {
                if (!dx && !dy) continue;
                samples[n++]=raw10(src+size_t(y+dy)*stride,x+dx);
            }
        std::sort(samples.begin(),samples.end());
        const unsigned value=(samples[3]+samples[4]+1)/2;
        auto *group=dst+size_t(y)*stride+(x/4)*5;
        group[x%4]=value>>2;
        unsigned shift=2*(x%4);
        group[4]=(group[4]&~(3u<<shift))|((value&3)<<shift);
    }
    return true;
}
}
