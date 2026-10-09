# Halcyon Frame native playback

The `matineevr` directory is adapted from the source archive distributed
with MatineeVR 0.0.10 by embedding-shapes. Original GPL-3.0-only licensing
and source notices are retained. Download and project information:
https://embedding-shapes.itch.io/matineevr.

The reference ARM64 package plays the local H.264/AAC sample directly on
Steam Frame through OpenXR, Vulkan and Iris hardware decoding. The owner
confirmed sound, functional controls and correct picture after selecting
flat projection and mono stereo mode explicitly.

Halcyon renders Steam's actual Frame controller meshes at tracked grip poses,
with lighting, vertex colors sampled from the installed textures, and depth
testing. They disappear when controller poses are invalid or the session loses
focus. The owner confirmed that the 3D models look good on the headset.
The prior blue/amber icon binary remains available separately.

`steam-frame/prepare-controller-models.py` reads the models already installed
by SteamVR, converts their grip offsets and prepares a local mesh cache. Valve's
controller assets are not redistributed in this repository. Prepare that cache
as `native-player/controller-mesh.bin` in the installation root before launching
the player. `HALCYON_FRAME_CONTROLLER_MESH` selects its location.

The release build and hardware H.264/Vulkan render probe passed on the Frame.
HTTP MP4 and HLS hardware render probes also passed. The owner verified a real
Jellyfin movie handoff after decoder URL handling was corrected.

## Native store foundation

`store_geometry.rs` and `store_scene.rs` add a native room with shelves and six
test cases, using the existing Vulkan mesh pipeline. OpenXR aim poses drive
visible controller pointers and case hover outlines. Trigger selection starts
the local sample in the same OpenXR session; B during playback or movie completion
returns to the room. B in the room exits the prototype. A also starts the sample.
Dynamic pointer buffers use the existing frame fences before writes, and room
geometry and controllers share the same depth pass.

`store-launch.sh` starts `native-player/halcyon-frame-player-store` through Steam.
`register-native.py` registers “Devkit Game: HalcyonFrameStore” alongside the
working library and cinema entries. The owner verified the room, controllers,
trigger selection, sample playback and return to the room on the headset.
This is a native interaction prototype: the six cases are sample placeholders,
not Jellyfin titles. Catalog artwork, details, search and comfort locomotion
are not implemented in this native room yet. The ordinary library retains
verified Jellyfin browsing and playback while that native interface is built.

Building on the headset requires Rust, glslc, a working C compiler, FFmpeg
and PulseAudio development headers. The upstream build uses a
`steam-frame-cc` compiler wrapper; the ARM64 linker can be overridden with
Cargo configuration when building natively. `frame-build/build.sh` provides
the native Clang wrappers, including explicit ARM64 loader and atomic support
needed by this SteamOS toolchain. Put Rust's `bin` directory on PATH and mark
the wrapper scripts executable before running it. The original toolchain
scripts describe the upstream cross-build process.

Keep the reference player available while validating the adapted binary.
The launcher uses `native-player/halcyon-frame-player-library`; the original,
icon and controller-model versions are preserved separately. The library bridge
passes an opaque localhost media URL and an initial resume position. The player
writes position, pause and completion snapshots without credentials; the runner
reports them to the local library server. B and natural completion exit the
cinema helper. The relay and native URL restrictions have automated coverage,
and a real Jellyfin movie handoff was verified by the owner.
