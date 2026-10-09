import { targetBelongsTo } from '../src/operator-defaults.ts';

export function playbackTarget(source, configured, itemId) {
  if (!configured || typeof source !== 'string' || !targetBelongsTo(source, configured)) return null;
  const url = new URL(source);
  const prefix = new URL(configured).pathname.replace(/\/+$/, '');
  const path = url.pathname.slice(prefix.length);
  const match = /^\/Videos\/([a-f0-9]{32})\/(?:stream(?:\.[a-z0-9]+)?|(?:master|main)\.m3u8|hls[\w/.-]*\.(?:ts|m3u8|mp4|m4s))$/i.exec(path);
  if (!match || itemId && match[1].toLowerCase() !== itemId.toLowerCase()) return null;
  for (const key of [...url.searchParams.keys()]) {
    if (/^(api_key|access_token|token)$/i.test(key)) url.searchParams.delete(key);
  }
  return { url: url.href, itemId: match[1].toLowerCase() };
}

export function relayPath(id, target) {
  const clean = new URL(target);
  const encoded = Buffer.from(clean.pathname + clean.search).toString('base64url');
  const extension = /\.[a-z0-9]+$/i.exec(clean.pathname)?.[0] || '.mp4';
  return `/__frame/media/${id}/${encoded}/media${extension}`;
}

export function rewritePlaylist(text, base, session, configured) {
  const resolve = value => {
    const target = playbackTarget(new URL(value, base).href, configured, session.itemId);
    if (!target) throw new Error('Playlist destination denied');
    return relayPath(session.id, target.url);
  };
  return text.split(/\r?\n/).map(line => {
    if (!line.trim()) return line;
    if (!line.startsWith('#')) return resolve(line.trim());
    return line.replace(/URI="([^"]+)"/g, (_, value) => `URI="${resolve(value)}"`);
  }).join('\n');
}
