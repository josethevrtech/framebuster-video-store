#!/bin/sh
set -eu
base=$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)
export HALCYON_FRAME_STORE=1
export FRAMEBUSTER_NATIVE=1
if [ -f "$base/framebuster.env" ]; then
    set -a
    . "$base/framebuster.env"
    set +a
fi
exec "$base/steam-frame/native-launch.sh" --projection flat --stereo mono --stats
