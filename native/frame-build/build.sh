#!/bin/sh
set -eu
tools=$(CDPATH= cd -- "$(dirname "$0")" && pwd)
export PATH="$tools:$PATH"
cd "$tools/../matineevr"
exec cargo build -j 1 --release --locked --target aarch64-unknown-linux-gnu \
    --config 'target.aarch64-unknown-linux-gnu.linker="steam-frame-link"'
