# 0.0.10 - Vulkan, FishEye, Flat 3D movies, 8K HEVC and faster seeking

**Features**

- OpenGL/GLES rendering has been replaced by Vulkan, improving performance a whole bunch
    - CPU now sits around 20% and GPU around 30% when playing back reference 8K HEVC Main10 video
- 8K HEVC Main10 videos are now supported! HDR still remains unsupported
- Added fisheye projection with 180°/190° presets, adjustable field of view, separate image centres, and lens-distortion controls
- Support for more flat 3D movies. Added Full/Half formats for side-by-side and top/bottom video, plus non-square-pixel correction.
- Added audio/video read-ahead buffering with a one-second target, intended to help with storage/network stalls

**Fixes**

- Playback timing and audio continuity fixes, like stale audio-clock handling and reducing skipped frames
- Reduced seek-preview latency from ~200ms > ~160ms in our 8K benchmark

# 0.0.9 - Easier on-Frame installs

- Added "install.py" to end-user package for easier on-Frame installs

# 0.0.8 - UX and wider playback support

**Features**

- Play 8-bit VP9 videos
- Files with an unsupported preferred video track can now use a supported alternate track
- Video projection, stereo layout and eye order are detected where possible
- Format and alignment settings are remembered per video until the app exits
- Video settings now separate projection from 180°/360° format and include Reset
- Adjust video alignment with the triggers:
  - Hold left trigger:
      - Left stick: yaw (left/right), pitch (up/down)
      - Right stick: roll (left/right), zoom (up/down)
  - Hold right trigger:
      - Left stick: stereo offset (left/right and up/down)
      - Right stick: screen or panorama position (left/right and up/down)
  - Press A while holding either trigger to reset alignment
- Hold right-stick or D-pad left/right to repeat and accelerate 10-second skips
- Seeking now shows a progress bar and an approximate preview thumbnail that usually loads faster than first real frame

**Fixes**

- Reduced jitter in controller panels
- Failed file inspections are retried on later folder visits
- Fixed an end-of-video decoder error
- Skipping past the start or end no longer triggers unnecessary seeks

# 0.0.7 - Audio Support

- We now support playback of audio as well, of course :)

# 0.0.6 - Seeking and better browsing

- Skip backward or forward 10 seconds with the right stick or D-pad
- Videos now default to 180° side-by-side stereo.
- Folders scan faster, with cached results when revisiting directories
- The file browser shows more entries
- Hold D-pad up/down to accelerate scrolling
- Returning to a folder restores your selection during the app session.
- B goes up a folder in the standalone browser instead of exiting during playback. B still stops a playing video.
- Launching from SteamOS gives the application full focus now and doesn't start paused anymore.

# 0.0.5 - Public build

First version being made public-public, and also slapped a GPLv3 license on it, so the community can learn from the code if wanted. Enjoy :)

# 0.0.4 - Published to Itch

This is the first version being published to Itch.io and to the public.

# 0.0.3 - Better UX
# 0.0.2 - Testing deploys
# 0.0.1 - Initial Version
