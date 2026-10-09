import { spawn } from 'node:child_process';
import { mkdir, mkdtemp, readFile, writeFile } from 'node:fs/promises';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { StoreApi } from './store-api.mjs';
import { connectAccount } from './store-auth.mjs';
import { loadCatalog, SHELF_CAPACITY } from './store-catalog.mjs';
import { createStorePlayback } from './store-playback.mjs';
import { systemEnvironment } from './store-process.mjs';
import { createStoreMusic } from './store-music.mjs';

const base = join(dirname(fileURLToPath(import.meta.url)), '..');
const state = join(base, 'native-store');
await mkdir(state, { recursive: true, mode: 0o700 });
const directory = await mkdtemp(join(state, 'session-'));
await mkdir(join(directory, 'local'), { mode: 0o700 });
await writeFile(join(state, 'current-session'), directory, { mode: 0o600 });
const status = text => writeFile(join(directory, 'status'), text, { mode: 0o600 });
await status('FrameBuster Video Store\nConnecting to your Jellyfin server...');
const child = spawn(join(base, 'native-player/framebuster-video-store'), [join(directory, 'local'), ...process.argv.slice(2)], {
  stdio: 'inherit', env: { ...process.env, LD_PRELOAD: process.env.FRAMEBUSTER_PLAYER_PRELOAD || '', HALCYON_FRAME_STORE_IPC: directory,
    HALCYON_FRAME_PROGRESS_FILE: join(directory, 'progress.json') },
});
let active = true, playback = null, music = null;
const exit = new Promise(resolve => {
  child.once('error', () => { active = false; resolve(1); });
  child.once('exit', code => { active = false; resolve(code ?? 1); });
});
for (const signal of ['SIGINT', 'SIGTERM']) process.on(signal, () => {
  child.kill(signal);
  music?.close();
  setTimeout(() => process.exit(0), 3000).unref();
});
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
  try { music = await createStoreMusic(api); }
  catch { console.error('Store music unavailable; continuing without background audio'); }
  let returnedFromMovie = false;
  playback = await createStorePlayback(api, directory, status, paused => music?.setPaused(paused),
    () => { returnedFromMovie = true; });
  let page = 0, search = '', series = null, storeContext = null, revision = 0, lastCommand = '', catalog;
  const showCatalog = () => status(`FrameBuster Video Store\n${series ? `${series.Name} | Episodes` : 'Movies and Series'}\nPage ${page + 1} of ${Math.max(1, Math.ceil(catalog.total / SHELF_CAPACITY))}${search ? ` | Search: ${search}` : ''}`);
  const refresh = async () => {
    await status('Loading your movie shelves...');
    catalog = await loadCatalog(api, directory, page, search, ++revision, series);
    console.log(`Catalog: ${series ? 'episodes' : 'store'}, ${catalog.items.length} entries, page ${page + 1}`);
    await showCatalog();
  };
  const returnToStore = async () => {
    series = null;
    if (storeContext) {
      ({ page, search, catalog } = storeContext); storeContext = null;
      await writeFile(join(directory, 'catalog-path'), catalog.path, { mode: 0o600 });
      await showCatalog();
    } else { page = 0; await refresh(); }
  };
  await refresh();
  while (active) {
    await new Promise(resolve => setTimeout(resolve, 200));
    if (returnedFromMovie) {
      returnedFromMovie = false;
      if (series) {
        try { await returnToStore(); } catch (error) { await status(error.message); }
      }
    }
    let command;
    try { command = await readFile(join(directory, 'command'), 'utf8'); } catch { continue; }
    if (command === lastCommand) continue;
    lastCommand = command;
    const [, action, value] = command.split('\n');
    console.log(`Store command: ${action}, index ${value}`);
    try {
      if (action === 'play' && catalog.items[Number(value)]) {
        const item = catalog.items[Number(value)];
        if (item.Type === 'Series') { storeContext = { page, search, catalog }; series = item; page = 0; search = ''; await refresh(); }
        else await playback.play(item);
      }
      else if (action === 'back' && series) {
        await returnToStore();
      }
      else if (action === 'page' || action === 'previous') {
        page = Math.max(0, Math.min(Math.max(0, Math.ceil(catalog.total / SHELF_CAPACITY) - 1), page + (action === 'previous' ? -1 : 1)));
        await refresh();
      } else if (action === 'search') {
        const dialog = spawn('zenity', ['--entry', '--title=FrameBuster Search', '--text=Search movie titles (empty clears search)', `--entry-text=${search}`], { stdio: ['ignore', 'pipe', 'ignore'], env: systemEnvironment() });
        let text = ''; dialog.stdout.on('data', bytes => { text += bytes; });
        if (await new Promise(resolve => dialog.once('exit', resolve)) === 0) { search = text.trim().slice(0, 200); series = null; page = 0; await refresh(); }
      }
    } catch (error) {
      console.error(`Store command failed: ${error.message}`);
      if (action === 'play' && series) await returnToStore();
      await status(error.message);
    }
  }
} catch (error) { await status(error.message); }
const code = await exit;
music?.close();
await playback?.close();
process.exit(code);
