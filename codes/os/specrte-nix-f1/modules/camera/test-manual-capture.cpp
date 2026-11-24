#include "manual-capture.h"
#include <cassert>
#include <limits>
#include <iostream>
int main() {
    using namespace spectre;
    ManualExposure state;
    auto update = [&](ExposureRequest r) { state.update(r, 12000, 3, 10, 50000, 1, 16); };
    ExposureRequest r;
    r.exposureUs = 40000; r.gain = 12;
    update(r); // Auto must ignore, including for a later switch.
    r = {}; r.ae = false; update(r);
    assert(!state.exposureAuto && !state.gainAuto && state.exposureUs == 12000 && state.gain == 3);
    r = {}; r.exposureUs = 90000; r.gain = -1; update(r);
    assert(state.exposureUs == 50000 && state.gain == 1);
    auto queued = state;
    r = {}; r.exposureUs = 2000; r.gain = 5; update(r);
    assert(queued.exposureUs == 50000 && queued.gain == 1); // Per-frame snapshot.
    update({}); assert(state.exposureUs == 2000 && state.gain == 5);
    r = {}; r.exposureUs = std::numeric_limits<double>::quiet_NaN();
    r.gain = std::numeric_limits<double>::infinity(); update(r);
    assert(state.exposureUs == 2000 && state.gain == 5);
    r = {}; r.ae = true; r.exposureAuto = false; r.exposureUs = 4000; r.gain = 10; update(r);
    assert(!state.exposureAuto && state.gainAuto && state.exposureUs == 4000 && state.gain == 5);
    r = {}; r.ae = true; update(r); assert(state.exposureAuto && state.gainAuto);
    r = {}; r.ae = false; update(r); assert(state.exposureUs == 12000 && state.gain == 3);
    assert(exposureLines(1001, 10, 1, 5000) == 100);
    assert(exposureLines(1006, 10, 1, 5000) == 101);
    assert(exposureLines(1e300, 1e-300, 1, 5000) == 5000);
    assert(exposureLines(-10, 10, 1, 5000) == 1);
    assert(exposureLines(100, 0, 1, 5000) == 1);
    ManualWhiteBalance wb;
    wb.update({}, 7, 7, 2, 1.5f);
    wb.update(false, {}, {}, 2, 1.5f); assert(wb.red == 2 && wb.blue == 1.5f);
    wb.update({}, 20, -1, 1, 1); assert(wb.red == 8 && wb.blue == 0.1f);
    wb.update({}, 2, std::numeric_limits<float>::quiet_NaN(), 1, 1);
    assert(wb.red == 8 && wb.blue == 0.1f);
    wb.update(true, 5, 5, 3, 2); assert(wb.automatic);
    wb.update(false, {}, {}, 3, 2); assert(wb.red == 3 && wb.blue == 2);
    std::cout << "Manual capture: persistence, precedence, freezing, finite bounds and quantization passed\n";
}
