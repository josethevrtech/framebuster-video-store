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
Connect when needed, and fetches 54 movies and series per shelf page. Series
selection opens paged episode shelves through Jellyfin's Show API. A private native
device account survives application restarts. Session catalog snapshots remain
on disk for diagnostics; they contain titles and artwork, not credentials.

The companion writes bounded binary catalog records and status text. Rust checks
record counts, text lengths, image dimensions and payload lengths before upload.
54 covers with title captions share one transparent atlas and swapchain.
Three atlas regions repeat across six wall bays and four freestanding rack faces.
Display poses are shared with geometry and pointer selection. Status, framed
Jellyfin posters, the section sign and shelf-mounted details use at most fifteen
quad layers plus the projection layer. The room and
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

Right stick pages shelves; Y searches. Left stick provides bounded steps of
0.6 metres times the room scale,
steps and 30-degree snap turns on stick edges, without continuously sliding the
view. Trigger opens details; A plays/resumes or opens a series' episodes. B closes
details, stops playback, or returns from episodes to the main shelves.
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
loading are verified. The owner also verified the shared artwork atlas and
native shelf-to-movie playback on the headset.

The owner has now verified posters and native shelf-to-Jellyfin playback.
The shop has expanded floor space, side aisles, a checkout counter and ceiling
fixtures. B closes details at the shelves and stops playback without exiting
the store session; Steam's own menu provides app exit. The expanded shop and
the corrected return behavior still need headset verification.

The latest store pass adds a shared fixture scale, STAGE floor reference,
separated wall trim, retro signs, tape spines, carpet and a CRT rental terminal.
See [store design and asset provenance](store-design.md) for the BingeBrowse
reference, offline environment assets, current limitations and visual checks.

The app is not a complete release: subtitle selection, Unicode text, a
polished VR keyboard, configurable comfort options, stronger account storage
integration, bounded cache retention and automated runtime packaging remain.
