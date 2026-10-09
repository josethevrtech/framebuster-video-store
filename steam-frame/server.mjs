// Standalone, loopback-only runtime. No Vite, npm, PC or root service at launch.
import { createServer } from 'node:http';
import { createReadStream } from 'node:fs';
import { realpath, stat } from 'node:fs/promises';
import { dirname, extname, join, relative, isAbsolute } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { readOperatorEnv, publicOperatorDefaults } from '../src/operator-defaults.ts';
import { createIntegrationProxy } from '../tools/integration-proxy.mjs';

const root = dirname(fileURLToPath(import.meta.url));
const types = { '.html': 'text/html; charset=utf-8', '.js': 'text/javascript; charset=utf-8',
  '.css': 'text/css; charset=utf-8', '.json': 'application/json', '.svg': 'image/svg+xml',
  '.png': 'image/png', '.jpg': 'image/jpeg', '.jpeg': 'image/jpeg', '.webp': 'image/webp',
  '.woff': 'font/woff', '.woff2': 'font/woff2', '.mp4': 'video/mp4', '.webm': 'video/webm',
  '.mp3': 'audio/mpeg', '.ogg': 'audio/ogg', '.glb': 'model/gltf-binary', '.gltf': 'model/gltf+json' };
function reply(res, status, message) {
  res.writeHead(status, { 'Content-Type': 'application/json', 'Cache-Control': 'no-store' });
  res.end(JSON.stringify(message));
}
function inside(base, target) {
  const rel = relative(base, target);
  return rel !== '..' && !rel.startsWith(`..${process.platform === 'win32' ? '\\' : '/'}`) && !isAbsolute(rel);
}

export async function createFrameServer({ dist = join(root, '../dist'), env = process.env } = {}) {
  const base = await realpath(dist);
  const config = readOperatorEnv(env);
  const proxy = createIntegrationProxy(config, { env });
  return createServer(async (req, res) => {
    res.setHeader('X-Content-Type-Options', 'nosniff');
    res.setHeader('Referrer-Policy', 'no-referrer');
    let url;
    try {
      url = new URL(req.url, `http://${req.headers.host}`);
      const hosts = [`127.0.0.1:${req.socket.localPort}`, `localhost:${req.socket.localPort}`];
      if (!hosts.includes(req.headers.host) || url.host !== req.headers.host) throw 0;
      if (req.headers.origin && new URL(req.headers.origin).origin !== url.origin) throw 0;
      if (req.headers['sec-fetch-site'] === 'cross-site') throw 0;
    } catch { return reply(res, 403, { error: 'Local access only' }); }
    try {
      if (url.pathname === '/dev-proxy') return await proxy(req, res, () => reply(res, 404, { error: 'Not found' }));
      if (!['GET', 'HEAD'].includes(req.method)) return reply(res, 405, { error: 'Method not allowed' });
      if (url.pathname === '/__halcyon/config') return reply(res, 200, publicOperatorDefaults(config));
      if (url.pathname === '/__frame/health') return reply(res, 200, { app: 'halcyon-frame', standalone: true });
      // The diagnostic page is shipped separately from the app bundle.
      let filename = url.pathname === '/frame-check.html' ? join(root, 'frame-check.html')
        : join(base, decodeURIComponent(url.pathname === '/' ? '/index.html' : url.pathname));
      if (url.pathname !== '/frame-check.html' && !inside(base, filename)) return reply(res, 403, { error: 'Path denied' });
      filename = await realpath(filename);
      if (url.pathname !== '/frame-check.html' && !inside(base, filename)) return reply(res, 403, { error: 'Path denied' });
      const info = await stat(filename);
      if (!info.isFile()) return reply(res, 404, { error: 'Not found' });
      let start = 0, end = info.size - 1, status = 200;
      if (req.headers.range) {
        const match = /^bytes=(\d*)-(\d*)$/.exec(req.headers.range);
        if (!match || (!match[1] && !match[2])) {
          res.setHeader('Content-Range', `bytes */${info.size}`);
          return reply(res, 416, { error: 'Invalid range' });
        }
        start = match[1] ? Number(match[1]) : Math.max(0, info.size - Number(match[2]));
        end = match[1] && match[2] ? Math.min(Number(match[2]), info.size - 1) : info.size - 1;
        if (!Number.isSafeInteger(start) || !Number.isSafeInteger(end) || start > end || start >= info.size) {
          res.setHeader('Content-Range', `bytes */${info.size}`);
          return reply(res, 416, { error: 'Invalid range' });
        }
        status = 206;
        res.setHeader('Content-Range', `bytes ${start}-${end}/${info.size}`);
      }
      res.writeHead(status, { 'Content-Type': types[extname(filename)] || 'application/octet-stream',
        'Content-Length': Math.max(0, end - start + 1), 'Accept-Ranges': 'bytes', 'Cache-Control': 'no-cache' });
      if (req.method === 'HEAD' || !info.size) return res.end();
      const stream = createReadStream(filename, { start, end });
      stream.on('error', () => res.destroy());
      res.on('close', () => stream.destroy());
      stream.pipe(res);
    } catch {
      if (!res.headersSent) reply(res, 404, { error: 'Not found' });
      else res.destroy();
    }
  });
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  const port = Number(process.env.HALCYON_FRAME_PORT || 1420);
  if (!Number.isInteger(port) || port < 1024 || port > 65535) throw new Error('Invalid HALCYON_FRAME_PORT');
  const server = await createFrameServer();
  server.on('error', () => { console.error('Halcyon Frame could not start its local server. Check the port and built app.'); process.exit(1); });
  server.listen(port, '127.0.0.1', () => console.log(`Halcyon Frame: http://127.0.0.1:${port}/frame-check.html`));
  for (const signal of ['SIGINT', 'SIGTERM']) process.on(signal, () => server.close(() => process.exit(0)));
}
