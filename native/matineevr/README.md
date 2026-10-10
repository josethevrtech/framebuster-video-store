# MatineeVR

A native Steam Frame player using FFmpeg demux/audio, Iris hardware decoding,
DMA-BUF video frames and Vulkan rendering through OpenXR.

Supports 8-bit H.264/HEVC/VP9 and HEVC Main10, including verified 8K Q10C imports.
Supports adjustable equidistant fisheye projection. HDR is unsupported.

## Build and deploy

Build in the `steam-frame` runner image configured by `local-infra`:

```sh
./build.sh
python3 -B scripts/package.py
```

`build.sh` cross-compiles Rust and embeds the C media bridge using Zig and the
runner's pinned Steam Frame FFmpeg SDK. Patched FFmpeg and Mesa builds produce
`dist/ffmpeg/` and `dist/mesa/` alongside the executable. Builds are cached by recipe and tool versions.
Cargo uses the lockfile and
downloads missing dependencies. Building does not require a connected headset.

The SDK must include `libpulse` and `libswresample` headers, libraries and
pkg-config metadata. To extend an older SDK once, with the Frame connected:

```sh
bash scripts/capture-audio-sdk.sh /absolute/path/to/new-sdk
export PKG_CONFIG_SYSROOT_DIR=/absolute/path/to/new-sdk
export PKG_CONFIG_LIBDIR="$PKG_CONFIG_SYSROOT_DIR/usr/lib/pkgconfig"
```

The package command creates a ZIP under `dist/package-*` containing `matineevr`,
`ffmpeg/`, `mesa/`, `install.py`, `deploy.py`, `deploy.cmd`, `LICENSES`, `source.zip`, and [README.user.md](README.user.md)
renamed to `README.md`. `source.zip` contains the tracked files from the current
Git commit, excluding `investigation/` via `.gitattributes`, plus `ffmpeg-7.0.tar.xz`.
Publishing requires a clean checkout; build and package the same commit.
`LICENSES` is generated offline from the locked ARM64 dependencies and the Rust
toolchain used to build the player. Package with that same toolchain after building.
The launcher selects our private Mesa driver only for the player. System drivers remain untouched.
Demuxing, resampling and system libraries remain supplied by SteamOS.
The SDK was captured from SteamOS Preview 0.5.0, build `20260925.6175226`.

From this checkout on the development computer, with the built files in `dist/`:

```sh
python3 deploy.py --launch --projection 180 --stereo sbs
```

`deploy.py` copies the binary into `~/devkit-game/MatineeVR/` and registers
**Devkit Game: MatineeVR** under **Steam > Library > Non-Steam**. Omit
`--launch` to install without starting it. Its Play button retains the path
and options from the latest deployment. With no path, it opens the browser.
Deployment stops the installed player and waits for it to exit before replacing the executable.

The shortcut runs directly on the host.
The executable selects the native SteamVR OpenXR runtime at startup. Unlike the earlier Bevy
experiment, this shortcut does not force the Steam Linux Runtime container.

The same Python deployer serves the checkout and extracted packages. It looks
beside itself for `matineevr`, then in `dist/`. `deploy.sh` delegates to it;
Windows users can double-click `deploy.cmd` to keep output visible until a keypress.
Deployment needs Python 3.8+ and system OpenSSH, with no pip dependencies.

The host defaults to `steamos@frame`. The deployer checks SteamOS Devkit Client's
key location for the local platform, then falls back to normal SSH configuration.
`FRAME_HOST` and `FRAME_SSH_KEY` also override the defaults:

```sh
python3 deploy.py --host 10.0.0.25 --key /path/to/key
```

Build requirements are supplied by `localhost/forgejo-steam-frame-ci:latest`:
Rust with the `aarch64-unknown-linux-gnu` target, Zig, `steam-frame-cc`, `glslc`,
glslang, Meson, Ninja, Bison, Flex, LLVM strip, QEMU, Python Mako/PyYAML/packaging,
Make, `patch`, and `pkg-config` configured for the Frame SDK. Only deployment requires SSH pairing
and an awake, reachable headset.

## Forgejo CI

Every push runs ARM64 release unit tests under QEMU, Clippy, and the build on
`runs-on: steam-frame`. Tests marked ignored still need their fixtures or Frame
hardware. Tag pushes also create a Forgejo release and attach the ARM64
`.zip`, using the job's `GITHUB_TOKEN`. Reruns preserve existing release assets.
The matching `# 0.0.4 - Title` section of `CHANGELOG.md` supplies the Forgejo
release notes; missing, duplicate, or empty sections fail publication. Tags may
have a `v` prefix. The changelog is not included in the ZIP.

After packaging and publishing to Forgejo, the same ZIP is uploaded to
`embedding-shapes/matineevr:steam-frame-linux-arm64`, with the tag as its itch.io
version. Set the repository Actions secret `BUTLER_API_KEY` to an itch.io API
key with upload access. The script downloads pinned Butler 15.31.0 for the Linux
x86_64 runner and verifies its SHA-256 checksum. No repository link is shared
with itch.io. Publish the itch.io project page once to make builds public.

Itch.io's documented upload API has no release-notes field; this workflow only
sets the build version there. Push release tags in order and let each upload
finish before starting another: itch.io orders builds by upload time, so
rerunning an older tag can replace the current download with that older version.

## License and distribution

The project is licensed under [GPL version 3 only](LICENSE). Paid downloads are
allowed. Each public binary release must provide access to its matching
corresponding source, including build and installation scripts, at no extra
charge. The release ZIP includes that commit's tracked source in `source.zip`.
Recipients retain GPL rights to modify and redistribute it.

The settings panel and `matineevr --version` show the Git build version.
Issue reports go to the [Itch page](https://embedding-shapes.itch.io/matineevr).

## Record the headset view

Run on this development computer while the headset is awake:

```sh
./record.sh
```

Press **Ctrl+C** once to stop, finalize the MP4, and download it to
`~/Videos/SteamFrame/` on this computer. Wait for the `Saved:` message.
`FRAME_HOST` and `FRAME_SSH_KEY` override the recording connection;
its defaults are `steamos@10.0.0.25` and `~/.config/steamos-devkit/devkit_rsa`.

It records SteamVR's `/dev/video99` headset mirror as silent H.264 video at
30 fps (currently 1920×1080). FFmpeg runs on the headset using its installed
CPU encoder, so recording adds some CPU load. The temporary recording lives in
the headset's home directory and is removed after a successful download. Its
path is printed at startup so it can be recovered if copying fails.

## Playback

```sh
./matineevr
./matineevr /path/to/video/directory
./matineevr /path/to/video.mp4 --projection 360 --stereo tb
```

- Without a path, the browser starts in `~/Videos`, or the home directory if
  `Videos` does not exist. A directory argument changes its starting folder;
  a file argument starts that video directly.
- **Browser:** D-pad up/down selects, left visits the parent folder, and right
  opens the selected file or folder. Y/X also select up/down; A opens; B goes up a folder
  when no video is playing.
  Four entries are shown per page; selection changes pages automatically.
- Files are listed after FFmpeg finds an eligible H.264, HEVC or VP9 video stream,
  regardless of extension. Scanning runs on a background thread. Metadata
  eligibility does not guarantee hardware decoding; open/decode/import errors
  return to the browser with a message. Decoder shutdown completes before
  another file can start.
- `--projection flat|180|360|fisheye|fisheye190`: screen, equirectangular hemisphere/full
  sphere, or equidistant fisheye with a 180°/190° field of view.
- `--stereo mono|sbs|tb`: mono, left/right halves, or top/bottom halves.
  Left or top is the left eye. Defaults are `180` and `sbs`.
- `--sbs-format full|half`: for Flat + SBS, preserve each eye's proportions (Full,
  the default), or expand each eye horizontally by 2× (Half).
- `--tb-format full|half`: the same for Flat + top/bottom, expanding each eye vertically by 2× for Half.
  Flat video uses FFmpeg's stream/frame sample aspect ratio, defaulting to 1:1 when unspecified.
  Pixel aspect correction and Full/Half selection also apply to seek previews.
- **Playback:** X pauses/resumes, Y shows/hides the HUD, A recenters horizontally,
  and B stops playback and returns to the browser, keeping the selected entry.
  Right-stick left/right skips ten seconds with the right grip released;
  D-pad left/right does the same when neither the browser nor settings is open.
  Hold either input to repeat after a quarter second, accelerating from four to twenty
  skips per second, or release between taps for individual skips.
  Seeking briefly shows progress and an approximate thumbnail.
- Video timing uses frame timestamps. Head tracking continues while paused.
  Playback pauses when the XR session becomes invisible. End of file returns
  to the browser automatically, keeping the selection. Press B to stop early.

On first opening, playback uses explicit CLI options, then supported stream metadata
and filename tokens, then the last played video's settings (initially 180/SBS/Full).
Filename tokens are case-insensitive and separated by punctuation or spaces:
`180`/`VR180`, `360`/`VR360`, `FISHEYE`/`FISHEYE190`, `SBS`/`LR`/`RL`, `TB`/`BT`, and `2D`.
`halfsbs` / `half-sbs` select SBS Half; `halftb` / `half-tb` select top/bottom Half.
Choose Flat for a flat 3D movie. Projection, stereo layout, SBS/top-bottom formats,
and eye order are resolved separately; conflicting or
unsupported hints fall back. Cubemap and arbitrary crops are unsupported.
Every successfully played video's full settings, including manual adjustments,
are remembered by full path until the app exits. Failed opens do not change the
last-used settings. Nothing from this settings memory is written to disk.
Audio uses the system output and volume controls. HEVC Main10 is supported; HDR is unsupported.

Fisheye settings scroll with the selection. FOV spans the image circle's diameter,
which fits the shorter side of each eye image. Image 1/2 centres are fractions of
each source image (left/right for SBS, top/bottom for TB), measured from its top left.
The four lens coefficients use the [OpenCV angular polynomial](https://docs.opencv.org/4.x/db/d58/group__calib3d__fisheye.html);
zero means equidistant. Centres and coefficients start neutral for each new file.
The 190° default follows the [documented equidistant export workflow](https://deovr.com/blog/207-tutorial-using-mistika-boutique-with-the-canon-vr-camera),
not a measured camera calibration. Adjustments are shared by playback and thumbnails.

## Headless profiling

With the same SDK environment as `build.sh`, build, copy and run on the Frame:

```sh
./profile.sh /home/steamos/Videos/stereo-meta-sphere.mp4 \
  1.25 6.6 0 --seconds 1 --repeat 3 > profile.csv
```

Use device paths; an unquoted `~` expands on the local machine.
`FRAME_HOST` and `FRAME_SSH_KEY` override the connection defaults.

Add `--thumbnails --projection fisheye190 --stereo sbs` to measure 256×144
seek previews through GPU completion, with per-stage timings and counters.
See [thumbnail profiling](investigation/THUMBNAIL-PROFILING.md) for the CSV fields.

The default uses the app's shared `Playback`, worker queue and native decoder.
Targets are absolute seconds, in order. Each run opens afresh, measures first-frame
delivery, plays for `--seconds N` (default 1), then repeats delivery/play measurements
for each seek. Use `--seconds 0` for latency only. `--decoder` uses the same native
video decoder synchronously, adds `open` and `seek_call` timings and decodes without
pacing or audio output. The default playback mode includes audio when present.

CSV goes to stdout after each run; diagnostics go to stderr. `start` includes opening;
`seek` ends at the first accepted frame or EOF. `play` reports frames, elapsed time,
PTS range and skipped frames; throughput is frames / elapsed seconds. Playback
uses `--poll-ms N` (default 1); timings include polling/scheduling delay and exclude
graphics by default. Filesystem caches are uncontrolled. Stop other video playback before testing.

Add `--render 1440 1440` for headless GPU measurements at that size per eye.
It uses the app's renderer with the selected projection/stereo (default 180°/SBS), a 90° field of view and two
sRGB targets. `graphics_init` measures setup; `import_ms` measures CPU preparation,
`draw_ms` includes submission and fence waits, and `gpu_ms` uses GPU timestamps.
Totals are per row; playback `cpu_percent` uses one CPU core as 100%. `start`, `seek` and `preview_ready_ms` end at GPU completion;
seek timing includes releasing old submissions. These runs require SteamVR for GPU
selection but no headset session. They do not measure display latency.

Independent pixel verification: [examples/verify/README.md](examples/verify/README.md).

## HUD and shortcut panels

During playback, hold the left inner grip for the file browser and the right
inner grip for video settings. Release the grip fully to hide its panel. Both
panels can be open at once, each moving and rotating with its controller.
Each quad uses that hand's OpenXR grip action space, initially 10 cm above and
10 cm forward of the grip, with a width of 42 cm and a 60° upward tilt.
A panel hides when its grip pose is unavailable. Startup browsing and the
information HUD use `VIEW` space.
D-pad up/down selects browser entries, left opens the parent folder, and right
opens the selection. The right stick selects settings with up/down and changes
values with left/right. The D-pad controls settings when the browser is closed.
Projection, stereo layout, SBS format, and eye order apply immediately without restarting
the decoder. Reopening a video restores its settings for this session. Reset restores
the built-in 180/SBS/Full/normal-eye defaults and remembers that choice for the video.
SBS format is visible only for Flat + Side by side and also applies to seek previews.

Analog-trigger alignment controls are described in [Controls](README.user.md#controls).
Tune response, speeds and limits in `src/alignment.rs`, and trigger thresholds in
`src/adjustment.rs`. Position translates a unit-distance screen or unit-radius panorama;
zoom magnifies around its forward direction. Stereo offsets use fractions of each eye image.

Hand assignments are plain `PanelKind` values in `Shortcuts`; drawing and input
routing use those assignments. There is no assignment editor yet.

The HUD starts hidden; Y toggles it during playback. When visible, it sits below
the center of view and follows the head through an OpenXR `VIEW`-space quad. It shows:

- The absolute video path and displayed frame position / file duration.
- Whole-device CPU and GPU utilization, each on a 0–100% scale.
- Whole-device used / total RAM in GiB, calculated with `MemAvailable`.
- Submitted XR frames per second, the runtime's target app cadence, and video
  frames uploaded per second. These are separate measurements; XR submissions
  do not measure compositor reprojection or physical display frames. The target
  is derived from OpenXR's predicted display period, which can change when the
  runtime throttles the app; it is not the panel's physical refresh rate.
- Hardware decoder, video resolution, skipped video frames, and playback state.

Counters are sampled on one background thread twice per second. Text is drawn
with an embedded bitmap font into a reusable 792×184 RGBA buffer, uploaded at
most twice per second, and reused by the compositor between updates. There is
no font library, shell polling or GPU readback in this path.
The information HUD adds one composition layer; each grip panel uses its own
layer. Grip panels temporarily hide the information panel. Text refreshes
immediately on input. Its cost can be measured with `--stats`.

GPU utilization comes from the installed MSM driver's `debugfs` `perf_now`
counter. It measures GPU busy time, not Iris video-decoder utilization. Missing,
inaccessible, or invalid counters display `N/A`. CPU excludes idle and I/O wait
and is normalized across all CPU cores. Unknown file duration shows `--:--`.
Paths wrap across two lines; longer paths advance a page every four seconds.
Non-ASCII path bytes and backslashes use `\xNN` escapes in the bitmap font.

Use `--hud` to show the performance panel at startup. Its sampling thread runs
only while the panel is visible. The file browser works with the HUD hidden.
`--stats` prints five-second XR, GPU and playback summaries; HUD teardown reports
CPU rasterization and texture-upload submission time, including swapchain wait.
These timings are not GPU execution timings. `--hud-snapshot FILE.ppm` saves
the panel's CPU pixels once, after two seconds, for inspecting text layout.
For playback captures, also pass `--hud`.
Like `--snapshot`, it refuses to overwrite a file. These diagnostic options
are optional and are not needed for normal playback.

With the headset worn, compare four runs (off/on/on/off) of the installed release
build. This saves timing logs and JSON under `artifacts/`, discards each run's
first five-second timing window, and rejects runs without visible playback:

```sh
python3 -B investigation/benchmark_hud.py \
  /home/steamos/Videos/Crabtree-Falls-NC-Hiking-VR180.mp4
# Restore the normal Steam shortcut after the timed runs:
./deploy.sh /home/steamos/Videos/Crabtree-Falls-NC-Hiking-VR180.mp4 \
  --projection 180 --stereo sbs
```

The ignored `native_playback_position_pause_and_end` test runs on the Frame
with `FRAME_TEST_VIDEO` set to a known-duration clip shorter than 20 seconds.
Run the ARM64 release test executable with `--include-ignored`.
It checks actual native decoding, pause, EOF and system
counters. `FRAME_TEST_HUD_SNAPSHOT=/path/new.ppm` also saves a text-layout preview.

Measured on Preview with the 8K hiking video: about 36.00 submitted XR frames/s
with the HUD both on and off; HUD updates averaged 0.36 ms at no more than 2 Hz.
See [HUD validation](investigation/HUD-VALIDATION.md) for results and limits.

See [verification and diagnostics](investigation/VERIFICATION.md) for tests and probes.

## Known issues

- **8K playback can fail after suspend.** Steam can retain the decoder for its
  finished suspend animation. Iris's session accounting then rejects video startup
  with `Cannot allocate memory`. Releasing that animation from memory restored
  8192×4096 decoding with the same binary and video. Restarting the headset also
  restored decoding and rendering in probes.

## Code

- `src/app.rs`, `src/browser.rs`: playback transitions and asynchronous directory scanning.
- `src/browser_view.rs`, `src/panel.rs`: file-list drawing and the shared OpenXR quad.
- `native/media.c`: native FFmpeg demuxing, forced V4L2 hardware decoding,
  retained frame buffers and layout validation.
- `src/media.rs`, `src/playback.rs`: FFI ownership, a bounded decoder queue,
  timestamps and pause state.
- `src/graphics.rs`, `src/renderer.rs`, `src/vk_*.rs`, `shaders/`: Vulkan device,
  DMA-BUF imports, GPU ownership, color conversion and projection.
- `src/xr.rs`, `src/swapchains.rs`, `src/input.rs`: OpenXR session, stereo
  swapchains and controller actions.

The original device investigation and its probes remain under
[investigation](investigation/FINDINGS.md). `FRAME_PROBE_DEFER_DIMENSIONS` is a
compile-time C diagnostic for provisional decoder dimensions; it is disabled
in normal builds and did not resolve the current 6K allocation failure.
