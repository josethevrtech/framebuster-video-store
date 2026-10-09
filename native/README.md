# Halcyon Frame native playback

The `matineevr` directory is adapted from the source archive distributed
with MatineeVR 0.0.10 by embedding-shapes. Original GPL-3.0-only licensing
and source notices are retained. Download and project information:
https://embedding-shapes.itch.io/matineevr.

The reference ARM64 package plays the local H.264/AAC sample directly on
Steam Frame through OpenXR, Vulkan and Iris hardware decoding. The owner
confirmed sound, functional controls and correct picture after selecting
flat projection and mono stereo mode explicitly.

Halcyon's additions currently provide blue/amber controller icons at valid
tracked grip positions. They are billboard icons, not controller meshes.
They disappear when controller poses are invalid or the session loses focus.
The release build and hardware H.264/Vulkan render probe passed on the Frame.
The adapted binary is installed, and the owner confirmed both icons are visible
while the movie continues to display correctly. Replacing these icons with
Steam's actual controller models is the next graphics change.

Building on the headset requires Rust, glslc, a working C compiler, FFmpeg
and PulseAudio development headers. The upstream build uses a
`steam-frame-cc` compiler wrapper; the ARM64 linker can be overridden with
Cargo configuration when building natively. `frame-build/build.sh` provides
the native Clang wrappers, including explicit ARM64 loader and atomic support
needed by this SteamOS toolchain. Put Rust's `bin` directory on PATH and mark
the wrapper scripts executable before running it. The original toolchain
scripts describe the upstream cross-build process.

Keep the reference player available while validating the adapted binary.
Do not replace its executable until the release build and playback checks
pass. The library bridge and playback progress integration are unfinished.
