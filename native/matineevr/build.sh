#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"
bash third_party/ffmpeg/build.sh
bash third_party/mesa/build.sh
cargo build --locked --release --target aarch64-unknown-linux-gnu
mkdir -p dist
cp target/aarch64-unknown-linux-gnu/release/matineevr dist/matineevr
printf '%s\n' 'Built ARM64 player with embedded media bridge in dist/matineevr.'
