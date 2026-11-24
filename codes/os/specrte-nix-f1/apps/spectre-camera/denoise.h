// SPDX-License-Identifier: GPL-3.0-or-later
#pragma once
#include <QImage>
#include <QJsonObject>
#include <vector>
struct NoiseFrame { QImage image; QJsonObject metadata; };
struct NoiseResult { QImage image; QJsonObject report; };
bool noiseFramesCompatible(const QJsonObject &reference, const QJsonObject &candidate);
QImage denoiseChroma(const QImage &image, float gain, bool preview);
NoiseResult denoiseBurst(const std::vector<NoiseFrame> &frames);
bool noiseSelfTest();
