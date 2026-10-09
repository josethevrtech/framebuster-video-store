# Jukebox and wood texture

Jukebox model by olesk, CC0 1.0 Universal. The owner's downloaded
Jukebox.zip supplied Jukebox.blend and BLENDSWAP_LICENSE.txt, preserved
here. The embedded license identifies the original BlendSwap record as
http://www.blendswap.com/blends/view/61319. The current author listing used
to locate the download is https://blendswap.com/blend/7373.

Dark Wood photographic texture from https://polyhaven.com/a/dark_wood,
CC0 1.0 Universal, https://polyhaven.com/license. Photography by Dimitrios
Savva, tiling by Rico Cilliers, baking by Dario Barresi.

prepare-jukebox-assets.py preserves the downloaded source file, applies
its modifiers in memory, unwraps the model, and bakes adapted wood, chrome,
gold, plastic and grille materials into 1K diffuse and ARM maps. Glass faces
are omitted from the derived opaque mesh to expose the cabinet interior.
The normal atlas is neutral; geometric normals retain the original bevels.
The native shader uses these maps for diffuse and roughness-dependent
specular lighting. jukebox-materials-v2.rgba joins diffuse, ARM, normal
maps in that order, with rows flipped for native upload.

Album art is fetched privately from the owner's Jellyfin server during
playback. It is not included in this repository or covered by CC0.
