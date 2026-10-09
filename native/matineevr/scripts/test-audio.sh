#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
mkdir -p artifacts
output=$(mktemp -d "$PWD/artifacts/audio-test-XXXXXXXX")
printf 'Audio test artifacts: %s\n' "$output"
: "${PULSE_SERVER:?Set PULSE_SERVER to an isolated test server with a null sink}"
cc -O2 -Wall -Wextra -Werror -I native tests/audio_native.c tests/audio_operations.c native/*.c \
    $(pkg-config --cflags --libs libavformat libavcodec libavutil libswresample libpulse) \
    -Wl,--wrap=pa_stream_cork,--wrap=pa_stream_write -lm -o "$output/audio-test"
for rate in 48000 44100; do
    ffmpeg -v error -n -f lavfi -i 'color=size=64x64:rate=24:duration=4' \
        -f lavfi -i "aevalsrc=0.1*sin(2*PI*440*t)|0.1*sin(2*PI*660*t)|0.1*sin(2*PI*880*t)|0.1*sin(2*PI*220*t):s=$rate:d=2:c=4.0" \
        -c:v libx264 -g 24 -pix_fmt yuv420p -c:a aac "$output/$rate.mp4"
    ffmpeg -v error -n -i "$output/$rate.mp4" -vn -ac 2 -ar 48000 -t 2 \
        -f f32le "$output/$rate.f32"
    timeout 30 "$output/audio-test" "$output/$rate.mp4" "$output/$rate.f32"
    ffmpeg -v error -n -i "$output/$rate.mp4" -f f32le -ar 48000 -ac 2 -i "$output/$rate.f32" \
        -map 0:v -map 1:a -c:v copy -c:a flac -frame_size 512 "$output/$rate.mkv"
    timeout 30 "$output/audio-test" "$output/$rate.mkv" "$output/$rate.f32"
    ffmpeg -v error -n -i "$output/$rate.mp4" -c copy -output_ts_offset 5 "$output/$rate-offset.mp4"
    ffmpeg -v error -n -copyts -i "$output/$rate-offset.mp4" -vn \
        -af 'aresample=48000,atrim=start=5:end=7' -ac 2 -f f32le "$output/$rate-offset.f32"
    timeout 30 "$output/audio-test" "$output/$rate-offset.mp4" "$output/$rate-offset.f32"
done
ffmpeg -v error -n -i "$output/48000.mp4" -an -c:v copy "$output/silent.mp4"
PULSE_SERVER="unix:$output/missing" timeout 10 "$output/audio-test" "$output/silent.mp4"
