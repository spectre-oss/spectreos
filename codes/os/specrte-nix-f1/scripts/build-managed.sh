#!/usr/bin/env bash
# Prebuild memory-heavy WebKit sequentially, then parallelize smaller packages.
set -euo pipefail
cd "$(dirname "$0")/.."
store="${NIX_STORE_ROOT:-$(realpath ../nix-store)}"
mkdir -p .build/roots
nix-build --keep-going --store "$store" --option build-users-group '' \
  --cores 4 --max-jobs 1 . -A pkgs.webkitgtk_4_1 -A pkgs.webkitgtk_6_0 \
  -o .build/roots/managed-webkit
make build JOBS=3 MAX_JOBS=2
