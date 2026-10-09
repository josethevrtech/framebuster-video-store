# FrameBuster native store design

The owner's visual reference is [BingeBrowse](https://bingebrowse.net/), inspected
on 9 October 2026. Its dense cover displays, section signs, carpet, shelving and
retail ceiling guide the native store's direction. FrameBuster remains its own
Steam Frame OpenXR application, using its own generated meshes.

The observed page inventory contained scripts, styles, icons and catalogue
requests, rather than an openly licensed environment asset pack. No reusable
licence for its environment artwork was identified. Its
[catalogue credits](https://bingebrowse.net/privacy#catalogue-credits) say movie
artwork and metadata are served under a commercial TMDB licence. No BingeBrowse
code, images, paid looks or catalogue have been copied into this repository.
FrameBuster loads the owner's cover art from their Jellyfin server.

The current room assets are local Vulkan meshes: blue walls, black shelves,
decorative tape spines, patterned carpet, a fluorescent tiled
ceiling and a rental counter with a CRT-style terminal. Decorative aisle tapes
are scenery; six continuous wall bays and two double-sided browsing racks hold
up to 54 selectable Jellyfin movies, series or episodes across 180 case slots.
The room does not fetch BingeBrowse or require an external theme service.
Movie streaming and fresh Jellyfin catalogue artwork still require the server;
downloaded movie playback and a persistent offline catalogue are not implemented.

The checkout-side wall now has two 8.3-metre window banks with narrow dark
aluminium mullions and a central double-door entrance. Actual wall openings
reveal a local dusk strip-mall scene: pavement, a curb, parking stripes, lamps
and neighbouring storefronts. Sparse highlights suggest glazing without an opaque
pane blocking the view. A striped entrance mat and a physical returns cabinet
complete the entrance. These features use the room's depth-tested mesh pass,
not additional OpenXR quad layers. The exterior is scenery, not a navigable area;
dynamic reflections, sunlight and shadows are not implemented.

Twelve additional parallel black rental racks fill three rows in the enlarged
32 by 30 metre store. A shared layout supplies geometry and collision bounds;
tests check that racks do not intersect and that the center and arcade aisles
remain open. These racks are empty fixtures for now. The checkout and the
window wall moved farther back; the window banks are now 14.3 metres wide.
The original upholstered bench, kiosk, ventilation grilles and trim remain.
Only the tracked floating controllers are rendered.

A dedicated arcade corner contains three original CRT/console stations and
physical cartridge/disc-case props. Four ceiling-mounted CRTs display Jellyfin
cover art as portrait images letterboxed onto their screens. Their artwork uses
the same depth-tested pass as rental cases. The stations and game cases are
currently scenery: selecting ROMs and playing a live emulator on a CRT are
not connected yet. No emulator compatibility guarantee is implied by import.

The intended game flow is physical checkout followed by taking the game home
into a separate, small CRT room. While gaming, only that room should render;
the store should not remain loaded into the active scene pass. Returning from
the home room restores browsing. This scene transition is planned alongside
native emulator integration, rather than implemented by launching a flat app
and claiming it is a native VR CRT.

Jellyfin store music uses audio items with an integer ProductionYear from 1
through 1998. Missing years and 1999-or-newer items are excluded. The companion
loads all catalog pages, shuffles eligible tracks, authenticates the audio
stream with request headers and pipes it to the headset's installed ffplay.
No token appears in the player's arguments. Playback uses a quiet 18% volume;
background music pauses during cinema playback and resumes on return. No
floating music text is added to the room. Server reachability is required.

`steam-frame/import-retro-library.py` reads release-year metadata to prepare
an import plan, then streams selected ROMs directly from the owner's archive
to `~/Emulation/roms`. It accepts games through 1999, preserving existing files
and recording imported entries and SHA-256 hashes in private headset manifests.
Unsupported platforms and unknown dates are not assumed eligible. Nintendo 64
and PS1 still require per-game testing; Dreamcast integration is separate.

The room uses physical metre dimensions without rescaling controller tracking.
On the tested Frame, STAGE exposed a zero head height, so treating it as a
calibrated floor shrank the room incorrectly. Store sessions now use LOCAL;
the first valid head position anchors a virtual eye height of 1.65 metres.
This is a game comfort origin, not a measurement of the owner's real floor.
The room keeps its dimensions when the owner looks around or crouches.
Cases are approximately 22 by 37 cm with artwork and a plain lower border; each
black shelf bay holds six columns and three rows. The arcade carpet remains.
Wall bands sit in front of wall faces rather than on coplanar surfaces.
Scale and edge shimmer still require owner verification.

The owner confirmed the eye-relative version feels much better. The next
layout expands the floor to 20 by 22 metres without scaling people, cases,
controllers or ceiling height. Clear central space separates longer black
aisle runs. The original Halcyon shield checkout and Blender shelf decks
are baked from `public/models/` by `prepare-halcyon-fixtures.py`; their
source models remain in this GPL-3.0 fork. Upstream layouts specify feet,
so the checkout converts with 0.3048 metres per foot. The imported decks
retain their bevels and use the requested black finish. These fixtures
are embedded locally and do not require a browser.

After comparing the native eye capture with BingeBrowse, the owner rejected
the isolated three-bay arrangement. The replacement has six adjacent wall
bays and two double-sided freestanding racks, with 180 physical case slots.
Up to 54 distinct Jellyfin entries repeat through these displays, like rental
copies; each visible case selects the same catalog entry as its artwork.
One shared display-pose definition positions cases, shelf meshes, cover meshes
and pointer hit testing. Four browsing-rack ends have black physical panels
with Jellyfin poster artwork. Floating status, details, generated cover captions
and hanging text signs have been removed. Previous sign asset files are preserved
but are not loaded. Account pairing uses a native Linux dialog outside the room.

Artwork upload flips bottom-up canvas rows into Vulkan image rows. Cover UVs
therefore use the atlas height minus each artwork's bottom offset. The same
catalog index drives both case selection and UVs; tests cover all 180 case
instances. Previously, OpenXR cover quads always composited over the projection
and showed through fixtures. They are now sampled by a Vulkan cover pipeline
inside the same depth-tested room pass. All shelf and rack-end artwork is part
of the stereo projection; no additional artwork composition layers are needed.
Headset confirmation of the new depth behavior is pending.

Offline furniture includes a vintage TV, potted plant and cushioned chair from
[Kenney Furniture Kit](https://kenney.nl/assets/furniture-kit), under **CC0**.
Original OBJ/MTL source and licence are preserved in `native/matineevr/assets/`.
`steam-frame/prepare-store-props.py` bakes material colours and placement into
`kenney-furniture.bin` with Python's standard library. The runtime embeds this
mesh; no model downloads or converter are needed at launch.

This remains a prototype environment. Grabbable cases, better text and materials,
configurable floor/comfort settings
and an offline media library remain work toward the full video-store experience.
