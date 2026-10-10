import { Readable } from 'node:stream';
import { playbackTarget, rewritePlaylist } from './native-policy.mjs';

export async function relayMedia(req, res, url, session, configured, fetchImpl = fetch) {
  const parts = url.pathname.split('/');
  if (!session || session.id !== parts[3] || !session.source || parts.length !== 6) {
    res.writeHead(404); return res.end();
  }
  let target;
  try {
    if (!/^[a-zA-Z0-9_-]{1,12000}$/.test(parts[4])) throw 0;
    const path = Buffer.from(parts[4], 'base64url').toString();
    if (!path.startsWith('/') || path.startsWith('//')) throw 0;
    target = playbackTarget(new URL(path, configured).href, configured, session.itemId);
    if (!target) throw 0;
  } catch { res.writeHead(403); return res.end(); }
  const controller = new AbortController();
  const timeout = setTimeout(() => controller.abort(), 30000);
  res.on('close', () => controller.abort());
  try {
    const headers = { 'X-Emby-Token': session.token };
    if (req.headers.range) {
      if (!/^bytes=\d*-\d*$/.test(req.headers.range)) { res.writeHead(416); return res.end(); }
      headers.Range = req.headers.range;
    }
    const upstream = await fetchImpl(target.url, { headers, redirect: 'manual', signal: controller.signal,
      method: req.method });
    clearTimeout(timeout);
    if (upstream.status >= 300 && upstream.status < 400) {
      await upstream.body?.cancel(); res.writeHead(502); return res.end();
    }
    res.setHeader('Cache-Control', 'no-store');
    if (target.url.split('?')[0].endsWith('.m3u8') && upstream.ok && req.method !== 'HEAD') {
      let text = '';
      for await (const chunk of upstream.body) {
        text += Buffer.from(chunk).toString();
        if (text.length > 2 * 1024 * 1024) throw new Error('Playlist too large');
      }
      res.writeHead(upstream.status, { 'Content-Type': 'application/vnd.apple.mpegurl' });
      return res.end(rewritePlaylist(text, target.url, session, configured));
    }
    for (const header of ['content-type', 'content-length', 'content-range', 'accept-ranges']) {
      const value = upstream.headers.get(header);
      if (value) res.setHeader(header, value);
    }
    res.writeHead(upstream.status);
    if (req.method === 'HEAD' || !upstream.body) return res.end();
    const stream = Readable.fromWeb(upstream.body);
    stream.on('error', () => res.destroy());
    res.on('close', () => stream.destroy());
    stream.pipe(res);
  } catch {
    if (!res.headersSent) res.writeHead(502);
    res.end();
  } finally { clearTimeout(timeout); }
}
