/* SPDX-License-Identifier: MIT */
#include "raw10-reduce.h"
#include <array>
#include <cassert>
#include <cstdio>
#include <vector>

int main()
{
	// Unequal CFA values catch accidental colour mixing; odd values exercise
	// the two low packed bits. Padded rows catch incorrect source stride.
	constexpr unsigned width = 64, height = 32, stride = width * 5 / 4 + 16;
	std::vector<uint8_t> input(stride * height, 0xa5);
	std::vector<uint8_t> output(width / 4 * 5 / 4 * height / 4 + 32, 0xcd);
	const std::array<unsigned, 4> base{67, 221, 399, 705};
	auto sample = [&](unsigned x, unsigned y) {
		return base[(y % 2) * 2 + x % 2] + (x / 2) + (y / 2) * 2;
	};
	for (unsigned y = 0; y < height; ++y) {
		for (unsigned x = 0; x < width; x += 4) {
			auto *p = input.data() + y * stride + x / 4 * 5;
			p[4] = 0;
			for (unsigned k = 0; k < 4; ++k) {
				unsigned v = sample(x + k, y);
				p[k] = v >> 2;
				p[4] |= (v & 3) << (2 * k);
			}
		}
	}
	spectre::reduceRaw10By4(input.data(), stride, output.data(), width, height);
	for (unsigned y = 0; y < height / 4; ++y) {
		for (unsigned x = 0; x < width / 4; ++x) {
			// Analytic mean of the test ramp: offsets average to
			// 1.5 + 2*1.5 = 4.5, rounded to 5.
			unsigned expected = base[(y % 2) * 2 + x % 2] +
					    4 * (x / 2) + 8 * (y / 2) + 5;
			assert(spectre::raw10(output.data() + y * (width / 4 * 5 / 4), x) ==
			       expected);
		}
	}
	for (size_t i = output.size() - 32; i < output.size(); ++i)
		assert(output[i] == 0xcd);
	std::puts("RAW10 reduction: CFA isolation, averaging, low bits, padded stride and output bounds PASS");
}
