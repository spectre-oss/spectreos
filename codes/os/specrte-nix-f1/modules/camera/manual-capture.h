// SPDX-License-Identifier: LGPL-2.1-or-later
// Request-state helpers shared by the Simple IPA and its boundary tests.
#pragma once
#include <algorithm>
#include <cmath>
#include <cstdint>
#include <optional>

namespace spectre {
struct ExposureRequest {
    std::optional<bool> ae, exposureAuto, gainAuto;
    std::optional<double> exposureUs, gain;
};
struct ManualExposure {
    bool exposureAuto = true, gainAuto = true;
    double exposureUs = 1, gain = 1;

    void update(const ExposureRequest &r, double autoUs, double autoGain,
                double minUs, double maxUs, double minGain, double maxGain) {
        // Explicit per-axis modes take precedence over the legacy AeEnable.
        bool nextExposure = r.exposureAuto.value_or(r.ae.value_or(exposureAuto));
        bool nextGain = r.gainAuto.value_or(r.ae.value_or(gainAuto));
        if (exposureAuto && !nextExposure) exposureUs = autoUs;
        if (gainAuto && !nextGain) gain = autoGain;
        exposureAuto = nextExposure;
        gainAuto = nextGain;
        // Controls sent in auto mode are ignored, not saved for a later lock.
        if (!exposureAuto && r.exposureUs && std::isfinite(*r.exposureUs))
            exposureUs = std::clamp(*r.exposureUs, minUs, maxUs);
        if (!gainAuto && r.gain && std::isfinite(*r.gain))
            gain = std::clamp(*r.gain, minGain, maxGain);
    }
};
inline int32_t exposureLines(double us, double lineUs, int32_t minimum, int32_t maximum) {
    if (!std::isfinite(us) || !std::isfinite(lineUs) || lineUs <= 0 || minimum > maximum)
        return minimum;
    // Clamp before conversion, including finite values whose quotient overflows.
    return int32_t(std::round(std::clamp(us / lineUs, double(minimum), double(maximum))));
}
struct ManualWhiteBalance {
    bool automatic = true;
    float red = 1, blue = 1;
    void update(std::optional<bool> enabled, std::optional<float> r,
                std::optional<float> b, float autoRed, float autoBlue) {
        bool next = enabled.value_or(automatic);
        if (automatic && !next) { red = autoRed; blue = autoBlue; }
        automatic = next;
        // Reject the whole pair if either member is invalid.
        if (!automatic && r && b && std::isfinite(*r) && std::isfinite(*b)) {
            red = std::clamp(*r, 0.1f, 8.0f);
            blue = std::clamp(*b, 0.1f, 8.0f);
        }
    }
};
}
