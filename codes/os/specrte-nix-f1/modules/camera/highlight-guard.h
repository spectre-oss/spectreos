/* SPDX-License-Identifier: LGPL-2.1-or-later */
#pragma once
#include <algorithm>
#include <cmath>
#include <cstdint>
#include <numeric>

namespace libcamera::ipa::soft::algorithms {
/* Return an AGC MSV command, preserving its existing exposure/gain stepping.
 * Quantile rejects small specular/light-source areas. Black-subtracted linear
 * luminance, not display gamma values, determines the available headroom.
 */
template<class Histogram>
double highlightGuard(const Histogram &histogram, unsigned blackLevel,
                      double quantile, double target, double msv)
{
    if (!(quantile > 0 && quantile <= 1 && target > 0 && target < 1))
        return msv;
    uint64_t total = std::accumulate(histogram.begin(), histogram.end(), uint64_t(0));
    if (!total || blackLevel >= 255)
        return msv;
    uint64_t threshold = std::max<uint64_t>(1, std::ceil(total * quantile));
    uint64_t seen = 0;
    unsigned bin = 0;
    for (; bin + 1 < histogram.size(); ++bin) {
        seen += histogram[bin];
        if (seen >= threshold)
            break;
    }
    double value = std::max(0.0, ((bin + 0.5) * 256.0 / histogram.size() - blackLevel) /
                               (255.0 - blackLevel));
    constexpr double hysteresis = 0.04;
    if (value > target + hysteresis)
        return std::max(msv, 2.8); // request the normal 10% exposure decrease
    if (value >= target - hysteresis)
        return std::max(msv, 2.5); // do not brighten into clipping
    return msv;
}
}
