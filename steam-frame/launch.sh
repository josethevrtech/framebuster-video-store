#!/usr/bin/env bash
set -euo pipefail
APP_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
NODE="${APP_DIR}/runtime/bin/node"
if [[ ! -x "$NODE" ]]; then NODE="$(command -v node || true)"; fi
if [[ -z "$NODE" ]] || ! "$NODE" -e 'const [a,b]=process.versions.node.split(".").map(Number);process.exit(a>22||(a===22&&b>=18)?0:1)'; then
  echo 'Halcyon Frame needs Node 22.18 or newer, either on PATH or in runtime/bin/node.' >&2
  exit 1
fi
BROWSER="${HALCYON_FRAME_BROWSER:-}"
if [[ -z "$BROWSER" && -x "$APP_DIR/chrome-dev/opt/google/chrome-unstable/chrome" ]]; then BROWSER="$APP_DIR/steam-frame/browser.sh"; fi
if [[ -z "$BROWSER" ]]; then
  for candidate in chromium-xr chromium chromium-browser google-chrome-unstable; do
    if command -v "$candidate" >/dev/null 2>&1; then BROWSER="$(command -v "$candidate")"; break; fi
  done
fi
if [[ -z "$BROWSER" ]]; then echo 'Set HALCYON_FRAME_BROWSER to the executable for an immersive WebXR browser.' >&2; exit 1; fi
PORT="${HALCYON_FRAME_PORT:-1420}"
if [[ ! "$PORT" =~ ^[0-9]+$ ]] || (( PORT < 1024 || PORT > 65535 )); then echo 'Invalid HALCYON_FRAME_PORT' >&2; exit 1; fi
URL="http://127.0.0.1:${PORT}/frame-check.html"
health() {
  "$NODE" -e 'fetch(process.argv[1]).then(r=>r.json()).then(x=>process.exit(x.app==="halcyon-frame"?0:1)).catch(()=>process.exit(1))' "http://127.0.0.1:${PORT}/__frame/health"
}
if ! health; then
  if systemctl --user cat halcyon-frame-server.service >/dev/null 2>&1; then
    systemctl --user start halcyon-frame-server.service
  else
    systemd-run --user --collect --unit halcyon-frame-server \
      --setenv="HALCYON_FRAME_PORT=$PORT" "$NODE" "$APP_DIR/steam-frame/server.mjs"
  fi
fi
READY=0
for ((attempt=0; attempt<50; attempt++)); do
  if health; then READY=1; break; fi
  sleep 0.1
done
if [[ "$READY" != 1 ]]; then echo 'Local server did not become ready.' >&2; exit 1; fi
exec "$BROWSER" --app="$URL"
