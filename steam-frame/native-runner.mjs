import { spawn } from 'node:child_process';
import { readFile, writeFile } from 'node:fs/promises';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const base = join(dirname(fileURLToPath(import.meta.url)), '..');
const endpoint = `http://127.0.0.1:${process.env.HALCYON_FRAME_PORT || 1420}/__frame/native`;
let request = null;
try {
  const response = await fetch(`${endpoint}/request`, { signal: AbortSignal.timeout(3000) });
  if (response.ok) request = await response.json();
} catch { /* The standalone sample remains available without the library server. */ }
if (request && (!/^[a-f0-9]{32}$/.test(request.id) || !request.url.startsWith(`${endpoint.replace('/native', '/media')}/`))) {
  throw new Error('Invalid native handoff');
}
const progressPath = request ? join(base, 'native-player', `progress-${request.id}.json`) : null;
let progress = { position: request?.startSeconds || 0, paused: false, ended: false, error: false };
if (progressPath) await writeFile(progressPath, JSON.stringify(progress), { mode: 0o600 });
const args = request ? [request.url, '--projection', 'flat', '--stereo', 'mono'] : process.argv.slice(2);
const child = spawn(join(base, 'native-player', 'halcyon-frame-player-library'), args, {
  stdio: 'inherit', env: { ...process.env, HALCYON_FRAME_RETURN: '1',
    ...(request ? { HALCYON_FRAME_START_SECONDS: String(request.startSeconds),
      HALCYON_FRAME_PROGRESS_FILE: progressPath } : {}) },
});
let finished = false;
let posting = false;
async function report(running, failed = false) {
  if (!request || posting) return;
  posting = true;
  try {
    try { progress = JSON.parse(await readFile(progressPath, 'utf8')); } catch { /* Retry the next complete snapshot. */ }
    progress.error ||= failed;
    const response = await fetch(`${endpoint}/state`, { method: 'POST', headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ id: request.id, ...progress, running }), signal: AbortSignal.timeout(5000) });
    if (response.ok && (await response.json()).stop && !finished) child.kill('SIGTERM');
  } catch { /* Playback continues during a brief library-server outage. */ }
  finally { posting = false; }
}
const timer = setInterval(() => { void report(!finished); }, 1000);
for (const signal of ['SIGTERM', 'SIGINT']) process.on(signal, () => child.kill(signal));
const code = await new Promise(resolve => {
  child.once('error', () => resolve(1));
  child.once('exit', code => resolve(code ?? 1));
});
finished = true;
clearInterval(timer);
while (posting) await new Promise(resolve => setTimeout(resolve, 50));
await report(false, code !== 0);
process.exitCode = code;
