#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
mkdir -p artifacts
output=$(mktemp -d "$PWD/artifacts/packet-test-XXXXXXXX")
printf 'Packet test artifacts: %s\n' "$output"
cc -O2 -g -Wall -Wextra -Werror -I native tests/packet_reader.c tests/packet_limits.c native/*.c \
    $(pkg-config --cflags --libs libavformat libavcodec libavutil libswresample libpulse) \
    -Wl,--wrap=av_read_frame -lm -o "$output/packet-test"
ffmpeg -v error -n -f lavfi -i 'color=size=64x64:rate=24:duration=4' \
    -f lavfi -i 'sine=frequency=440:sample_rate=48000:duration=2' \
    -c:v libx264 -g 24 -pix_fmt yuv420p -c:a aac "$output/fixture.mp4"
timeout 15 "$output/packet-test" "$output/fixture.mp4"
