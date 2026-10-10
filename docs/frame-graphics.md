# Native Frame graphics polish

The first material pass remains forward rendering, with one draw per material
atlas. The shader uses GGX specular distribution, Schlick Fresnel and Smith
masking, roughness/metallic maps, an orthogonal normal-map basis, hemisphere
ambient light and a restrained ceiling reflection approximation. Lighting is
defined in store coordinates, so turning the player does not rotate the lights.
The render target is sRGB; the material shader outputs linear light.

Fixture cavity shading is baked offline into the red channel of the existing
AO/roughness/metallic atlas. This adds no new runtime texture fetch or pass.
It provides local material occlusion, not complete scene shadows. The ceiling
reflection is a lighting approximation, not a mirror or ray-traced reflection.

Khronos recommends forward rendering for small light counts or baked lighting:
https://docs.vulkan.org/tutorial/latest/Building_a_Simple_Engine/Advanced_Topics/Forward_ForwardPlus_Deferred.html
Google's Filament documents physically based material models:
https://google.github.io/filament/main/filament.html
Mobile GPUs require particular attention to external-memory bandwidth and
render-pass synchronization:
https://github.com/KhronosGroup/Vulkan-Samples/tree/main/samples/performance/pipeline_barriers

Next worthwhile work is scene lightmaps with contact shadows, diffuse light
probes for controllers and furniture, and prefiltered reflection probes for
CRT glass and laminate. These can be baked offline, with no per-frame ray
tracing. Mipmaps and appropriate texture compression should follow artwork
quality checks. Bloom would be restrained and limited to fluorescent fixtures
and CRTs, with any extra stereo pass measured before inclusion. Full-screen
SSAO, many dynamic shadow lights and screen-space reflections are not enabled
in this pass; their added buffers, bandwidth and frame-time cost need profiling.

With --stats, native draw timing now reports scope=store_gpu, including
gpu_draw_ms and gpu_fence_wait_ms. Each GPU draw sample covers one eye's room,
artwork, controllers and fixtures. It excludes the separate video-background
pass and the compositor. OpenXR period and work timing remain separate. A
72-Hz frame has 13.89 ms; a 90-Hz frame has 11.11 ms. Device measurements must
include both eyes, loaded shelves and trailers in the visible, focused store.
Dashboard-covered or sleeping runs cannot establish rendering performance.

Initial focused-device samples on 2026-10-09, after all 27 game bays and
22 album sections loaded, measured 1.52–1.57 ms mean store draw time per
eye across two five-second windows (maximum 2.90 ms). The OpenXR period was
13.889 ms for one window and 13.928 ms for the next, with isolated CPU/work
and submission spikes. This is a short observation at one viewing position,
not a sustained performance guarantee or a before/after comparison. Full
scene contact shadows and bloom remain future work.

Layout source of truth is steam-frame/room-layout.json. Offline asset preparation
exports the native collision rectangles, TV and button positions and speaker
locations. The oversized CD stereo uses scale 2.7; its case artwork is placed
5 mm in front of the physical cover surface to prevent coplanar flicker.
