#!/usr/bin/env bash
# Keep Nix's logical GC root, and expose a host-readable result separately.
set -euo pipefail
root_link=${1:?registered Nix result link}
public_link=${2:?public result link}
store_root=${3:-}
target=$(readlink "$root_link")
case "$target" in
  /nix/store/*) resolved="${store_root%/}$target" ;;
  *) echo "Unexpected Nix result target: $target" >&2; exit 1 ;;
esac
[[ -e "$resolved" ]] || { echo "Nix result is not accessible at $resolved" >&2; exit 1; }
ln -sfnT "$resolved" "$public_link"
