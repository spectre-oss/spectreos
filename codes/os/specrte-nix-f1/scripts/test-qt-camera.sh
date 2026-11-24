#!/usr/bin/env bash
# Execute ONLY on the build server after make qt-camera-test has built the app.
set -euo pipefail
project=$(cd "$(dirname "$0")/.." && pwd)
store=${NIX_STORE_ROOT:-$(dirname "$project")/nix-store}
app=$(readlink "$project/.build/roots/result-qt-camera-test")
work="$project/.build/qt-tests"
mkdir -p "$work/runtime"
chmod 700 "$work/runtime"
c++ -std=c++17 -O2 -g -fsanitize=address,undefined -fno-omit-frame-pointer   "$project/modules/camera/test-raw10-denoise.cpp" -o "$work/raw-denoise-test"
"$work/raw-denoise-test"
c++ -std=c++17 -O2 -g -fsanitize=address,undefined -fno-omit-frame-pointer \
  "$project/modules/camera/test-raw10-defects.cpp" -o "$work/raw-defects-test"
"$work/raw-defects-test"
c++ -std=c++17 -O2 -g -fsanitize=address,undefined -fno-omit-frame-pointer \
  "$project/modules/camera/test-manual-capture.cpp" -o "$work/manual-capture-test"
"$work/manual-capture-test"
c++ -std=c++17 -O2 -g -fsanitize=address,undefined -fno-omit-frame-pointer \
  "$project/modules/camera/test-raw10-shading.cpp" -o "$work/raw-shading-test"
"$work/raw-shading-test"
run() {
  bwrap --ro-bind / / --tmpfs /nix --dir /nix/store \
    --ro-bind "$store/nix/store" /nix/store \
    --dev-bind /dev /dev --proc /proc --tmpfs /tmp \
    --bind "$work" /tmp/qt-test \
    --setenv HOME /tmp/qt-test --setenv XDG_RUNTIME_DIR /tmp/qt-test/runtime \
    --setenv XDG_CONFIG_HOME /tmp/qt-test/config --setenv XDG_DATA_HOME /tmp/qt-test/data \
    --setenv QT_QPA_PLATFORM offscreen --setenv QT_QUICK_BACKEND software \
    --setenv QT_FORCE_STDERR_LOGGING 1 --setenv LANG C.UTF-8 \
    "$app/bin/spectre-camera" "$@"
}
run --self-test
run --noise-self-test
run --demo --smoke-test --screenshot /tmp/qt-test/preview.png
run --demo --controls --smoke-test --screenshot /tmp/qt-test/controls.png
run --demo --rotation-test --screenshot /tmp/qt-test/rotation
run --demo --capture-test
run --demo --lossless --capture-test
printf 'Qt Camera: RGB layout, QML render, controls render, repeated JPEG/PNG save/reopen passed.\n'
