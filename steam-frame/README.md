# Halcyon Frame standalone prototype

This fork targets rendering and serving Halcyon entirely on Steam Frame.
It is a prototype under active testing on the owner's headset. Native
OpenXR/Vulkan playback through MatineeVR has been verified with an H.264/AAC
sample: video, sound, correct 16:9 mono presentation and button controls.
The Jellyfin library bridge is implemented and installed for headset testing;
an actual Jellyfin movie handoff has not yet been verified.

## Runtime

The built `dist/` folder is served on `127.0.0.1` by `steam-frame/server.mjs`.
The server uses only Node's built-in modules and the existing integration
policy helpers. No development server, npm install, PC, root access or
system service is needed at launch. Node 22.18 or newer is required, either
on PATH or as a compatible Linux ARM64 executable in `runtime/bin/node`.
The browser can be selected with `HALCYON_FRAME_BROWSER` (an executable
path, not a command with arguments). The dedicated Chrome Dev wrapper in
`browser.sh` disables the seccomp filter sandbox to allow SteamVR access.
It uses a separate profile. The native player does not require that browser
exception.

Build the repository using its normal `npm ci` and `npm run build` workflow,
then run `bash steam-frame/launch.sh` on the headset. The launcher opens a
regular library with native playback enabled by the `frame=1` query parameter.
It starts a persistent user service for the local server. Add the
launcher to Steam as a non-Steam application to launch from the library;
launch the native player through Steam so Steam supplies its VR environment.

Set `HALCYON_FRAME_NATIVE=1` and `HALCYON_JELLYFIN_URL` in the local server's
environment to enable Jellyfin handoff. `register-native.py` registers the
library and cinema helper. “Watch in VR” launches the helper automatically.
The separate Cinema entry can also play the diagnostic sample directly.
The browser wrapper clears Steam's injected graphics layers and preload
libraries before starting Chrome; those injections crashed its GPU process.
The library uses ANGLE SwiftShader software rendering because the hardware
browser renderer produced corrupted squares on this headset. Native cinema
playback continues to use Vulkan and hardware video decoding.

The bridge keeps Jellyfin credentials in memory and relays only the selected
movie through an opaque localhost URL. Native playback reports position and
pause state to the library for resume and progress reporting. HLS playlists
and segments use the same restricted relay. Jellyfin HLS requests select
H.264/AAC for the native player. A server restart interrupts active playback.

Library connections continue to use Halcyon's Jellyfin, Plex and Emby
clients. Those media servers must still be reachable on the network.
This does not require a PC to serve the Halcyon app, but does not turn an
existing media server into local headset storage. Local-file library
scanning is not implemented. Subscriptions remain provider link-outs.

## VR changes and remaining work

- Convert WebXR metre-based tracking to the store's feet-based dimensions,
  including controller poses, clipping distances and carried-case sizes.
- Retain existing thumbstick walking, snap turning and trigger selection.
- Retain upstream checkout behavior: it exits VR before playback.
- The WebGL1 tracking cube works on the headset. The experimental browser
  cinema currently produces a black picture despite audio and tracking;
  use the native player for playback.
- Native controls: X pauses, A recenters, B exits playback, right stick seeks,
  and grips show shortcut panels. The cinema helper exits when playback ends.
- Native Steam Frame 3D controller models passed the release build and owner
  visual verification. The original and icon binaries are preserved separately.
- Library playback and resume need real-server headset verification. Native
  subtitle selection, store comfort and modern pointing controls need more
  implementation and testing. This is not a
  complete standalone release yet.

## Native player source

`native/matineevr` contains the GPL-3.0-only source supplied with MatineeVR
0.0.10 by embedding-shapes: https://embedding-shapes.itch.io/matineevr.
Its original license and attribution are retained. This fork adds tracked
Steam Frame controller meshes using Vulkan. Prepare the headset's installed
model assets with `prepare-controller-models.py`; they are not bundled here.
Launch ordinary movies
with `--projection flat --stereo mono`; the upstream defaults are VR180 SBS.

Local verification: `node --test tests/frame-server.test.ts tests/frame-native.test.ts tests/vr-units.test.ts`.

Official platform setup: https://partner.steamgames.com/doc/steamhardware/steamframe/setup
