#include "raw10-shading.h"
#include <cassert>
#include <iostream>
#include <limits>
#include <chrono>
int main() {
    using namespace spectre;
    ShadingTable table;for (auto &c:table)c.fill(1);
    Raw10Shading shading;
    constexpr unsigned w=8,h=6,stride=12;
    std::vector<uint8_t> src(stride*h,0xa5),dst(src.size());
    auto fill=[&](unsigned value) {
        for (unsigned y=0;y<h;++y) for (unsigned x=0;x<w;x+=4) {
            auto *p=src.data()+y*stride+x/4*5;
            for (unsigned k=0;k<4;++k)p[k]=value>>2;
            p[4]=(value&3)*0x55;
        }
    };
    assert(shading.prepare(w,h,false,table,true));fill(123);
    assert(shading.apply(src.data(),src.size(),stride,dst.data(),dst.size()));assert(src==dst);
    table[0].fill(2);table[3].fill(0.5f);
    assert(shading.prepare(w,h,false,table,false));fill(128);
    assert(shading.apply(src.data(),src.size(),stride,dst.data(),dst.size()));
    assert(raw10(dst.data(),0)==192 && raw10(dst.data(),1)==128);
    assert(raw10(dst.data()+stride,0)==128 && raw10(dst.data()+stride,1)==96);
    assert(dst[10]==0xa5 && dst[11]==0xa5 && raw10(src.data(),0)==128);
    fill(64);assert(shading.apply(src.data(),src.size(),stride,dst.data(),dst.size()));assert(src==dst);
    fill(48);assert(shading.apply(src.data(),src.size(),stride,dst.data(),dst.size()));
    assert(raw10(dst.data(),0)==32 && raw10(dst.data()+stride,1)==56);
    fill(1023);assert(shading.apply(src.data(),src.size(),stride,dst.data(),dst.size()));assert(raw10(dst.data(),0)==1023);
    // Colour-only mode removes spatial colour changes without a global WB shift.
    assert(shading.prepare(w,h,false,table,true));fill(128);
    assert(shading.apply(src.data(),src.size(),stride,dst.data(),dst.size()));assert(src==dst);
    for (unsigned y=0;y<13;++y)for (unsigned x=0;x<17;++x)table[0][y*17+x]=1+x/32.0f;
    assert(shading.prepare(w,h,false,table,true));
    assert(shading.apply(src.data(),src.size(),stride,dst.data(),dst.size()));
    assert(raw10(dst.data(),0)==115); // (128-64) * (1 / 1.25) + 64
    assert(raw10(dst.data(),6)==137); // x=6/7 -> gain=(1+3/7)/1.25
    assert(raw10(dst.data(),1)==128);
    assert(!shading.apply(src.data(),src.size()-1,stride,dst.data(),dst.size()));
    assert(!shading.apply(src.data(),src.size(),stride,dst.data(),dst.size()-1));
    assert(!shading.apply(src.data(),src.size(),stride,src.data(),src.size()));
    assert(!shading.prepare(1000,750,true,table,true));
    assert(shading.prepare(1008,756,true,sunnyShading,true));
    assert(shading.prepare(4032,3024,false,sunnyShading,false));
    // Byte-exact accelerated/scalar parity, including packed tails, row
    // padding, all code values, clipping and signed below-black samples.
    for (unsigned width : {16u,20u,32u,1008u,4032u}) for (bool colour : {false,true}) {
        constexpr unsigned height=18;
        unsigned pitch=width*5/4+7;
        std::vector<uint8_t> input(size_t(pitch)*height),scalar(input.size()),fast(input.size());
        uint32_t seed=width;
        for (auto &v:input) { seed=seed*1664525u+1013904223u;v=seed>>24; }
        assert(shading.prepare(width,height,false,sunnyShading,colour));
        assert(shading.apply(input.data(),input.size(),pitch,scalar.data(),scalar.size(),false));
        assert(shading.apply(input.data(),input.size(),pitch,fast.data(),fast.size()));
        assert(scalar==fast);
    }
    // Warm processing timing (not allocation/configuration) on the same data.
    {
        constexpr unsigned width=1008,height=756,pitch=width*5/4;
        std::vector<uint8_t> input(pitch*height,0x91),output(input.size());
        assert(shading.prepare(width,height,true,sunnyShading,true));
        for (bool fast : {false,true}) {
            auto start=std::chrono::steady_clock::now();
            for (unsigned n=0;n<30;++n)
                assert(shading.apply(input.data(),input.size(),pitch,output.data(),output.size(),fast));
            auto elapsed=std::chrono::duration<double,std::milli>(std::chrono::steady_clock::now()-start).count()/30;
            std::cout<<(fast ? "Vector" : "Scalar")<<" shading ms/frame: "<<elapsed<<"\n";
        }
    }
    table[2][100]=std::numeric_limits<float>::quiet_NaN();assert(!shading.prepare(w,h,false,table,true));
    assert(!shading.apply(src.data(),src.size(),stride,dst.data(),dst.size()));
    std::cout<<"RAW shading: pedestal, CFA, spatial colour, saturation, padding, immutable input and bounds passed\n";
}
