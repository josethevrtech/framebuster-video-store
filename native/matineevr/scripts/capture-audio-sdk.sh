#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
source scripts/device.sh
destination=${1:?Usage: capture-audio-sdk.sh NEW_SDK_DIRECTORY}
: "${PKG_CONFIG_SYSROOT_DIR:?Set PKG_CONFIG_SYSROOT_DIR to the existing Frame SDK}"
mkdir "$destination"
cp -a "$PKG_CONFIG_SYSROOT_DIR/." "$destination/"
chmod -R u+w "$destination"
frame_ssh '
    set -euo pipefail
    dependencies=$(ldd /usr/lib/libpulse.so /usr/lib/libswresample.so)
    if [[ "$dependencies" == *"not found"* ]]; then
        echo "$dependencies" >&2
        exit 1
    fi
    {
        printf "%s\n" usr/include/pulse usr/include/libswresample \
            usr/lib/pkgconfig/libpulse.pc usr/lib/pkgconfig/libswresample.pc \
            usr/lib/libpulse.so usr/lib/libswresample.so
        readelf -d /usr/lib/libpulse.so /usr/lib/libswresample.so |
            awk '\''/\(SONAME\)/ {gsub(/[\[\]]/, "", $5); print "usr/lib/" $5}'\''
        printf "%s\n" "$dependencies" |
            awk '\''$2 == "=>" {print substr($3, 2)} $1 ~ /^\// && NF == 2 {print substr($1, 2)}'\''
    } | LC_ALL=C sort -u | tar --dereference -cf - -C / -T -
' | tar -xf - -C "$destination"
sed -i '/^Requires.private:/d; /^Libs.private:/d' \
    "$destination/usr/lib/pkgconfig/libpulse.pc" "$destination/usr/lib/pkgconfig/libswresample.pc"
printf 'Audio SDK: %s\n' "$destination"
