# FrameBuster Steam Frame runtime

The primary application is now **FrameBuster Video Store**, a native OpenXR/Vulkan
store and cinema with its own Jellyfin companion. Launch
`Devkit Game: FrameBusterVideoStore` in Steam. It does not require Chrome,
the browser build, or the Halcyon web server. See
[native architecture and installation](../docs/native-architecture.md) for the
current runtime, controls, requirements, and verification status.

The sections below document the earlier Halcyon browser prototype, retained
as a development fallback.

This fork targets rendering and serving Halcyon entirely on Steam Frame.
It is a prototype under active testing on the owner's headset. Native
OpenXR/Vulkan playback through MatineeVR has been verified with an H.264/AAC
sample: video, sound, correct 16:9 mono presentation and button controls.
The owner verified Jellyfin sign-in, library browsing and a real movie handoff
to the native cinema. A separate native 3D store room prototype is also verified.

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
The Frame library boots directly into the HTML shelf view and disables browser
GPU rendering because the hardware renderer produced corrupted squares on
this headset. It does not require WebGL. Native cinema
playback continues to use Vulkan and hardware video decoding.
The Frame profile fixes the effective render mode to the shelf view and omits
the 3D store menu option. Library browsing and Jellyfin sign-in are owner-verified;
the browser's 3D store is not supported by this profile.

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
- Library playback passed real-server headset verification; resume and progress
  still need further checks. Native
  subtitle selection, store comfort and modern pointing controls need more
  implementation and testing. This is not a
  complete standalone release yet.

The new native store prototype is “Devkit Game: HalcyonFrameStore”. It renders
its room and shelves directly through Vulkan/OpenXR, with Frame controller aim
pointers and trigger selection. Its six test cases play the local sample;
B returns from the movie to the room without changing apps. Jellyfin cases,
artwork, search and comfort locomotion remain future work in this native room.
The browser's 3D store remains disabled on Frame.

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


## Current native store and music

See the repository README for the current FrameBuster runtime; the historical
browser experiments above are preserved as development records. The enlarged
native store has a CRT corner and hanging TVs. Store music uses Jellyfin audio
items with ProductionYear before 1999 and pauses while watching a movie.
The headset must have its native `ffplay` available. The native companion never
passes Jellyfin tokens in audio-player command arguments.

## Private retro-library import

`import-retro-library.py` uses only Python's standard library and SSH. Supply
`--source` for the owner's ZIP collection, `--metadata` for Libretro's
`metadat/releaseyear` DAT files, and `--plan` for a new JSON plan path. Review
that plan before adding `--copy --key <SSH key>`. The PS1 collection's shared
ScreenScraper gamelist is used across both archive halves. Unmatched release
years are skipped; titles from 1999 are eligible, while music's cutoff excludes
1999. Imported files and hashes are recorded under the headset's private
`~/.local/share/halcyon-frame/game-library` directory. Existing data is verified
and preserved; matching partial transfers can resume by appending only the
remaining bytes. A conflicting file fails without overwriting it.

The importer prepares a library, not a compatibility certification. Actual
native CRT game playback, N64/Dreamcast performance testing and the separate
checked-out/home room still require emulator integration. ROMs, BIOS files and
private import plans are not committed to this public repository.

## Shared game lounge and ceiling trailers

The gaming area now has one textured CRT, a wood media cabinet, two vintage
sofas and a coffee table. Furniture is CC0 Poly Haven;
source assets and attribution are retained in native/matineevr/assets/lounge.
The cabinet's five physical input buttons swap licensed textured console meshes:
NES, SNES, Genesis, PS1 and N64, left to right.
Point and squeeze the trigger; the active input has a green indicator.
This selects the physical display model. Emulator sessions and playable ROM
selection on this shared screen remain separate integration work.

The right game wing extends the room from 32 to 44 metres wide while retaining
physical scale, floor height and ceiling height. It has a separate teal carpet,
a hanging printed VIDEO GAMES board, and 32 black shelf bays with printed console
headers. Every imported game has one boxed cover in its console section; the
current private library occupies 27 bays with 1,043 games across eleven systems.
There is no game shelf pagination or filtering. Cabinet inputs change only the
console on the lounge cabinet. Game covers are display items at this stage,
not playable selection controls. The former standalone computer kiosk is gone
from the room; its computer and keyboard now rest on the checkout counter.

Each bay has at most 54 covers with a stable game identity and its own artwork
atlas, independent of movies and music. The native renderer loads one bay per
250ms while the store is active, then keeps all loaded covers visible. This
avoids one large startup upload and allows full-library console sections. The
manifest reader validates section allocation, capacity and private file paths.

`prepare-game-art.py` builds a private catalog and cover cache from an import
plan, using exact ROM names or unambiguous regional title matches from the
Libretro thumbnail collections. Place its catalog.json and artwork directory
inside the headset's private game-library directory. prepare-rental-covers.py
adds original store-branded sleeves with titles and years for unmatched covers,
preserving every existing artwork file. import-private-artwork.py safely copies
an artwork ZIP into that private library and rejects conflicting existing files.
The installed library has 946 matched box-art images and 97 original rental
sleeves. ROMs, game-cover caches and source plans remain private and are not
redistributed with the application. Console meshes for the other six systems
and live game playback still need integration.

Hardware attribution is recorded in assets/rental-hardware/README.md. Download
source archives with download-hardware-assets.py, bake with Blender using
prepare-hardware-assets.py and prepare-cartridge-assets.py, then pack the maps
with pack-hardware-materials.py (append `cartridges` for the cartridge atlas).

The four hanging CRTs share a muted 512x384, 12-fps trailer feed. Candidates
come only from Jellyfin movies/series, preferring local trailers and otherwise
using their RemoteTrailers YouTube links. Run install-trailers.sh to install
the pinned official yt-dlp Python ZIP utility; Python 3, FFmpeg and the bundled
Node runtime are required. yt-dlp uses its official EJS helper through GitHub.
These are streamed native video textures, with no browser interface, website
overlays, downloaded trailer collection or committed private media. Network
trailers require internet access; unavailable sources are skipped. Local
trailers require the Jellyfin server. Decoding pauses during cinema playback.
The jukebox now faces inward from the entrance wall, and its album art and
physical controls use the same transformed position as its model.
