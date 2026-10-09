# Halcyon Frame standalone prototype

This fork targets rendering and serving Halcyon entirely on Steam Frame.
It is a prototype pending tests on the owner's headset, not a verified
standalone release. An immersive WebXR-capable browser is required; a
normal browser panel alone does not provide immersive VR.

## Runtime

The built `dist/` folder is served on `127.0.0.1` by `steam-frame/server.mjs`.
The server uses only Node's built-in modules and the existing integration
policy helpers. No development server, npm install, PC, root access or
system service is needed at launch. Node 22.18 or newer is required, either
on PATH or as a compatible Linux ARM64 executable in `runtime/bin/node`.
The browser can be selected with `HALCYON_FRAME_BROWSER` (an executable
path, not a command with arguments). No browser sandbox flags are changed
by this launcher.

Build the repository using its normal `npm ci` and `npm run build` workflow,
then run `bash steam-frame/launch.sh` on the headset. The launcher opens a
VR support check, with buttons for the regular store and demo catalog.
It keeps the local server running until the launcher is stopped. Add the
launcher to Steam as a non-Steam application to launch from the library;
the exact browser/runtime setup still needs validation on the device.

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
- Headset tests still needed: browser/runtime support, both controllers,
  physical scale, comfortable locomotion, performance, playback and resume.
- Immersive playback and controller-operated player controls are not yet
  implemented. Do not describe this prototype as a complete VR player.

Local verification: `node --test tests/frame-server.test.ts tests/vr-units.test.ts`.

Official platform setup: https://partner.steamgames.com/doc/steamhardware/steamframe/setup
