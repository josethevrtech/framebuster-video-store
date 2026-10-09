#!/usr/bin/env bash
set -euo pipefail
APP_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
BROWSER="$APP_DIR/chrome-dev/opt/google/chrome-unstable/chrome"
if [[ ! -x "$BROWSER" ]]; then echo 'Install the Linux ARM64 Chrome Dev runtime into chrome-dev first.' >&2; exit 1; fi
export XR_RUNTIME_JSON="${XR_RUNTIME_JSON:-/opt/steamvr/steamxr_linuxarm64.json}"
export DISPLAY="${DISPLAY:-:0}"
# Temporary SteamVR ARM64 compatibility: its SO_PEERCRED call is rejected
# by Chromium's XR seccomp policy. This applies to this isolated app profile
# only. Namespace sandboxing stays enabled; do not use --no-sandbox.
exec "$BROWSER" --user-data-dir="$APP_DIR/browser-dev-profile" \
  --no-first-run --no-default-browser-check --enable-features=OpenXR \
  --disable-features=WebXRLayers --disable-blink-features=WebXRLayers \
  --disable-seccomp-filter-sandbox "$@"
