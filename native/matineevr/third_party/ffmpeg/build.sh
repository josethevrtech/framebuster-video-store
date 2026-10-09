#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../.."
root=$PWD
recipe=$(realpath third_party/ffmpeg)
cache=$(realpath -m artifacts/ffmpeg)
revision=$({ sha256sum "$recipe/build.sh" "$recipe/dmabuf.patch" "$recipe/zig.patch"; zig version; } | sha256sum | cut -d' ' -f1)
work="$cache/$revision"
if [[ ! -f "$work/complete" ]]; then
    mkdir -p "$work/build"
    archive="$cache/ffmpeg-7.0.tar.xz"
    if [[ ! -f "$archive" ]]; then
        curl --fail --location https://ffmpeg.org/releases/ffmpeg-7.0.tar.xz -o "$archive.part"
        mv "$archive.part" "$archive"
    fi
    printf '%s  %s\n' 4426a94dd2c814945456600c8adfc402bee65ec14a70e8c531ec9a2cd651da7b "$archive" | sha256sum -c
    if [[ ! -f "$work/patched" ]]; then
        tar -xf "$archive" -C "$work"
        patch -d "$work/ffmpeg-7.0" -p1 < "$recipe/zig.patch"
        patch -d "$work/ffmpeg-7.0" -p1 < "$recipe/dmabuf.patch"
        touch "$work/patched"
    fi
    cd "$work/build"
    "$work/ffmpeg-7.0/configure" \
        --cc='zig cc -target aarch64-linux-gnu.2.31' --arch=aarch64 --target-os=linux \
        --enable-cross-compile --enable-shared --disable-static --disable-autodetect \
        --disable-doc --disable-programs --disable-network --disable-debug \
        --disable-avdevice --disable-avformat --disable-avfilter --disable-swscale --disable-postproc \
        --disable-encoders --disable-muxers --enable-v4l2-m2m
    timeout 1200 make -j"${JOBS:-8}" \
        LDFLAGS='-Llibavcodec -Llibavformat -Llibavutil -Llibswresample -Wl,--as-needed -Wl,-z,noexecstack'
    touch "$work/complete"
fi
mkdir -p "$root/dist/ffmpeg"
cp -L "$work/build/libavcodec/libavcodec.so.61" "$root/dist/ffmpeg/"
cp "$work/ffmpeg-7.0/COPYING.LGPLv2.1" "$root/dist/FFMPEG-LICENSE"
cp "$cache/ffmpeg-7.0.tar.xz" "$root/dist/"
