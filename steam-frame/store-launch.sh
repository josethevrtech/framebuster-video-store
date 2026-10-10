#!/bin/sh
set -eu
base=$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)
export HALCYON_FRAME_STORE=1
export HALCYON_FRAME_STORE_SAMPLE="$base/media/sample-h264.mp4"
exec "$base/steam-frame/native-launch.sh" "$base/media" --projection flat --stereo mono --stats
