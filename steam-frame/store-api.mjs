export class StoreApi {
  constructor(server, fetchImpl = fetch) {
    this.server = new URL(server);
    if (!['http:', 'https:'].includes(this.server.protocol) || this.server.username || this.server.password
      || this.server.search || this.server.hash) throw new Error('Invalid Jellyfin server');
    this.fetch = fetchImpl;
    this.account = null;
  }
  async request(path, { method = 'GET', data, query, timeout = 15000 } = {}) {
    const url = new URL(this.server.href.replace(/\/+$/, '') + path);
    if (query) for (const [key, value] of Object.entries(query)) url.searchParams.set(key, String(value));
    const headers = { 'Authorization': 'MediaBrowser Client="FrameBuster Video Store", Device="Steam Frame", DeviceId="framebuster-native", Version="0.1.0"' };
    if (this.account) headers['X-Emby-Token'] = this.account.token;
    if (data !== undefined) headers['Content-Type'] = 'application/json';
    const response = await this.fetch(url, { method, headers, body: data === undefined ? undefined : JSON.stringify(data),
      redirect: 'manual', signal: AbortSignal.timeout(timeout) });
    if (!response.ok) { await response.body?.cancel(); throw new Error(`Jellyfin request failed (${response.status})`); }
    return response;
  }
  async json(path, options) { return (await this.request(path, options)).json(); }
}
