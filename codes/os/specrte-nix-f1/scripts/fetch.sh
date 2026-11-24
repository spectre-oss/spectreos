#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
if [[ ! -d mobile-nixos ]]; then
  git init mobile-nixos
  git -C mobile-nixos remote add origin https://github.com/mobile-nixos/mobile-nixos
  git -C mobile-nixos fetch --depth 1 origin 2c132754323fc1915e8d21dcfc0ef68ab084c6fb
  git -C mobile-nixos switch -c specrte-nix-f1 FETCH_HEAD
  git -C mobile-nixos apply ../patches/mobile-kernel-olddefconfig.patch
fi
fetch() {
  local dir=$1 url=$2 file=$3
  if [[ ! -f "$dir/$file" ]]; then
    curl -fL --retry 3 "$url" -o "$dir/$file.part"
    mv "$dir/$file.part" "$dir/$file"
  fi
  (cd "$dir" && sha256sum -c SHA256SUMS)
}
fetch kernel https://gitlab.com/sdm845-mainline/linux/-/archive/85f1df2a4ec71d7a91dd95a7a49f889d1595ffa8/linux-85f1df2a4ec71d7a91dd95a7a49f889d1595ffa8.tar.gz linux.tar.gz
fetch firmware https://gitlab.com/sdm845-mainline/firmware-xiaomi-beryllium/-/archive/d396d32118803235a58aab0d0bc3643256634a05/firmware-xiaomi-beryllium-d396d32118803235a58aab0d0bc3643256634a05.tar.gz beryllium.tar.gz
