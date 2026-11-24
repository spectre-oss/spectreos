// SPDX-License-Identifier: MIT
#include "raw10-denoise.h"
#include <cassert>
#include <iostream>
#include <random>
int main() {
    constexpr unsigned w=128,h=288,stride=w*5/4+8;
    std::vector<uint8_t> src(stride*h,0xA5),dst(stride*h+32,0xCC);
    std::mt19937 rng(42);std::normal_distribution<float> noise(0,5);
    auto write=[](uint8_t *row,unsigned x,unsigned v) {
        auto *p=row+x/4*5;p[x%4]=v>>2;p[4]=(p[4]&~(3u<<(2*(x%4))))|((v&3)<<(2*(x%4)));
    };
    for(unsigned y=0;y<h;++y)for(unsigned x=0;x<w;++x) {
        unsigned c=y%2*2+x%2;
        int baseline=(x<64?160:640)+c*20;
        write(src.data()+stride*y,x,std::clamp(int(baseline+noise(rng)),0,1023));
    }
    auto copy=src;
    assert(!spectre::denoiseRaw10(src.data(),src.size()-1,stride,dst.data(),w,h,8));
    assert(!spectre::denoiseRaw10(src.data(),src.size(),stride,src.data(),w,h,8));
    assert(spectre::denoiseRaw10(src.data(),src.size(),stride,dst.data(),w,h,8));
    assert(src==copy);
    double before=0,after=0;int n=0;
    for(unsigned y=4;y<h-4;++y)for(unsigned x=4;x<60;++x) {
        int truth=160+(y%2*2+x%2)*20;
        int a=int(spectre::raw10(src.data()+y*stride,x))-truth;
        int b=int(spectre::raw10(dst.data()+y*stride,x))-truth;
        before+=a*a;after+=b*b;++n;
    }
    assert(after<before*0.5);
    for(unsigned y=0;y<h;++y) {
        for(unsigned x=w*5/4;x<stride;++x)assert(dst[y*stride+x]==0xA5);
        assert(spectre::raw10(dst.data()+y*stride,63)<250);
        assert(spectre::raw10(dst.data()+y*stride,64)>600);
    }
    for(unsigned i=stride*h;i<dst.size();++i)assert(dst[i]==0xCC);
    for(unsigned c=0;c<4;++c) {
        assert(spectre::rawNoiseVariance(100,16,c)>spectre::rawNoiseVariance(100,4,c));
        assert(std::isfinite(spectre::rawNoiseVariance(100,NAN,c)));
    }
    std::cout<<"RAW noise variance ratio "<<after/before<<"; CFA edges, padding, bounds and gain model PASS\n";
}
