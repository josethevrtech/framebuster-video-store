# FrameBuster native architecture

FrameBuster targets Steam Frame's ARM64 SteamOS runtime. Steam launches the
OpenXR scene; Rust owns the room, controllers, pointer selection, artwork panels,
movie details and video rendering. A headless Node process owns network access,
Jellyfin authentication and catalog/progress work. Chrome, Vite, the Halcyon web
server and browser credentials are not required by this launch path.

## Application lifetime

`framebuster-launch.sh` loads optional server configuration and invokes the
Steam OpenXR prelaunch wrapper. `native-store-runner.mjs` creates a private
session directory, starts `framebuster-video-store`, pairs through Jellyfin Quick
Connect when needed, and fetches six movies per shelf page. A private native
device account survives application restarts. Session catalog snapshots remain
on disk for diagnostics; they contain titles and artwork, not credentials.

The companion writes bounded binary catalog records and status text. Rust checks
record counts, text lengths, image dimensions and payload lengths before upload.
Six covers with title captions are packed into a single transparent shelf atlas,
reducing OpenXR swapchain/resource use. Status, shelf artwork and details use
at most three quad layers in addition to the projection layer. The room and
tracked Valve controller meshes share the depth-tested Vulkan mesh pass.

Native selection writes a numbered command. The companion prepares Jellyfin
H.264/AAC HLS and an opaque localhost media URL. Rust opens it without treating
it as a filesystem path and remains in the same OpenXR session. B or completion
returns to the room. Position/pause snapshots drive Jellyfin session reports;
saved Jellyfin playback position is applied on resume.

The movie relay admits only the configured server and selected movie, rejects
redirects, strips token query parameters, authenticates upstream with a header,
rewrites HLS segment URLs and handles byte ranges. Native decoder arguments
contain only the opaque local URL. Authentication stays in the companion.

## Native Linux integration

Zenity supplies server-address and search dialogs. Artwork conversion uses the
system FFmpeg with inherited decoder/preload libraries cleared: the cinema's
patched FFmpeg codec library is incompatible with the system image converter.
The cinema retains its Iris decoder library and Turnip Vulkan driver settings.

Right stick pages shelves; Y searches. Left stick provides bounded 0.6-metre
steps and 30-degree snap turns on stick edges, without continuously sliding the
view. Movie details open with a trigger; A plays/resumes and B closes details.
The original library/cinema/store-test entries remain separate development
fallbacks. The primary entry is `Devkit Game: FrameBusterVideoStore`.

## Installation and verification

Build natively with `native/frame-build/build.sh`, then place its release
`framebuster-video-store` in `native-player/`. The current runtime payload also
needs `runtime/bin/node`, the patched `matineevr/ffmpeg` codec library and
`matineevr/mesa` Turnip driver, SteamVR, system FFmpeg, Python and Zenity.
`steam-frame/install-native.sh` validates the native payload, prepares controller
meshes from installed Valve files when needed, and registers only FrameBuster.
It does not require `dist/`, Chrome or the Halcyon local server.

Release builds, restricted-URL and selection tests, malformed catalog parsing,
comfort movement and poster composition checks cover the native changes. Node
tests cover token headers, catalog bounds, opaque movie requests and progress
reporting. Hardware HTTP/HLS render probes and the earlier real Jellyfin cinema
handoff passed on the owner's Frame. Native Quick Connect sign-in and catalog
loading are verified; the shared artwork atlas and complete native shelf-to-movie
flow still need visual verification on the headset.

The app is not a complete release: TV episodes, subtitles, Unicode text, a
polished VR keyboard, configurable comfort options, stronger account storage
integration, bounded cache retention and automated runtime packaging remain.
