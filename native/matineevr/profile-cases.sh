#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"
source scripts/device.sh
if (( $# < 2 )); then
    echo 'Usage: profile-cases.sh AUDIO_VIDEO VARIANT_VIDEO [PROFILE_ARGS...]' >&2
    exit 2
fi
./profile.sh --help >&2
output=$(mktemp -d "$PWD/artifacts/seek-cases-XXXXXXXX")
cargo test --locked --release --target aarch64-unknown-linux-gnu --example profile --no-run --message-format=json > "$output/build.json"
binary=$(python3 - "$output/build.json" <<'PY'
import json
import sys

for line in open(sys.argv[1]):
    artifact = json.loads(line)
    if artifact.get("executable") and artifact.get("profile", {}).get("test"):
        print(artifact["executable"])
PY
)
cargo build --locked --release --target aarch64-unknown-linux-gnu --example verify
remote="matineevr-tools/$(basename "$output")"
frame_ssh "mkdir -p '$remote'"
frame_scp "$binary" "$FRAME_HOST:$remote/cases"
frame_scp scripts/profile-cases.py scripts/profile_results.py target/aarch64-unknown-linux-gnu/release/examples/verify "$FRAME_HOST:$remote/"
printf -v arguments '%q ' "$@"
status=0
frame_ssh "python3 -B '$remote/profile-cases.py' $arguments" || status=$?
frame_scp -r "$FRAME_HOST:$remote/." "$output/"
printf 'Results: %s\n' "$output"
exit "$status"
