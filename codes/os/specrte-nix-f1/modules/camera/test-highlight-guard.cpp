#include "highlight-guard.h"
#include <array>
#include <cassert>
#include <iostream>
using libcamera::ipa::soft::algorithms::highlightGuard;
int main() {
    std::array<unsigned,256> h{};
    assert(highlightGuard(h,16,.95,.8,1.5)==1.5); // empty input
    h[60]=960; h[255]=40; // ignore a small lamp/specular area
    assert(highlightGuard(h,16,.95,.8,1.5)==1.5);
    h={}; h[60]=940; h[255]=60; // broad clipping must reduce exposure
    assert(highlightGuard(h,16,.95,.8,1.5)>2.7);
    assert(highlightGuard(h,16,0,.8,1.5)==1.5); // other sensors unchanged
    h={}; h[208]=1000; // near target: hold, avoiding oscillation
    assert(highlightGuard(h,16,.95,.8,1.5)==2.5);
    assert(highlightGuard(h,16,.95,.8,3.5)==3.5); // existing decrease wins
    h={}; h[0]=1000;
    assert(highlightGuard(h,16,.95,.8,1.5)==1.5); // black frame may brighten
    h={}; h[255]=1000;
    assert(highlightGuard(h,16,.95,.8,1.5)>2.7);
    std::cout << "PASS: highlight headroom, specular rejection, hysteresis, empty/dark/saturated inputs\n";
}
