/* SPDX-License-Identifier: MIT */
#pragma once
#include <cstddef>
#include <cstdint>

namespace spectre {
inline unsigned raw10(const uint8_t *row, unsigned x)
{
	const uint8_t *group = row + (x / 4) * 5;
	return (unsigned(group[x % 4]) << 2) |
	       ((group[4] >> (2 * (x % 4))) & 3);
}

/*
 * Reduce each 8x8 Bayer cell group to a 2x2 Bayer cell. Average only
 * samples at the same CFA position, in linear 10-bit sensor space.
 * Preserve all four CFA positions, including the two distinct greens.
 * The caller validates dimensions divisible by 16x8 and buffer lengths.
 */
inline void reduceRaw10By4(const uint8_t *src, unsigned sourceStride,
			  uint8_t *dst, unsigned width, unsigned height)
{
	const unsigned outWidth = width / 4;
	const unsigned outHeight = height / 4;
	const unsigned outStride = outWidth * 5 / 4;
	for (unsigned y = 0; y < outHeight; ++y) {
		const unsigned sy = (y / 2) * 8 + (y % 2);
		for (unsigned x = 0; x < outWidth; x += 4) {
			uint8_t *out = dst + size_t(y) * outStride + (x / 4) * 5;
			unsigned low = 0;
			for (unsigned k = 0; k < 4; ++k) {
				const unsigned sx = ((x + k) / 2) * 8 + ((x + k) % 2);
				unsigned sum = 0;
				for (unsigned j = 0; j < 4; ++j) {
					const uint8_t *row = src + size_t(sy + j * 2) * sourceStride;
					for (unsigned i = 0; i < 4; ++i)
						sum += raw10(row, sx + i * 2);
				}
				const unsigned value = (sum + 8) / 16;
				out[k] = value >> 2;
				low |= (value & 3) << (2 * k);
			}
			out[4] = low;
		}
	}
}
}
