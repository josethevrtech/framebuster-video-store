import { test } from 'node:test';
import assert from 'node:assert/strict';
import { createServer } from 'node:http';
import { createNativeBridge } from '../steam-frame/native-bridge.mjs';
import { playbackTarget, relayPath, rewritePlaylist } from '../steam-frame/native-policy.mjs';

const item = 'a'.repeat(32);
const configured = 'http://jellyfin.test:8096';
const source = `${configured}/Videos/${item}/stream?Static=true&api_key=private-token`;

test('native playback restricts destinations and removes credentials from relay URLs', () => {
  const target = playbackTarget(source, configured);
  assert.equal(target?.itemId, item);
  assert.equal(target?.url.includes('private-token'), false);
  for (const invalid of [source.replace('jellyfin.test', 'evil.test'),
    `${configured}/Users/Me`, `${configured}/Videos/${'b'.repeat(32)}/stream`,
    `${configured}/Videos/${item}/%2e%2e/stream`]) {
    assert.equal(playbackTarget(invalid, configured, item), null);
  }
  const session = { id: 'c'.repeat(32), itemId: item };
  const playlist = rewritePlaylist(`#EXTM3U\nmain.m3u8?api_key=private-token\n`,
    `${configured}/Videos/${item}/master.m3u8`, session, configured);
  assert.match(playlist, /\/__frame\/media\//);
  assert.equal(playlist.includes('private-token'), false);
  assert.match(relayPath(session.id, `${configured}/Videos/${item}/hls1/main/0.ts`), /media\.ts$/);
  assert.throws(() => rewritePlaylist('#EXTM3U\nhttp://evil.test/file.ts\n', source, session, configured));
});

test('native bridge relays authenticated ranges, returns only public progress, and releases sessions', async () => {
  let launches = 0;
  const requests: Array<{ target: string; options: RequestInit }> = [];
  const handler = createNativeBridge({ jellyfin: { url: configured } }, {
    env: { HALCYON_FRAME_NATIVE: '1' }, launch: async () => { launches++; },
    fetchImpl: async (target: string, options: RequestInit) => {
      requests.push({ target, options });
      return new Response('video-bytes', { status: 206,
        headers: { 'Content-Type': 'video/mp4', 'Content-Range': 'bytes 0-10/11' } });
    },
  });
  const server = createServer(async (req, res) => {
    if (!await handler(req, res, new URL(req.url!, `http://${req.headers.host}`))) {
      res.writeHead(404); res.end();
    }
  });
  await new Promise<void>(resolve => server.listen(0, '127.0.0.1', resolve));
  const address = server.address() as { port: number };
  const base = `http://127.0.0.1:${address.port}`;
  const post = (path: string, data: object) => fetch(base + path, { method: 'POST',
    headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(data) });
  try {
    assert.equal((await post('/__frame/native/play', { source: 'http://evil.test/file', token: 'abcdefgh', startSeconds: 0 })).status, 400);
    const response = await post('/__frame/native/play', { source, token: 'abcdefgh', startSeconds: 42 });
    assert.equal(response.status, 200);
    const state = await response.json();
    assert.equal(launches, 1);
    assert.equal(JSON.stringify(state).includes('abcdefgh'), false);
    assert.equal(state.position, 42);
    assert.equal((await post('/__frame/native/play', { source, token: 'abcdefgh', startSeconds: 0 })).status, 409);
    const request = await (await fetch(base + '/__frame/native/request')).json();
    assert.equal(request.url.includes('private-token'), false);
    const media = await fetch(request.url, { headers: { Range: 'bytes=0-10' } });
    assert.equal(media.status, 206);
    assert.equal(await media.text(), 'video-bytes');
    assert.equal((requests[0].options.headers as Record<string, string>)['X-Emby-Token'], 'abcdefgh');
    assert.equal((requests[0].options.headers as Record<string, string>).Range, 'bytes=0-10');
    assert.equal(requests[0].options.redirect, 'manual');
    assert.equal((await fetch(base + relayPath('f'.repeat(32), source))).status, 404);
    await post('/__frame/native/state', { id: state.id, position: 123, paused: true, running: true });
    const updated = await (await fetch(base + '/__frame/native')).json();
    assert.equal(updated.state.position, 123);
    assert.equal(updated.state.paused, true);
    await post('/__frame/native/state', { id: state.id, position: 124, running: false, ended: true });
    assert.equal((await fetch(request.url)).status, 404);
    assert.equal(await (await fetch(base + '/__frame/native/request')).json(), null);
  } finally { server.closeAllConnections(); await new Promise<void>(resolve => server.close(() => resolve())); }
});
