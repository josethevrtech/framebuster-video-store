# FrameBuster Video Store

A native video store and cinema for **Steam Frame**, built around **OpenXR, Vulkan and the Frame controllers**. Browse your Jellyfin collection on shelves, select cases, and watch in VR in the same native application. The runtime runs entirely on the headset; your Jellyfin server must remain reachable.

This is an actively tested prototype, not a finished release. Native room rendering, controller pointers, sample selection, return from cinema, and the earlier real Jellyfin cinema handoff are headset-verified. The new browser-free account/catalog interface is undergoing headset verification.

## Native application

- Rust renders the room, tracked controllers, shelf artwork and cinema through SteamVR OpenXR/Vulkan.
- A headless Node companion handles Jellyfin device sign-in, catalog/artwork loading, restricted localhost media relay, resume and progress reporting. It opens no web interface.
- Jellyfin Quick Connect pairs the app without putting passwords into the VR UI. Its device account is saved in a private local file.
- FFmpeg uses the Frame's Iris hardware video decoder, with H.264/AAC HLS requested for Jellyfin playback.
- Native Linux dialogs provide server-address entry, Quick Connect pairing and title search. The signed-in runtime does not require Chrome or a development server.

Launch **Devkit Game: FrameBusterVideoStore** in Steam. Trigger selects a case; A plays, resumes, or opens a series' episodes. B returns from a movie, clears selection, or returns from episodes to the main store. Exit through Steam's menu. Right-stick up/down changes pages; left/right makes 30-degree snap turns about the headset. Y opens native search. Left stick walks smoothly in the direction you face. X switches between stick walking and surface-grip movement: squeeze near a rack or counter and pull yourself along, with a short damped coast on release. The room preserves physical scale and anchors its starting eye height to the headset pose.

The room has no floating status, details, section sign or generated case captions. Four browsing-rack ends display Jellyfin posters on black panels. The store shows up to 54 movies/series or episodes across a continuous wall and double-sided racks. Posters and movie playback are owner-verified; the corrected eye-relative scale is owner-verified; the latest continuous display layout, atlas correction and episode navigation need headset verification. Unicode text rendering, subtitle selection, a polished VR keyboard, configurable comfort controls and a complete installer remain unfinished. Valve controller assets are prepared from installed SteamVR files and are not redistributed. Offline shop props include CC0 Kenney furniture; see [store design and asset credits](docs/store-design.md).

## Source and development

Native source: [native/matineevr](native/matineevr). The directory retains the original project's name for provenance. [Native build instructions](native/README.md) describe the ARM64 toolchain and current validation.

The companion and Steam launchers live in [steam-frame](steam-frame). The native launch path is `framebuster-launch.sh` → Steam OpenXR prelaunch → `native-store-runner.mjs` → `framebuster-video-store`. `framebuster.env` can set `FRAMEBUSTER_JELLYFIN_URL`; the app can also request a server address through a native dialog.

The earlier Halcyon browser code remains as a reference and fallback while native features are built. It is not the primary interface for FrameBuster. Its original documentation is preserved in [the upstream README](docs/halcyon-upstream-readme.md); upstream features should not be assumed to exist in the native app.

## Credits and licensing

FrameBuster is derived from [Halcyon Video](https://github.com/halcyon-video/halcyon-video) and the GPL-3.0-only source distributed with [MatineeVR 0.0.10 by embedding-shapes](https://embedding-shapes.itch.io/matineevr). Their licenses and source notices are retained. Steam Frame and SteamVR are Valve products; this is an independent community project.
