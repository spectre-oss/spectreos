// SPDX-License-Identifier: MIT
#pragma once
#include "raw10-reduce.h"
#include "shading-profile.h"
#include <algorithm>
#include <cmath>
#include <cstring>
#include <vector>
#if defined(__aarch64__)
#include <arm_neon.h>
#endif

namespace spectre {
class Raw10Shading {
    unsigned width_=0, height_=0;
    std::vector<uint16_t> gains_;
public:
    void clear() { width_=height_=0; gains_.clear(); }
    bool prepare(unsigned width,unsigned height,bool reduced,const ShadingTable &table,bool colourOnly) {
        clear();
        if (!width || !height || width%4 || height%2 || width>4032 || height>3024 ||
            (reduced && (width!=1008 || height!=756))) return false;
        for (const auto &channel:table) for (float v:channel)
            if (!std::isfinite(v) || v<0.25f || v>8.0f) return false;
        auto sample=[&](unsigned c,float x,float y) {
            unsigned ix=std::min(unsigned(x),15u),iy=std::min(unsigned(y),11u);
            float dx=x-ix,dy=y-iy;
            return (table[c][iy*17+ix]*(1-dx)+table[c][iy*17+ix+1]*dx)*(1-dy)+
                   (table[c][(iy+1)*17+ix]*(1-dx)+table[c][(iy+1)*17+ix+1]*dx)*dy;
        };
        float centreGreen=(table[1][110]+table[2][110])/2;
        width_=width;height_=height;gains_.resize(size_t(width)*height);
        for (unsigned y=0;y<height;++y) {
            // Reduced pixels average a 4x4 set of same-CFA samples: the
            // footprint centre is source coordinate 8*cell + phase + 3.
            float sy=reduced ? ((y/2)*8+(y%2)+3)*12.0f/3023 : y*12.0f/(height-1);
            for (unsigned x=0;x<width;++x) {
                float sx=reduced ? ((x/2)*8+(x%2)+3)*16.0f/4031 : x*16.0f/(width-1);
                unsigned c=(y%2)*2+x%2;
                float gain=sample(c,sx,sy);
                if (colourOnly) {
                    float green=(sample(1,sx,sy)+sample(2,sx,sy))/2;
                    gain=gain/green*centreGreen/table[c][110];
                }
                if (!std::isfinite(gain) || gain<0.25f || gain>8.0f) { clear(); return false; }
                gains_[size_t(y)*width+x]=uint16_t(std::lround(gain*4096));
            }
        }
        return true;
    }
    bool apply(const uint8_t *src,size_t length,unsigned stride,uint8_t *dst,size_t dstLength,
               bool vectorized=true) const {
        if (!src || !dst || src==dst || gains_.empty() || stride<size_t(width_)*5/4 ||
            length<size_t(stride)*height_ || dstLength<size_t(stride)*height_) return false;
        std::memcpy(dst,src,size_t(stride)*height_);
        for (unsigned y=0;y<height_;++y) {
            const auto *row=src+size_t(y)*stride;auto *out=dst+size_t(y)*stride;
            unsigned x=0;
#if defined(__aarch64__)
            if (vectorized) for (;x+16<=width_;x+=16) {
                // Two overlapping loads cover exactly 20 packed bytes. No
                // tail/padding overread, including the final sensor row.
                const uint8_t *p=row+x/4*5;uint8_t *q=out+x/4*5;
                const uint8x16x2_t packed={{vld1q_u8(p),vld1q_u8(p+4)}};
                static constexpr uint8_t hiIndex[16]={0,1,2,3,5,6,7,8,10,11,12,13,15,28,29,30};
                static constexpr uint8_t loIndex[16]={4,4,4,4,9,9,9,9,14,14,14,14,31,31,31,31};
                static constexpr int8_t shifts[16]={0,-2,-4,-6,0,-2,-4,-6,0,-2,-4,-6,0,-2,-4,-6};
                const auto hi=vqtbl2q_u8(packed,vld1q_u8(hiIndex));
                const auto lo=vandq_u8(vshlq_u8(vqtbl2q_u8(packed,vld1q_u8(loIndex)),
                                               vld1q_s8(shifts)),vdupq_n_u8(3));
                auto correct=[](uint16x8_t value,uint16x8_t gain) {
                    const auto signal=vsubq_s16(vreinterpretq_s16_u16(value),vdupq_n_s16(64));
                    const auto magnitude=vreinterpretq_u16_s16(vabsq_s16(signal));
                    const auto lower=vmull_u16(vget_low_u16(magnitude),vget_low_u16(gain));
                    const auto upper=vmull_high_u16(magnitude,gain);
                    const auto scaled=vreinterpretq_s16_u16(vcombine_u16(
                        vshrn_n_u32(vaddq_u32(lower,vdupq_n_u32(2048)),12),
                        vshrn_n_u32(vaddq_u32(upper,vdupq_n_u32(2048)),12)));
                    const auto signedValue=vbslq_s16(vcltq_s16(signal,vdupq_n_s16(0)),vnegq_s16(scaled),scaled);
                    return vreinterpretq_u16_s16(vminq_s16(vdupq_n_s16(1023),vmaxq_s16(vdupq_n_s16(0),
                        vaddq_s16(signedValue,vdupq_n_s16(64)))));
                };
                const auto a=correct(vorrq_u16(vshll_n_u8(vget_low_u8(hi),2),vmovl_u8(vget_low_u8(lo))),
                                     vld1q_u16(gains_.data()+size_t(y)*width_+x));
                const auto b=correct(vorrq_u16(vshll_high_n_u8(hi,2),vmovl_high_u8(lo)),
                                     vld1q_u16(gains_.data()+size_t(y)*width_+x+8));
                const auto high=vcombine_u8(vshrn_n_u16(a,2),vshrn_n_u16(b,2));
                const auto low=vandq_u8(vcombine_u8(vmovn_u16(a),vmovn_u16(b)),vdupq_n_u8(3));
                const auto shifted=vshlq_u8(low,vnegq_s8(vld1q_s8(shifts)));
                const auto pairs=vpadd_u8(vget_low_u8(shifted),vget_high_u8(shifted));
                const auto groups=vpadd_u8(pairs,pairs);
                const uint8x16x2_t result={{high,vcombine_u8(groups,groups)}};
                static constexpr uint8_t outputIndex[16]={0,1,2,3,16,4,5,6,7,17,8,9,10,11,18,12};
                vst1q_u8(q,vqtbl2q_u8(result,vld1q_u8(outputIndex)));
                q[16]=vgetq_lane_u8(high,13);q[17]=vgetq_lane_u8(high,14);
                q[18]=vgetq_lane_u8(high,15);q[19]=vget_lane_u8(groups,3);
            }
#else
            (void)vectorized;
#endif
            for (;x<width_;x+=4) {
                unsigned low=0;
                for (unsigned k=0;k<4;++k) {
                    int signal=int(raw10(row,x+k))-64;
                    int magnitude=(std::abs(signal)*gains_[size_t(y)*width_+x+k]+2048)/4096;
                    unsigned value=std::clamp(64+(signal<0 ? -magnitude : magnitude),0,1023);
                    out[x/4*5+k]=value>>2;low|=(value&3)<<(2*k);
                }
                out[x/4*5+4]=low;
            }
        }
        return true;
    }
};
}
