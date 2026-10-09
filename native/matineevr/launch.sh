#!/bin/sh
set -eu
directory=$(CDPATH= cd -- "$(dirname "$0")" && pwd)
export LD_LIBRARY_PATH="$directory/ffmpeg:/opt/steamvr/bin/linuxarm64${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
export XR_RUNTIME_JSON=/opt/steamvr/steamxr_linuxarm64.json
ulimit -c 0
app_key="steam.app.${SteamAppId:?Launch MatineeVR from Steam}"
printf '{"applications":[{"app_key":"%s","launch_type":"url","url":"steam://rungameid/%s","strings":{"en_us":{"name":"MatineeVR"}}}]}\n' \
    "$app_key" "${SteamGameId:?Missing Steam game ID}" > "$directory/matineevr.vrmanifest"
vrcmd=/opt/steamvr/bin/linuxarm64/vrcmd
"$vrcmd" --background --appmanifest "$directory/matineevr.vrmanifest"
result=$("$vrcmd" --background --prelaunch "$app_key")
case "$result" in
    *' return VRApplicationError_None') ;;
    *) printf 'SteamVR prelaunch failed: %s\n' "$result" >&2; exit 1 ;;
esac
VK_DRIVER_FILES="$directory/mesa/freedreno_icd.aarch64.json" exec "$directory/matineevr" "$@"
