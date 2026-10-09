import { randomBytes } from 'node:crypto';
import { createServer } from 'node:http';
import { readFile, writeFile } from 'node:fs/promises';
import { createNativeBridge } from './native-bridge.mjs';

export async function createStorePlayback(api, directory, status, pauseMusic = () => {}, onStopped = () => {}) {
  const bridge = createNativeBridge({ jellyfin: { url: api.server.href } }, {
    env: { HALCYON_FRAME_NATIVE: '1' }, launch: async () => {},
  });
  const server = createServer(async (req, res) => {
    if (req.headers.host !== `127.0.0.1:${server.address().port}` || req.headers.origin) { res.writeHead(403); return res.end(); }
    try { await bridge(req, res, new URL(req.url, `http://${req.headers.host}`)); }
    catch { if (!res.headersSent) res.writeHead(500); res.end(); }
  });
  await new Promise(resolve => server.listen(0, '127.0.0.1', resolve));
  const endpoint = `http://127.0.0.1:${server.address().port}/__frame/native`;
  const post = async (route, data) => {
    const response = await fetch(endpoint + route, { method: 'POST', headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(data), signal: AbortSignal.timeout(10000) });
    if (!response.ok) throw new Error(`Cinema request failed (${response.status})`);
    return response.json();
  };
  let current = null, revision = 0, reporting = false;
  const notify = async (route, data) => { try { await api.request(route, { method: 'POST', data }); } catch {} };
  const timer = setInterval(async () => {
    if (!current || reporting) return;
    reporting = true;
    try {
      const state = JSON.parse(await readFile(`${directory}/progress.json`, 'utf8'));
      if (!current.started && !state.running && !state.error) return;
      if (!current.started && state.running) {
        current.started = true;
        await notify('/Sessions/Playing', { ItemId: current.itemId, PlaySessionId: current.playSessionId,
          PositionTicks: Math.round(state.position * 1e7), CanSeek: true, PlayMethod: 'Transcode' });
      }
      await post('/state', { id: current.id, ...state });
      if (current.started && Date.now() - current.reported > 10000) {
        current.reported = Date.now();
        await notify('/Sessions/Playing/Progress', { ItemId: current.itemId, PlaySessionId: current.playSessionId,
          PositionTicks: Math.round(state.position * 1e7), IsPaused: state.paused, CanSeek: true });
      }
      if (!state.running) {
        await notify('/Sessions/Playing/Stopped', { ItemId: current.itemId, PlaySessionId: current.playSessionId,
          PositionTicks: Math.round(state.position * 1e7), Failed: state.error });
        await status(state.error ? 'The movie could not play. Choose another title.' : 'Choose a movie. Trigger: details. A: play.');
        current = null;
        pauseMusic(false);
        onStopped();
      }
    } catch {} finally { reporting = false; }
  }, 1000);
  return {
    async play(item) {
      if (current) return;
      await status(`Opening ${item.Name}...`);
      const info = await api.json(`/Items/${item.Id}/PlaybackInfo`, { method: 'POST', data: {
        UserId: api.account.userId, StartTimeTicks: 0, IsPlayback: true, EnableDirectPlay: false,
        EnableDirectStream: false, MaxStreamingBitrate: 20000000,
        DeviceProfile: { Name: 'FrameBuster Steam Frame', MaxStreamingBitrate: 20000000,
          TranscodingProfiles: [{ Container: 'ts', Type: 'Video', VideoCodec: 'h264', AudioCodec: 'aac',
            Context: 'Streaming', Protocol: 'hls', MaxAudioChannels: '2', MinSegments: 1, BreakOnNonKeyFrames: true }] },
      } });
      const media = info.MediaSources?.[0];
      if (!media) throw new Error('Jellyfin did not provide a video source');
      const playSessionId = info.PlaySessionId || randomBytes(16).toString('hex');
      const source = new URL(api.server.href.replace(/\/+$/, '') + `/Videos/${item.Id}/master.m3u8`);
      for (const [key, value] of Object.entries({ MediaSourceId: media.Id, DeviceId: 'framebuster-native',
        PlaySessionId: playSessionId, VideoCodec: 'h264', AudioCodec: 'aac', VideoBitrate: 20000000,
        AudioBitrate: 192000, MaxAudioChannels: 2, TranscodingMaxAudioChannels: 2,
        TranscodingProtocol: 'hls', TranscodingContainer: 'ts', SegmentContainer: 'ts',
        MinSegments: 1, BreakOnNonKeyFrames: true, StartTimeTicks: 0 })) source.searchParams.set(key, String(value));
      const startSeconds = Math.max(0, (item.UserData?.PlaybackPositionTicks || 0) / 1e7);
      const state = await post('/play', { source: source.href, token: api.account.token, startSeconds });
      const request = await (await fetch(endpoint + '/request')).json();
      await writeFile(`${directory}/progress.json`, JSON.stringify({ position: startSeconds, paused: false,
        running: false, ended: false, error: false }), { mode: 0o600 });
      current = { id: state.id, itemId: item.Id, playSessionId, started: false, reported: 0 };
      pauseMusic(true);
      try {
        await writeFile(`${directory}/movie-request`, `${++revision}\n${request.url}\n${startSeconds}`, { mode: 0o600 });
      } catch (error) { current = null; pauseMusic(false); throw error; }
    },
    async close() {
      clearInterval(timer);
      if (current) await post('/state', { id: current.id, running: false }).catch(() => {});
      server.closeAllConnections(); server.close();
    },
  };
}
