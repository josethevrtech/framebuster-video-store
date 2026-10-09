# Pixel verification

Build with the same ARM64 environment as the player:

```sh
./build.sh
cargo build --locked --release --target aarch64-unknown-linux-gnu --example verify
```

Copy `verify`, `dist/ffmpeg/` and `dist/mesa/` into one directory on the Frame. From there:

```sh
export LD_LIBRARY_PATH="$PWD/ffmpeg:/opt/steamvr/bin/linuxarm64"
export XR_RUNTIME_JSON=/opt/steamvr/steamxr_linuxarm64.json
export VK_DRIVER_FILES="$PWD/mesa/freedreno_icd.aarch64.json"
timeout 120 ./verify /path/to/video.mp4 1.3 0.1
```

This compares DMA-BUF samples against system FFmpeg software decoding at matching
PTS: raw YUV at native precision, RGB conversion and chroma placement. It also
holds GPU execution while decoding 16 further frames, then checks the retained
frame. Leave at least 16 frames after each requested position.

Raw tolerance is one source code value; RGB tolerance includes two code values
and the device's filtering precision. Readback uses chunks within the storage-buffer limit.
10-bit inputs must exercise their lower two bits. HDR remains unsupported.
Use `profile.sh --help` for performance measurements.
