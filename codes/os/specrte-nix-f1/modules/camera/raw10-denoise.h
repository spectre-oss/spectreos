/* SPDX-License-Identifier: MIT */
#pragma once
#include "raw10-reduce.h"
#include "noise-profile.h"
#include <array>
#include <cstring>
#include <future>
#include <vector>

namespace spectre {
// Same-CFA bilateral filter in linear RAW10, before demosaic/WB/CCM/gamma.
// Input is immutable. Borders and row padding are preserved byte-for-byte.
inline bool denoiseRaw10(const uint8_t *src, size_t length, unsigned stride,
                         uint8_t *dst, unsigned width, unsigned height, float gain)
{
    if (!src || !dst || src == dst || width < 8 || height < 8 || width % 4 ||
        stride < size_t(width)*5/4 || length < size_t(stride)*height)
        return false;
    std::array<std::array<int,1024>,4> thresholds;
    for (unsigned c=0;c<4;++c)
        for (int v=0;v<1024;++v)
            thresholds[c][v] = std::max(3, int(3.0f*std::sqrt(2*rawNoiseVariance(v-64,gain,c))));
    std::vector<uint16_t> unpacked(size_t(width)*height);
    for (unsigned y=0;y<height;++y)
        for (unsigned x=0;x<width;++x)
            unpacked[size_t(y)*width+x]=raw10(src+size_t(y)*stride,x);
    std::memcpy(dst,src,size_t(stride)*height);
    auto rows = [&](unsigned begin, unsigned end) {
    for (unsigned y=begin;y<end;++y) {
        auto *row=dst+size_t(y)*stride;
        for (unsigned x=2;x+2<width;++x) {
            int center=unpacked[size_t(y)*width+x];
            int threshold=thresholds[(y%2)*2+x%2][center];
            int sum=center*8;
            for (int dy=-2;dy<=2;dy+=2)
                for (int dx=-2;dx<=2;dx+=2) {
                    if (!dx && !dy) continue;
                    int value=unpacked[size_t(int(y)+dy)*width+int(x)+dx];
                    // Reject cross-edge samples; fixed normalization avoids a
                    // costly per-pixel integer division on ARM.
                    sum+=std::abs(value-center)<threshold ? value : center;
                }
            unsigned value=(sum+8)/16;
            auto *group=row+(x/4)*5;
            group[x%4]=value>>2;
            unsigned shift=2*(x%4);
            group[4]=(group[4]&~(3u<<shift))|((value&3)<<shift);
        }
    }
    };
    // Independent output rows allow bounded parallelism during still capture.
    // The immutable unpacked input is shared; no two workers write one row.
    if (height < 256) rows(2,height-2);
    else {
        std::vector<std::future<void>> jobs;
        for (unsigned i=0;i<4;++i) {
            unsigned begin=2+(height-4)*i/4,end=2+(height-4)*(i+1)/4;
            jobs.push_back(std::async(std::launch::async,rows,begin,end));
        }
        for (auto &job:jobs) job.get();
    }
    return true;
}
}
