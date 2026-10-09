import { randomBytes } from 'node:crypto';
import { spawn } from 'node:child_process';
import { playbackTarget, relayPath } from './native-policy.mjs';
import { relayMedia } from './native-media.mjs';

function json(res, status, value) {
  res.writeHead(status, { 'Content-Type': 'application/json', 'Cache-Control': 'no-store' });
  res.end(JSON.stringify(value));
}
async function body(req) {
  let text = '';
  for await (const chunk of req) {
    text += chunk;
    if (text.length > 24000) throw new Error('Payload too large');
  }
  const value = JSON.parse(text);
  if (!value || Array.isArray(value) || typeof value !== 'object') throw new Error('Invalid payload');
  return value;
}

export function createNativeBridge(config, { env = process.env, launch, fetchImpl = fetch } = {}) {
  const enabled = env.HALCYON_FRAME_NATIVE === '1';
  let session = null;
  const release = current => {
    const source = current.source && new URL(current.source);
    const token = current.token;
    current.source = null; current.token = null;
    if (source?.pathname.endsWith('.m3u8') && ['halcyon-frame-native', 'framebuster-native'].includes(source.searchParams.get('DeviceId'))) {
      const stop = new URL(`${config.jellyfin.url.replace(/\/+$/, '')}/Videos/ActiveEncodings`);
      stop.searchParams.set('DeviceId', source.searchParams.get('DeviceId'));
      stop.searchParams.set('PlaySessionId', source.searchParams.get('PlaySessionId') || '');
      void fetchImpl(stop.href, { method: 'DELETE', headers: { 'X-Emby-Token': token },
        redirect: 'manual', signal: AbortSignal.timeout(5000) }).then(r => r.body?.cancel()).catch(() => {});
    }
  };
  const publicState = () => session ? {
    id: session.id, position: session.position, paused: session.paused,
    running: session.running, ended: session.ended, error: session.error,
  } : null;
  launch ??= () => new Promise((resolve, reject) => {
    const gameId = env.HALCYON_FRAME_GAME_ID || '9530342691087319040';
    if (!/^\d{18,20}$/.test(gameId)) return reject(new Error('Invalid game ID'));
    const child = spawn('steam', [`steam://rungameid/${gameId}`], { stdio: 'ignore' });
    child.once('error', reject);
    child.once('exit', code => code === 0 ? resolve() : reject(new Error('Steam launch failed')));
  });
  return async (req, res, url) => {
    if (!url.pathname.startsWith('/__frame/native') && !url.pathname.startsWith('/__frame/media/')) return false;
    if (!enabled) { json(res, 404, { available: false }); return true; }
    if (session?.running && Date.now() - session.updated > 120000) {
      session.running = false; session.error = true; release(session);
    }
    if (url.pathname.startsWith('/__frame/media/')) {
      if (!['GET', 'HEAD'].includes(req.method)) json(res, 405, { error: 'Method not allowed' });
      else await relayMedia(req, res, url, session, config.jellyfin?.url, fetchImpl);
      return true;
    }
    if (req.method === 'GET') {
      if (url.pathname === '/__frame/native') json(res, 200, { available: !!config.jellyfin, state: publicState() });
      else if (url.pathname === '/__frame/native/request') json(res, 200, session?.running && session.source ? {
        id: session.id, url: `http://127.0.0.1:${req.socket.localPort}${relayPath(session.id, session.source)}`,
        startSeconds: session.startSeconds, stop: session.stop,
      } : null);
      else json(res, 404, { error: 'Not found' });
      return true;
    }
    if (req.method !== 'POST') { json(res, 405, { error: 'Method not allowed' }); return true; }
    let data;
    try { data = await body(req); } catch { json(res, 400, { error: 'Invalid payload' }); return true; }
    if (url.pathname === '/__frame/native/play') {
      const target = playbackTarget(data.source, config.jellyfin?.url);
      if (!target || typeof data.token !== 'string' || !/^[a-zA-Z0-9_-]{8,512}$/.test(data.token)
        || !Number.isFinite(data.startSeconds) || data.startSeconds < 0 || data.startSeconds > 1e7) {
        json(res, 400, { error: 'Invalid playback request' }); return true;
      }
      if (session?.running) { json(res, 409, { error: 'A movie is already playing' }); return true; }
      session = { id: randomBytes(16).toString('hex'), source: target.url, itemId: target.itemId,
        token: data.token, startSeconds: data.startSeconds, position: data.startSeconds,
        paused: false, ended: false, error: false, running: true, stop: false, updated: Date.now() };
      try { await launch(); json(res, 200, publicState()); }
      catch { session.running = false; session.source = null; session.token = null;
        session.error = true; json(res, 503, { error: 'Steam could not launch the cinema' }); }
    } else if (['/__frame/native/state', '/__frame/native/stop'].includes(url.pathname)) {
      if (!session || data.id !== session.id) json(res, 404, { error: 'Playback session not found' });
      else if (url.pathname.endsWith('/stop')) { session.stop = true; json(res, 200, { ok: true }); }
      else {
        if (Number.isFinite(data.position)) session.position = Math.max(0, Math.min(1e7, data.position));
        session.paused = data.paused === true; session.running = data.running === true;
        session.ended = data.ended === true; session.error = data.error === true;
        session.updated = Date.now();
        if (!session.running) release(session);
        json(res, 200, { ok: true, stop: session.stop });
      }
    } else json(res, 404, { error: 'Not found' });
    return true;
  };
}
