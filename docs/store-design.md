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

The checkout-side wall now has two 6.3-metre window banks with narrow dark
aluminium mullions and a central double-door entrance. Actual wall openings
reveal a local dusk strip-mall scene: pavement, a curb, parking stripes, lamps
and neighbouring storefronts. Sparse highlights suggest glazing without an opaque
pane blocking the view. A striped entrance mat and a physical returns cabinet
complete the entrance. These features use the room's depth-tested mesh pass,
not additional OpenXR quad layers. The exterior is scenery, not a navigable area;
dynamic reflections, sunlight and shadows are not implemented.

Six additional double-sided black rental racks fill the rear aisles; these
new racks are empty display fixtures for now. Their shared layout definitions
also supply locomotion collision and grip-surface bounds. Existing selectable
cases and rack-end posters continue to use the owner's Jellyfin artwork.
External decorative movie catalogs are not yet connected.

The first body-presence pass renders a torso and two-segment arms estimated
from the tracked headset and controllers. Reachable elbow poses use two-bone
inverse kinematics; longer reaches stretch to keep wrists at the controllers.
This is not full-body tracking or a reproduction of Lone Echo's body system.
Legs, foot planting, calibrated arm lengths, torso yaw filtering and physical
case grabbing remain future work. Grip locomotion stays on the floor rather
than introducing zero-gravity movement into the shop.

The room uses physical metre dimensions without rescaling controller tracking.
On the tested Frame, STAGE exposed a zero head height, so treating it as a
calibrated floor shrank the room incorrectly. Store sessions now use LOCAL;
the first valid head position anchors a virtual eye height of 1.65 metres.
This is a game comfort origin, not a measurement of the owner's real floor.
The room keeps its dimensions when the owner looks around or crouches.
Cases are approximately 22 by 37 cm including their title captions; each
black shelf bay holds six columns and three rows. The arcade carpet remains.
Wall bands sit in front of wall faces rather than on coplanar surfaces.
Scale and edge shimmer still require owner verification.

The owner confirmed the eye-relative version feels much better. The next
layout expands the floor to 16 by 18 metres without scaling people, cases,
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
One shared display-pose definition positions cases, shelf meshes, cover quads
and pointer hit testing. Four browsing-rack ends have black physical panels
with Jellyfin poster artwork. Floating status, details, generated cover captions
and hanging text signs have been removed. Previous sign asset files are preserved
but are not loaded. Account pairing uses a native Linux dialog outside the room.

Artwork upload flips bottom-up canvas rows into Vulkan image rows. Region
selection must therefore use `atlas_height - bottom - region_height`; the
previous offset selected another bank's titles. A regression test covers the
three region offsets. The store uses up to 15 total composition layers, below the observed limit of 16. Native visual verification of
this complete display arrangement is still pending.

Offline furniture includes a vintage TV, potted plant and cushioned chair from
[Kenney Furniture Kit](https://kenney.nl/assets/furniture-kit), under **CC0**.
Original OBJ/MTL source and licence are preserved in `native/matineevr/assets/`.
`steam-frame/prepare-store-props.py` bakes material colours and placement into
`kenney-furniture.bin` with Python's standard library. The runtime embeds this
mesh; no model downloads or converter are needed at launch.

This remains a prototype environment. Grabbable cases, better text and materials,
configurable floor/comfort settings
and an offline media library remain work toward the full video-store experience.
