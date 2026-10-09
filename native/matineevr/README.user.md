# MatineeVR

An experimental, open-source VR video player built for standalone playback on
Steam Frame, using hardware decoding and GPU rendering.

Play local **H.264, HEVC and VP9** videos on Steam Frame.
Supports HEVC Main10 through 8K and adjustable fisheye projection. HDR is unsupported.

## Install on Steam Frame

- Download the archive
- Extract the archive into a folder you want to keep
- Double-click "install.py"
- Press "Launch" in the popup asking you what to do
- Wait for the notification in the bottom right

Now MatineeVR should be available under "Non-Steam" in your Steam Library! :)

To update, close MatineeVR and run `install.py` from the new extracted folder.

## Install from desktop

1. Enable Developer Mode and pair with SteamOS Devkit Client. Keep the Frame
   awake on the same network, with Steam running.
2. Install Python 3.8+ and OpenSSH, then extract the ZIP.
   **Windows:** open `deploy.cmd`. **Linux:** run `python3 deploy.py`.
   If needed, use `python3 deploy.py --host <Frame-IP>`.
3. Launch
   **Steam > Library > Non-Steam > Devkit Game: MatineeVR**.

## Controls

- **Browser:** D-pad up/down selects, B/left goes up a folder, A/right opens/plays.
- **Playback:** X pauses/resumes; A recenters; B returns to the browser; Y toggles debug info.
- **Volume:** use the headset's volume buttons or system controls.
- **Right stick left/right:** skip ±10 seconds with both triggers and the right grip released.
  Hold to repeat, or return to center to skip again.
- **Left inner grip:** hold for the browser.
- **Right inner grip:** hold for video settings. Stick up/down selects; left/right changes.

- **Analog triggers:** the first held trigger wins until both are released; left wins a tie.
- **Left analog trigger:** hold for alignment. Left stick: yaw/pitch; right stick: roll/zoom.
- **Right analog trigger:** hold for stereo/position. Left stick: stereo horizontal/vertical
  offset; right stick: horizontal/vertical screen or panorama position.
- **A while adjusting:** reset alignment. Centre the sticks to begin adjusting and again
  after releasing the trigger to resume seeking. More stick deflection adjusts faster.

## Video settings

Video format is detected where possible, manual changes override detection.
Settings are remembered per video until exit. New videos start with neutral alignment.

### Flat 3D movies

Select **Flat**, then **Side by side** or **Top / bottom**.
Set **SBS format** or **Top/bottom format**:

- **Full** (default): preserve each half's proportions.
- **Half**: expand each eye by 2x horizontally for SBS or vertically for top/bottom.

## Known issues

If 8K playback fails after sleep, restart the headset.

Report issues on [itch.io](https://embedding-shapes.itch.io/matineevr).
GPLv3; see `LICENSES` for license and dependency notices.
Source and build instructions are in `source.zip`.
