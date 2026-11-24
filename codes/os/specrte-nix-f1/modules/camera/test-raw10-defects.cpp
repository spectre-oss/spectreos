// SPDX-License-Identifier: MIT
#include "raw10-defects.h"
#include <cassert>
#include <iostream>
#include <vector>
static void put(uint8_t *row,unsigned x,unsigned v) {
    auto *p=row+x/4*5;p[x%4]=v>>2;
    p[4]=(p[4]&~(3u<<(2*(x%4))))|((v&3)<<(2*(x%4)));
}
int main() {
    constexpr unsigned w=32,h=24,stride=w*5/4+7;
    std::vector<uint8_t> src(stride*h,0xA5),dst(stride*h+32,0xCC);
    for(unsigned y=0;y<h;++y)for(unsigned x=0;x<w;++x)
        put(src.data()+y*stride,x,100+100*(y%2*2+x%2));
    const spectre::DefectPoint points[]={{8,8},{9,8},{8,9},{9,9}};
    for(auto p:points)put(src.data()+p.y*stride,p.x,1023);
    put(src.data()+16*stride,16,1023); // A real bright scene point, not in the map.
    auto original=src;
    assert(spectre::correctRaw10Defects(src.data(),src.size(),stride,dst.data(),w,h,points,4));
    assert(src==original);
    for(auto p:points)assert(spectre::raw10(dst.data()+p.y*stride,p.x)==100+100*(p.y%2*2+p.x%2));
    assert(spectre::raw10(dst.data()+16*stride,16)==1023);
    for(unsigned y=0;y<h;++y)for(unsigned x=w*5/4;x<stride;++x)assert(dst[y*stride+x]==0xA5);
    for(unsigned i=stride*h;i<dst.size();++i)assert(dst[i]==0xCC);
    const spectre::DefectPoint bad[]={{0,0}};
    auto previous=dst;
    assert(!spectre::correctRaw10Defects(src.data(),src.size(),stride,dst.data(),w,h,bad,1));
    assert(dst==previous);
    assert(!spectre::correctRaw10Defects(src.data(),src.size()-1,stride,dst.data(),w,h,points,4));
    assert(!spectre::correctRaw10Defects(src.data(),src.size(),stride,src.data(),w,h,points,4));
    assert(!spectre::correctRaw10Defects(src.data(),src.size(),stride,dst.data(),w,h,nullptr,4));
    assert(spectre::correctRaw10Defects(src.data(),src.size(),stride,dst.data(),w,h,nullptr,0));
    assert(std::equal(src.begin(),src.end(),dst.begin()));
    for(auto p:spectre::sunnyDefects)assert(p.x>=2 && p.x<4030 && p.y>=2 && p.y<3022);
    std::cout<<"Measured RAW defect map: CFA separation, unmapped highlights, padding, bounds and immutable input PASS\n";
}
