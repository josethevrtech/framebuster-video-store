#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../.."
root=$PWD
recipe=$(realpath third_party/mesa)
cache=$(realpath -m artifacts/mesa)
commit=eda9aceb39d5ff169b096444abe366bdf2269e24
revision=$({ sha256sum "$recipe"/*; zig version; meson --version; } | sha256sum | cut -d' ' -f1)
work="$cache/$revision"
source="$work/mesa-$commit"
if [[ ! -f "$work/complete" ]]; then
    mkdir -p "$work"
    archive="$cache/mesa-$commit.tar.gz"
    if [[ ! -f "$archive" ]]; then
        curl --fail --location "https://gitlab.freedesktop.org/mesa/mesa/-/archive/$commit/mesa-$commit.tar.gz" -o "$archive.part"
        mv "$archive.part" "$archive"
    fi
    printf '%s  %s\n' 8b92e41f8917b101b05fffb26201310ef2fbffea7c1ee6e2de531453c0af1923 "$archive" | sha256sum -c
    if [[ ! -f "$work/patched" ]]; then
        tar -xf "$archive" -C "$work"
        patch -d "$source" -p1 < "$recipe/build.patch"
        patch -d "$source" -p1 < "$recipe/q10c.patch"
        touch "$work/patched"
    fi
    cat > "$work/cross.ini" <<EOF
[binaries]
c = ['$recipe/compiler.sh', 'cc', '-target', 'aarch64-linux-gnu.2.31']
cpp = ['$recipe/compiler.sh', 'c++', '-target', 'aarch64-linux-gnu.2.31']
ar = ['$recipe/compiler.sh', 'ar']
strip = ['llvm-strip']
pkg-config = ['pkg-config']
exe_wrapper = ['qemu-aarch64']

[host_machine]
system = 'linux'
cpu_family = 'aarch64'
cpu = 'aarch64'
endian = 'little'
EOF
    export QEMU_LD_PREFIX=${PKG_CONFIG_SYSROOT_DIR:?Set the Steam Frame SDK environment}
    meson setup "$work/build" "$source" --cross-file "$work/cross.ini" \
        --buildtype release --wrap-mode=nofallback --force-fallback-for=libdrm,zlib \
        -Dallow-fallback-for=libdrm -Dauto_features=disabled -Ddefault_library=static \
        -Dvulkan-drivers=freedreno -Dfreedreno-kmds=msm -Dgallium-drivers=[] \
        -Dplatforms=[] -Dglx=disabled -Dopengl=false -Dvideo-codecs=[] \
        -Dzlib=enabled -Dshader-cache=enabled
    timeout 1200 ninja -C "$work/build" -j"${JOBS:-8}" \
        src/freedreno/vulkan/libvulkan_freedreno.so \
        src/freedreno/vulkan/freedreno_icd.aarch64.json
    llvm-strip --strip-unneeded -o "$work/libvulkan_freedreno.so" \
        "$work/build/src/freedreno/vulkan/libvulkan_freedreno.so"
    python3 - "$work" <<'PY'
import json, sys
from pathlib import Path
work = Path(sys.argv[1])
manifest = json.loads((work / 'build/src/freedreno/vulkan/freedreno_icd.aarch64.json').read_text())
manifest['ICD']['library_path'] = './libvulkan_freedreno.so'
(work / 'freedreno_icd.aarch64.json').write_text(json.dumps(manifest, indent=2) + '\n')
PY
    XZ_OPT=-T2 tar -cJf "$work/source.tar.xz" -C "$work" "mesa-$commit"
    touch "$work/complete"
fi
mkdir -p "$root/dist/mesa"
cp "$work/libvulkan_freedreno.so" "$work/freedreno_icd.aarch64.json" "$work/source.tar.xz" "$root/dist/mesa/"
zig_lib=$(zig env | sed -n 's/.*lib_dir.*[=:] "\([^"]*\)".*/\1/p')
cat "$source/docs/license.rst" "$source/licenses/MIT" "$recipe/LICENSE.zig" \
    "$source/subprojects/zlib-1.3.1/LICENSE" \
    "$zig_lib/libcxx/LICENSE.TXT" "$zig_lib/libcxxabi/LICENSE.TXT" \
    "$zig_lib/libunwind/LICENSE.TXT" > "$root/dist/MESA-LICENSE"
