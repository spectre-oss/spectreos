#!/bin/sh
# Build on the build host; downloads public reference sources, never contacts phone.
set -eu
base=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
out=${1:-.build/fingerprint/diagnostics}
mkdir -p "$out/rpmb-vendor"
revision=5e3c6951d13162a262ea65ea8e8351228f092b58
for name in rpmb_ufs.c rpmb_ufs.h rpmb.h rpmb_private.h rpmb_logging.h; do
    curl --fail --silent --show-error --location \
        "https://raw.githubusercontent.com/qualcomm/minkipc/$revision/listeners/librpmbservice/$name" \
        -o "$out/rpmb-vendor/$name"
done
${CC:-aarch64-linux-gnu-gcc} -static -O2 -Wall -Wextra -Werror \
    -DRPMB_DEBUG_CONSOLE_OUTPUT -I"$out" "$base/rpmb-bridge.c" -o "$out/rpmb-bridge"
${CC:-aarch64-linux-gnu-gcc} -static -O2 -Wall -Wextra -Werror \
    "$base/gatekeeper-preflight.c" -o "$out/gfber-identify"
${CC:-aarch64-linux-gnu-gcc} -static -O2 -Wall -Wextra -Werror \
    "$base/key-exchange-only.c" -o "$out/key-exchange-only"
