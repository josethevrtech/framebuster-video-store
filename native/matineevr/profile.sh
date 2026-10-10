#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"
source scripts/device.sh
bash third_party/ffmpeg/build.sh >&2
bash third_party/mesa/build.sh >&2
cargo build --locked --release --target aarch64-unknown-linux-gnu --example profile >&2
frame_ssh 'mkdir -p matineevr-tools/ffmpeg matineevr-tools/mesa'
frame_scp target/aarch64-unknown-linux-gnu/release/examples/profile "$FRAME_HOST:matineevr-tools/profile" >&2
frame_scp dist/ffmpeg/*.so.* "$FRAME_HOST:matineevr-tools/ffmpeg/" >&2
frame_scp dist/mesa/*.so dist/mesa/*.json "$FRAME_HOST:matineevr-tools/mesa/" >&2
printf -v remote_command '%q ' ./matineevr-tools/profile "$@"
frame_ssh "export LD_LIBRARY_PATH=\$HOME/matineevr-tools/ffmpeg:/opt/steamvr/bin/linuxarm64; export XR_RUNTIME_JSON=/opt/steamvr/steamxr_linuxarm64.json; export VK_DRIVER_FILES=\$HOME/matineevr-tools/mesa/freedreno_icd.aarch64.json; exec $remote_command"
