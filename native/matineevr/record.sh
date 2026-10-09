#!/bin/sh
set -eu
cd "$(dirname "$0")"
. ./scripts/device.sh

directory="$HOME/Videos/SteamFrame"
mkdir -p "$directory"
output="$directory/$(date +%Y-%m-%d_%H-%M-%S)-$$.mp4"
remote=$(frame_ssh 'mktemp ~/steam-frame-record-XXXXXXXX.mp4')

printf 'Recording the headset view. Press Ctrl+C to stop and save.\n'
printf 'Remote recording: %s:%s\n' "$FRAME_HOST" "$remote"
# A remote terminal forwards Ctrl+C to FFmpeg so it can finalize the MP4.
trap ':' INT
# FFmpeg returns 255 after Ctrl+C; SSH failures must still stop the script.
frame_ssh -t "ffmpeg -hide_banner -loglevel warning -stats \
    -f v4l2 -i /dev/video99 -an -r 30 \
    -c:v libx264 -preset ultrafast -crf 23 -pix_fmt yuv420p \
    -y '$remote' || [ \$? -eq 255 ]"

printf '\nSaving recording...\n'
frame_scp "$FRAME_HOST:$remote" "$output"
frame_ssh "rm -- '$remote'"
printf 'Saved: %s\n' "$output"
