import { readFile, writeFile, chmod } from 'node:fs/promises';

export async function connectAccount(api, directory, status, active) {
  const path = `${directory}/account.json`;
  try {
    const saved = JSON.parse(await readFile(path, 'utf8'));
    if (saved.server === api.server.href && /^[a-f0-9]{32}$/i.test(saved.userId) && typeof saved.token === 'string') {
      api.account = saved;
      await api.json('/Users/Me');
      return;
    }
  } catch { api.account = null; }
  const enabled = await api.json('/QuickConnect/Enabled');
  if (!enabled) throw new Error('Enable Jellyfin Quick Connect in the server settings');
  const pending = await api.json('/QuickConnect/Initiate', { method: 'POST' });
  if (!/^\d{6}$/.test(pending.Code) || typeof pending.Secret !== 'string') throw new Error('Invalid device sign-in response');
  await status(`SIGN IN: ${pending.Code}\nIn Jellyfin, open your profile > Quick Connect.\nApprove this code to connect FrameBuster.\nB exits. No password is sent to the headset UI.`);
  for (let attempt = 0; attempt < 120 && active(); attempt++) {
    await new Promise(resolve => setTimeout(resolve, 5000));
    const state = await api.json('/QuickConnect/Connect', { query: { secret: pending.Secret } });
    if (!state.Authenticated) continue;
    const result = await api.json('/Users/AuthenticateWithQuickConnect', { method: 'POST', data: { Secret: pending.Secret } });
    if (!result.AccessToken || !/^[a-f0-9]{32}$/i.test(result.User?.Id)) throw new Error('Invalid device account');
    api.account = { server: api.server.href, token: result.AccessToken, userId: result.User.Id };
    await writeFile(path, JSON.stringify(api.account), { mode: 0o600 });
    await chmod(path, 0o600);
    return;
  }
  throw new Error('Device sign-in expired. Restart FrameBuster to retry.');
}
