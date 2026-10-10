# Retail fixtures and material atlas

This work is based on **Panasonic SA-PM02 Stereo System** by **AleixoAlonso**:
https://sketchfab.com/3d-models/panasonic-sa-pm02-stereo-system-e6b0d376c49d4d4c9ce7c1502661c8e4
Artist: https://sketchfab.com/AleixoAlonso
License: **CC BY 4.0**, https://creativecommons.org/licenses/by/4.0/

The derived stereo geometry and PBR textures in `../retail-models.bin` and
`../retail-materials.rgba` retain that license. Adaptations: placed at metre
scale, UVs repacked, diffuse/roughness/metallic/normal maps baked at 1024 pixels,
and the transmissive LCD cover flattened to opaque display artwork for the
native opaque material shader. Retrieved 2026-10-09. The primary publisher API
license is recorded in `stereo-source.json`. Original archive is preserved in
the local asset preparation directory; the public download mirror is
https://mirror.traines.eu/sketchfab-backup/e6/e6b0d376c49d4d4c9ce7c1502661c8e4.zip

The music cabinet is **Modern Wooden Cabinet** by **Patrik Pangerl**,
https://polyhaven.com/a/modern_wooden_cabinet, **CC0 1.0**:
https://creativecommons.org/publicdomain/zero/1.0/
Original glTF, creator metadata and textures are retained in `../lounge/`.
It is placed at its original metre scale, with repacked UVs and baked maps.

The counter, terminal and keyboard geometry are original Halcyon assets under
the repository **GPL-3.0** license. Sources: `public/models/rental-terminal.glb`,
`public/models/rental-keyboard.glb`,
`public/models/checkout-counter-shield-rounded.glb`, and their editable sources
in `tools/models/`. The native adaptation places two angled stations on the
2.82-foot staff worktop, reroutes the low rear cables onto it and bakes the
surface maps. The CRT glass is opaque dark green in the native material pass.

Counter laminate uses the existing **ambientCG Plastic013A** photographic scan,
https://ambientcg.com/view?id=Plastic013A, **CC0 1.0**. Original maps and license
are in `public/textures/surfaces/store-shelf/`. Counter finishes retain the
original cream laminate, blue rolled edge, yellow inlay and recessed joinery.
Historical rental-store photos inform proportions and placement; no reference
photographs or store logos are included in these runtime assets.

Generated with `steam-frame/prepare-retail-assets.py` and
`steam-frame/pack-retail-materials.py`. Runtime assets are fully offline.
