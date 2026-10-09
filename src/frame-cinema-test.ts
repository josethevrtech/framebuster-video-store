import { VRCinema } from './vr-cinema';
const cinema = new VRCinema();
const video = document.getElementById('sample') as HTMLVideoElement;
const button = document.getElementById('enter') as HTMLButtonElement;
const status = document.getElementById('status')!;
window.addEventListener('error', event => {
  // This diagnostic page only plays the bundled sample. Keep failures local.
  const APIs = ['getViewSubImage', 'getViewerPose', 'XRWebGLBinding', 'getVideoPlaybackQuality', 'framebuffer', 'matrixWorld', 'createProjectionLayer'];
  const fault = APIs.find(api => event.message.includes(api)) ?? event.error?.name ?? 'Error';
  void fetch('/__frame/report', { method: 'POST', headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ secure: window.isSecureContext, xr: !!navigator.xr, immersive: true,
      session: true, mode: 'cinema', fault }) });
});
button.onclick = () => {
  void video.play().catch(() => { status.textContent = 'Press play on the sample video to start audio.'; });
  void cinema.enter(video, {
    togglePlayPause: () => { if (video.paused) void video.play().catch(() => {}); else video.pause(); },
    seekBy: seconds => { if (Number.isFinite(video.duration)) video.currentTime = Math.max(0, Math.min(video.duration, video.currentTime + seconds)); },
  }).catch(() => { status.textContent = 'The cinema could not start. Make sure SteamVR is running and try again.'; });
};
window.addEventListener('pagehide', () => cinema.stop());
