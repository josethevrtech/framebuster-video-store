#!/bin/sh
set -eu
base=$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)
export LD_LIBRARY_PATH="$base/matineevr/ffmpeg:/opt/steamvr/bin/linuxarm64${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
export XR_RUNTIME_JSON=/opt/steamvr/steamxr_linuxarm64.json
export VK_DRIVER_FILES="$base/matineevr/mesa/freedreno_icd.aarch64.json"
export HALCYON_FRAME_CONTROLLER_MESH="$base/native-player/controller-mesh.bin"
ulimit -c 0
app_key="steam.app.${SteamAppId:?Launch FrameBuster Video Store from Steam}"
app_name="FrameBuster Video Store"
manifest="$base/native-player/halcyon-frame.vrmanifest"
printf '{"applications":[{"app_key":"%s","launch_type":"url","url":"steam://rungameid/%s","strings":{"en_us":{"name":"%s"}}}]}\n' \
    "$app_key" "${SteamGameId:?Missing Steam game ID}" "$app_name" > "$manifest"
vrcmd=/opt/steamvr/bin/linuxarm64/vrcmd
"$vrcmd" --background --appmanifest "$manifest"
result=$("$vrcmd" --background --prelaunch "$app_key")
case "$result" in
    *' return VRApplicationError_None') ;;
    *) printf 'SteamVR prelaunch failed: %s\n' "$result" >&2; exit 1 ;;
esac
if [ "${FRAMEBUSTER_NATIVE:-0}" = 1 ]; then
    export FRAMEBUSTER_PLAYER_PRELOAD="${LD_PRELOAD:-}"
    unset LD_PRELOAD
    exec "$base/runtime/bin/node" "$base/steam-frame/native-store-runner.mjs" "$@"
fi
if [ "${HALCYON_FRAME_STORE:-0}" = 1 ]; then
    exec "$base/native-player/halcyon-frame-player-store" "$@"
fi
exec "$base/runtime/bin/node" "$base/steam-frame/native-runner.mjs" "$@"
