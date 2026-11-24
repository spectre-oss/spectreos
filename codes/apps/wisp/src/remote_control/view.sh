#!/bin/sh
# Native Rust viewer; defaults to attaching to the persistent poco session.
set -eu
viewer_dir=$(CDPATH= cd -- "$(dirname "$0")/../../../../utils/wisp_remote_viewer" && pwd)
exec cargo run --quiet --release --locked --manifest-path "$viewer_dir/Cargo.toml" -- "$@"
