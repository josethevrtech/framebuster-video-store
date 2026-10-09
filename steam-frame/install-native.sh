#!/bin/sh
set -eu
base=$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)
[ "$(uname -m)" = aarch64 ] || { echo 'FrameBuster targets Steam Frame ARM64.' >&2; exit 1; }
[ -x "$base/runtime/bin/node" ] || { echo 'Install the ARM64 Node runtime in runtime/bin/node.' >&2; exit 1; }
[ -x "$base/native-player/framebuster-video-store" ] || { echo 'Place the native release binary in native-player/framebuster-video-store.' >&2; exit 1; }
[ -f /opt/steamvr/steamxr_linuxarm64.json ] || { echo 'Steam Frame OpenXR runtime is missing.' >&2; exit 1; }
command -v ffmpeg >/dev/null
command -v zenity >/dev/null
chmod +x "$base/steam-frame/framebuster-launch.sh" "$base/steam-frame/native-launch.sh"
if [ ! -f "$base/native-player/controller-mesh.bin" ]; then
    python "$base/steam-frame/prepare-controller-models.py" "$base/native-player/controller-mesh.bin"
fi
python "$base/steam-frame/register-native.py" --framebuster-only
echo 'FrameBuster Video Store is installed in Steam. Launch Devkit Game: FrameBusterVideoStore.'
