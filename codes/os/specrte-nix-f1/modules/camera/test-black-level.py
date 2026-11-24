#!/usr/bin/env python3
"""Compile and test the actual CPU ISP LUT methods. Run on the build server.

Usage: test-black-level.py LIBCAMERA_SOURCE [--unpatched]
Only camera-independent methods are extracted; RGB/Matrix types are upstream.
The unpatched run is expected to fail, demonstrating regression sensitivity.
"""
from pathlib import Path
import subprocess
import sys
import tempfile

source = Path(sys.argv[1]).resolve()
project = Path(__file__).resolve().parents[2]
with tempfile.TemporaryDirectory(prefix="spectre-blc-") as tmp:
    work = Path(tmp)
    cpp = source / "src/libcamera/software_isp/debayer_cpu.cpp"
    (work / "debayer_cpu.cpp").write_bytes(cpp.read_bytes())
    if "--unpatched" not in sys.argv:
        subprocess.run(["patch", str(work / "debayer_cpu.cpp"),
                        str(project / "patches/libcamera-softisp-black-level.patch")], check=True)
        if "--without-highlight-fix" not in sys.argv:
            subprocess.run(["patch", str(work / "debayer_cpu.cpp"),
                            str(project / "patches/libcamera-softisp-ccm-highlight.patch")], check=True)
    text = (work / "debayer_cpu.cpp").read_text()
    methods = text[text.index("void DebayerCpu::updateGammaTable("):text.index("void DebayerCpu::process(")]
    # Logging alone requires linking libcamera-base; omit it, not any LUT math.
    start = methods.index("\tLOG(Debayer, Debug)")
    end = methods.index("\n\tparams_ = params;", start)
    methods = methods[:start] + methods[end:]
    harness = r'''
#include <memory>
#include <cassert>
#include <iostream>
#include <libcamera/base/log.h>
#undef ASSERT
#define ASSERT(condition) assert(condition)
#include "libcamera/internal/software_isp/debayer_params.h"
using namespace libcamera;
struct DebayerCpu {
    static constexpr unsigned kRGBLookupSize=256;
    struct CcmColumn { int16_t r,g,b; };
    std::array<uint8_t,256> red_{},green_{},blue_{},gammaLut_{};
    std::array<CcmColumn,256> redCcm_{},greenCcm_{},blueCcm_{};
    std::array<double,1024> gammaTable_{};
    bool ccmEnabled_=false, swapRedBlueGains_=false;
    DebayerParams params_{};
    void updateGammaTable(DebayerParams &);
    void updateLookupTables(DebayerParams &);
};
'''
    tests = r'''
int main() {
    DebayerParams p{};
    p.gamma=1.0f/2.2f; p.contrastExp=1.0f;
    p.blackLevel=RGB<float>(16.0f/256);
    p.combinedMatrix=Matrix<float,3,3>::identity();
    for (float gain: {1.0f,1.66f,4.0f}) {
        DebayerCpu d;
        p.gains=RGB<float>({gain,1.0f,1.23f});
        d.updateLookupTables(p);
        for (unsigned i=0;i<=16;i++)
            assert(d.red_[i]==0 && d.green_[i]==0 && d.blue_[i]==0);
        assert(d.red_[17]>0 && d.green_[17]>0 && d.blue_[17]>0);
        for (unsigned i=1;i<256;i++) {
            assert(d.red_[i]>=d.red_[i-1]);
            assert(d.green_[i]>=d.green_[i-1]);
            assert(d.blue_[i]>=d.blue_[i-1]);
        }
        // Neutral RAW signal scaled inversely by WB must stay neutral.
        if (gain==4.0f) assert(d.red_[36]==d.green_[96]);
        d.swapRedBlueGains_=true; d.params_={}; d.updateLookupTables(p);
        assert(d.blue_[16]==0 && d.red_[16]==0);
        if (gain==4.0f) assert(d.blue_[36]==d.green_[96]);
    }
    // Unequal offsets and a matrix with off-diagonal terms: each input
    // channel's entire contribution must be zero at its own black level.
    DebayerCpu d; d.ccmEnabled_=true;
    p.blackLevel=RGB<float>({8.0f/256,16.0f/256,24.0f/256});
    p.combinedMatrix=Matrix<float,3,3>({1.2f,-.1f,-.1f,-.2f,1.3f,-.1f,-.1f,-.1f,1.2f});
    d.updateLookupTables(p);
    auto zero=[](auto col) { return col.r==0 && col.g==0 && col.b==0; };
    assert(zero(d.redCcm_[8]) && zero(d.greenCcm_[16]) && zero(d.blueCcm_[24]));
    // Stock Sunny matrix with real-scene white balance. Saturated neutral
    // inputs must not become pink. Also exercise swapped RGB storage.
    const Matrix<float,3,3> sunny({1.6887f,-.7724f,.0837f,-.1941f,1.3974f,-.2033f,.0445f,-.7526f,1.7081f});
    p.blackLevel=RGB<float>(16.0f/256);
    p.gains=RGB<float>({2.12f,1.0f,1.37f});
    p.combinedMatrix=sunny*Matrix<float,3,3>({2.12f,0,0,0,1,0,0,0,1.37f});
    for (bool swap: {false,true}) {
        DebayerCpu c; c.ccmEnabled_=true; c.swapRedBlueGains_=swap;
        c.updateLookupTables(p);
        auto pixel=[&](int r,int g,int b) {
            auto R=swap?c.blueCcm_[r]:c.redCcm_[r];
            auto G=c.greenCcm_[g];
            auto B=swap?c.redCcm_[b]:c.blueCcm_[b];
            auto gamma=[&](int v) { return int(c.gammaLut_[std::clamp(v,0,255)]); };
            return std::array<int,3>{gamma(R.r+G.r+B.r),gamma(R.g+G.g+B.g),gamma(R.b+G.b+B.b)};
        };
        auto white=pixel(255,255,255);
        assert(*std::min_element(white.begin(),white.end())>=253);
        // Unsaturated coloured signal must still match the linear matrix,
        // within LUT integer rounding and gamma-table quantisation.
        auto colour=pixel(50,80,40);
        const float input[3]={(50-16)*256.0f/240*2.12f,(80-16)*256.0f/240,(40-16)*256.0f/240*1.37f};
        for(int row=0;row<3;row++) {
            float linear=0;
            for(int col=0;col<3;col++) linear+=sunny[row][col]*input[col];
            int expected=c.gammaLut_[std::clamp(int(std::round(linear)),0,255)];
            assert(std::abs(colour[swap?2-row:row]-expected)<=3);
        }
    }
    std::cout << "PASS: production LUT black neutrality, WB, monotonicity, RGB swap, per-channel CCM offsets\n";
    std::cout << "PASS: Sunny CCM saturated-white neutrality and unsaturated colour preservation\n";
}
'''
    (work / "test.cpp").write_text(harness + methods + tests)
    subprocess.run(["g++", "-std=c++20", "-DLIBCAMERA_BASE_PRIVATE", "-O1", "-g", "-fsanitize=address,undefined",
                    "-I" + str(source / "include"), str(work / "test.cpp"), "-o", str(work / "test")], check=True)
    subprocess.run([str(work / "test")], check=True)
