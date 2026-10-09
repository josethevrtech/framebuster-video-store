import { test } from 'node:test';
import assert from 'node:assert/strict';
import { mkdtemp, writeFile, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { once } from 'node:events';
import { request } from 'node:http';
import { createFrameServer } from '../steam-frame/server.mjs';

test('standalone server serves the app and media ranges, without exposing runtime files or other origins', async () => {
  const dist = await mkdtemp(join(tmpdir(), 'halcyon-frame-'));
  await writeFile(join(dist, 'index.html'), '<h1>Frame store</h1>');
  await writeFile(join(dist, 'movie.mp4'), '0123456789');
  const server = await createFrameServer({ dist, env: { HALCYON_JELLYFIN_URL: 'http://library.local:8096' } });
  server.listen(0, '127.0.0.1');
  await once(server, 'listening');
  const base = `http://127.0.0.1:${server.address().port}`;
  try {
    assert.equal(await (await fetch(base)).text(), '<h1>Frame store</h1>');
    assert.deepEqual(await (await fetch(`${base}/__frame/health`)).json(), { app: 'halcyon-frame', standalone: true });
    assert.deepEqual(await (await fetch(`${base}/__halcyon/config`)).json(), { jellyfin: { url: 'http://library.local:8096' } });
    const range = await fetch(`${base}/movie.mp4`, { headers: { Range: 'bytes=2-5' } });
    assert.equal(range.status, 206);
    assert.equal(range.headers.get('content-range'), 'bytes 2-5/10');
    assert.equal(await range.text(), '2345');
    assert.equal(await (await fetch(`${base}/movie.mp4`, { headers: { Range: 'bytes=-3' } })).text(), '789');
    assert.equal((await fetch(`${base}/movie.mp4`, { headers: { Range: 'bytes=10-' } })).status, 416);
    assert.equal((await fetch(`${base}/movie.mp4`, { method: 'HEAD' })).headers.get('content-length'), '10');
    const badHostStatus = await new Promise(resolve => {
      const req = request(base, { headers: { Host: 'attacker.example' } }, res => { res.resume(); resolve(res.statusCode); });
      req.end();
    });
    assert.equal(badHostStatus, 403);
    assert.equal((await fetch(base, { headers: { Origin: 'https://attacker.example' } })).status, 403);
    assert.equal((await fetch(`${base}/__halcyon/config`, { headers: { 'Sec-Fetch-Site': 'cross-site' } })).status, 403);
    assert.equal((await fetch(`${base}/%2e%2e%2fpackage.json`)).status, 403);
    assert.equal((await fetch(`${base}/src/operator-defaults.ts`)).status, 404);
    assert.equal((await fetch(`${base}/dev-proxy`, { headers: { 'x-proxy-target': 'http://unconfigured.local/api/v1/movie' } })).status, 403);
    assert.equal((await fetch(base, { method: 'POST' })).status, 405);
    assert.match(await (await fetch(`${base}/frame-check.html`)).text(), /isSessionSupported/);
  } finally {
    await new Promise(resolve => server.close(resolve));
    await rm(dist, { recursive: true, force: true });
  }
});
