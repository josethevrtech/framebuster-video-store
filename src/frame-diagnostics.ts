// Local-only headset health. Never include library URLs, titles or credentials.
let nextReport = 0;
let startedAt = 0;
let frames = 0;
const enabled = typeof location !== 'undefined' && new URLSearchParams(location.search).get('frame') === '1';
export function reportFrame(session: XRSession | null, time: number, tracked: boolean, mode: 'store' | 'cinema', video?: HTMLVideoElement): void {
  if (!enabled) return;
  frames++;
  if (session && time < nextReport) return;
  const fps = session && startedAt && time > startedAt ? frames * 1000 / (time - startedAt) : 0;
  startedAt = time; frames = 0; nextReport = time + 2000;
  const quality = video?.getVideoPlaybackQuality();
  void fetch('/__frame/report', { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({
    secure: window.isSecureContext, xr: !!navigator.xr, immersive: true,
    session: !!session, tracked, mode, fps,
    videoReady: video ? video.readyState >= 2 : false,
    videoFrames: quality?.totalVideoFrames ?? 0,
    droppedVideoFrames: quality?.droppedVideoFrames ?? 0,
    controllers: session ? Array.from(session.inputSources, source => ({ hand: source.handedness,
      buttons: source.gamepad?.buttons.length ?? 0, axes: source.gamepad?.axes.length ?? 0 })) : [],
  }) }).catch(() => {});
}
