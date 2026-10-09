from html.parser import HTMLParser
import json
from pathlib import Path
import subprocess


class PlainText(HTMLParser):
    def __init__(self):
        super().__init__()
        self.parts = []

    def handle_data(self, data):
        if data.strip():
            self.parts.append(data.strip())


def crate_notices(crate, root):
    directory = Path(crate["manifest_path"]).parent
    files = sorted(path for path in directory.iterdir() if path.is_file()
                   and path.name.upper().startswith(("LICENSE", "LICENCE", "COPYING", "NOTICE")))
    if crate.get("license_file"):
        files = sorted(set(files + [directory / crate["license_file"]]))
    if not files and crate["name"] in ("openxr", "openxr-sys"):
        revision = json.loads((directory / ".cargo_vcs_info.json").read_text())["git"]["sha1"]
        if revision == "f50cf13afc047f96d540d9d85a08e023e0d3647d":
            files = [root / "licenses/openxrs-MIT.txt"]
    if not files:
        raise ValueError(f"Missing license text for {crate['name']} {crate['version']}")
    heading = f"{crate['name']} {crate['version']} ({crate['license']})"
    return "\n\n".join([heading, crate.get("repository") or "",
                         *(path.name + "\n" + path.read_text() for path in files)])


def generate(root):
    metadata = json.loads(subprocess.check_output([
        "cargo", "metadata", "--locked", "--offline", "--format-version", "1",
        "--filter-platform", "aarch64-unknown-linux-gnu",
    ], cwd=root, text=True))
    resolved = {node["id"] for node in metadata["resolve"]["nodes"]}
    sections = [
        "MatineeVR — GNU GPL version 3 only.\n"
        "Copyright 2026 MatineeVR contributors.\n"
        "This program comes with no warranty. See the license below.",
        (root / "LICENSE").read_text(),
        "Third-party notices\n\n"
        "The following components retain their respective licenses.\n"
        "The patched FFmpeg 7.0 codec library is bundled under LGPL 2.1 or later.\n"
        "Its original source is in source.zip/ffmpeg-7.0.tar.xz; patches and the build recipe are in source.zip/third_party/ffmpeg.\n"
        "The remaining FFmpeg, SteamVR and PulseAudio libraries are supplied by SteamOS.\n"
        "A private Mesa Turnip driver with static libdrm and zlib is bundled.\n"
        "Its modified sources and original notices are in mesa/source.tar.xz; patches and build recipe are in source.zip/third_party/mesa.\n"
        "LLVM C++ runtime notices follow the Mesa notices.",
        (root / "dist/FFMPEG-LICENSE").read_text(),
        (root / "dist/MESA-LICENSE").read_text(),
    ]
    for crate in sorted(metadata["packages"], key=lambda item: (item["name"], item["version"])):
        if crate["id"] in resolved and crate["id"] != metadata["resolve"]["root"]:
            sections.append(crate_notices(crate, root))
    sysroot = subprocess.check_output(["rustc", "--print", "sysroot"], cwd=root, text=True).strip()
    docs = Path(sysroot) / "share/doc/rust"
    parser = PlainText()
    parser.feed((docs / "COPYRIGHT-library.html").read_text())
    sections.append("Rust standard library notices (including other platforms)\n\n"
                    + "\n".join(parser.parts))
    texts = sorted((docs / "licenses").glob("*.txt"))
    if not texts:
        raise ValueError("Rust toolchain license texts are missing")
    sections.extend(path.name + "\n" + path.read_text() for path in texts)
    return "\n\n".join(sections) + "\n"
