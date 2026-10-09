#!/usr/bin/env bash
set -euo pipefail
APP_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
[[ -x "$APP_DIR/runtime/bin/node" && -f "$APP_DIR/dist/index.html" ]] || { echo 'Place the ARM64 Node runtime and built app in this directory first.' >&2; exit 1; }
chmod +x "$APP_DIR/steam-frame/launch.sh" "$APP_DIR/steam-frame/browser.sh"
mkdir -p "$HOME/.config/systemd/user" "$HOME/.local/share/applications"
cat > "$HOME/.config/systemd/user/halcyon-frame-server.service" <<EOF
[Unit]
Description=Halcyon Frame local app server
[Service]
Type=simple
WorkingDirectory=$APP_DIR
EnvironmentFile=-$APP_DIR/server.env
ExecStart="$APP_DIR/runtime/bin/node" "$APP_DIR/steam-frame/server.mjs"
Restart=on-failure
RestartSec=3
[Install]
WantedBy=default.target
EOF
cat > "$HOME/.local/share/applications/halcyon-frame.desktop" <<EOF
[Desktop Entry]
Type=Application
Name=Halcyon Frame
Comment=Your standalone VR video store
Exec="$APP_DIR/steam-frame/launch.sh"
Terminal=false
Categories=AudioVideo;Video;
EOF
systemctl --user daemon-reload
echo 'Halcyon Frame launcher installed. Open Halcyon Frame from Applications, or add its launch.sh to Steam as a non-Steam app.'
