import { spawn } from 'node:child_process';
import { mkdir, mkdtemp, readFile, writeFile } from 'node:fs/promises';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { StoreApi } from './store-api.mjs';
import { connectAccount } from './store-auth.mjs';
import { loadCatalog } from './store-catalog.mjs';
import { createStorePlayback } from './store-playback.mjs';
import { systemEnvironment } from './store-process.mjs';

const base = join(dirname(fileURLToPath(import.meta.url)), '..');
const state = join(base, 'native-store');
await mkdir(state, { recursive: true, mode: 0o700 });
const directory = await mkdtemp(join(state, 'session-'));
await mkdir(join(directory, 'local'), { mode: 0o700 });
await writeFile(join(state, 'current-session'), directory, { mode: 0o600 });
const status = text => writeFile(join(directory, 'status'), text, { mode: 0o600 });
await status('FrameBuster Video Store\nConnecting to your Jellyfin server...');
const child = spawn(join(base, 'native-player/framebuster-video-store'), [join(directory, 'local'), ...process.argv.slice(2)], {
  stdio: 'inherit', env: { ...process.env, HALCYON_FRAME_STORE_IPC: directory,
    HALCYON_FRAME_PROGRESS_FILE: join(directory, 'progress.json') },
});
let active = true, playback = null;
const exit = new Promise(resolve => {
  child.once('error', () => { active = false; resolve(1); });
  child.once('exit', code => { active = false; resolve(code ?? 1); });
});
for (const signal of ['SIGINT', 'SIGTERM']) process.on(signal, () => child.kill(signal));
let server = process.env.FRAMEBUSTER_JELLYFIN_URL;
if (!server) {
  try { server = (await readFile(join(state, 'server'), 'utf8')).trim(); } catch {}
}
if (!server) {
  const dialog = spawn('zenity', ['--entry', '--title=FrameBuster Video Store', '--text=Jellyfin server address', '--entry-text=http://'], { stdio: ['ignore', 'pipe', 'ignore'], env: systemEnvironment() });
  let value = ''; dialog.stdout.on('data', bytes => { value += bytes; });
  if (await new Promise(resolve => dialog.once('exit', resolve)) === 0) server = value.trim();
}
try {
  if (!server) throw new Error('No Jellyfin server configured. Restart to enter its address.');
  const api = new StoreApi(server);
  await writeFile(join(state, 'server'), api.server.href, { mode: 0o600 });
  await connectAccount(api, state, status, () => active);
  playback = await createStorePlayback(api, directory, status);
  let page = 0, search = '', revision = 0, lastCommand = '', catalog;
  const refresh = async () => {
    await status('Loading your movie shelves...');
    catalog = await loadCatalog(api, directory, page, search, ++revision);
    await status(`FrameBuster Video Store\nPage ${page + 1} of ${Math.max(1, Math.ceil(catalog.total / 6))}${search ? ` | Search: ${search}` : ''}\nTrigger: details | A: play | Right stick: pages\nY: search | Left stick: move and snap turn | B: back`);
  };
  await refresh();
  while (active) {
    await new Promise(resolve => setTimeout(resolve, 200));
    let command;
    try { command = await readFile(join(directory, 'command'), 'utf8'); } catch { continue; }
    if (command === lastCommand) continue;
    lastCommand = command;
    const [, action, value] = command.split('\n');
    try {
      if (action === 'play' && catalog.items[Number(value)]) await playback.play(catalog.items[Number(value)]);
      else if (action === 'page' || action === 'previous') {
        page = Math.max(0, Math.min(Math.max(0, Math.ceil(catalog.total / 6) - 1), page + (action === 'previous' ? -1 : 1)));
        await refresh();
      } else if (action === 'search') {
        const dialog = spawn('zenity', ['--entry', '--title=FrameBuster Search', '--text=Search movie titles (empty clears search)', `--entry-text=${search}`], { stdio: ['ignore', 'pipe', 'ignore'], env: systemEnvironment() });
        let text = ''; dialog.stdout.on('data', bytes => { text += bytes; });
        if (await new Promise(resolve => dialog.once('exit', resolve)) === 0) { search = text.trim().slice(0, 200); page = 0; await refresh(); }
      }
    } catch (error) { await status(error.message); }
  }
} catch (error) { await status(error.message); }
const code = await exit;
await playback?.close();
process.exitCode = code;
