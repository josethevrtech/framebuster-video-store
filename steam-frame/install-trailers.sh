#!/bin/sh
set -eu
base=$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)
version=2026.08.19
expected=1fa6733c37ea6fb51c99ad8fe785e7b7e5f3246c9b980230329d4fb72ed8d4d6
mkdir -p "$base/native-tools"
target="$base/native-tools/yt-dlp"
if [ ! -f "$target" ]; then
    curl --fail --location "https://github.com/yt-dlp/yt-dlp/releases/download/$version/yt-dlp" --output "$target"
fi
actual=$(sha256sum "$target")
[ "${actual%% *}" = "$expected" ] || { echo 'Preserved yt-dlp file differs from the pinned release.' >&2; exit 1; }
python3 "$target" --version
command -v ffmpeg >/dev/null
echo 'Native library-matched ceiling trailers are available.'
