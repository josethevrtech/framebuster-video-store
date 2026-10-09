# FrameBuster native store design

The owner's visual reference is [BingeBrowse](https://bingebrowse.net/), inspected
on 9 October 2026. Its dense cover displays, section signs, carpet, shelving and
retail ceiling guide the native store's direction. FrameBuster remains its own
Steam Frame OpenXR application, using its own generated meshes and lettering.

The observed page inventory contained scripts, styles, icons and catalogue
requests, rather than an openly licensed environment asset pack. No reusable
licence for its environment artwork was identified. Its
[catalogue credits](https://bingebrowse.net/privacy#catalogue-credits) say movie
artwork and metadata are served under a commercial TMDB licence. No BingeBrowse
code, images, paid looks or catalogue have been copied into this repository.
FrameBuster loads the owner's cover art from their Jellyfin server.

The current room assets are local Vulkan meshes: blue and yellow store signs,
labelled aisles, decorative tape spines, patterned carpet, a fluorescent tiled
ceiling and a rental counter with a CRT-style terminal. Decorative aisle tapes
are scenery; three front/side cover bays hold up to 54 selectable Jellyfin
movies, series or episodes. Details stay at the selected shelf bay.
The room does not fetch BingeBrowse or require an external theme service.
Movie streaming and fresh Jellyfin catalogue artwork still require the server;
downloaded movie playback and a persistent offline catalogue are not implemented.

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

Offline furniture includes a vintage TV, potted plant and cushioned chair from
[Kenney Furniture Kit](https://kenney.nl/assets/furniture-kit), under **CC0**.
Original OBJ/MTL source and licence are preserved in `native/matineevr/assets/`.
`steam-frame/prepare-store-props.py` bakes material colours and placement into
`kenney-furniture.bin` with Python's standard library. The runtime embeds this
mesh; no model downloads or converter are needed at launch.

This remains a prototype environment. Grabbable cases, better text and materials,
configurable floor/comfort settings
and an offline media library remain work toward the full video-store experience.
