import argparse
import io
import json
import os
from pathlib import Path
import re
import subprocess
import zipfile
import tempfile
from urllib.error import HTTPError
from urllib.parse import quote, urlencode
from urllib.request import Request, urlopen

if __package__:
    from . import itch
    from .licenses import generate
else:
    import itch
    from licenses import generate


ROOT = Path(__file__).resolve().parents[1]


def git(*args):
    return subprocess.check_output(["git", *args], cwd=ROOT, text=True).strip()


def release_notes(changelog, tag):
    headings = list(re.finditer(r"^# (\S+)[^\n]*$", changelog, re.MULTILINE))
    matches = [i for i, heading in enumerate(headings) if heading[1] == tag.removeprefix("v")]
    if len(matches) != 1:
        raise ValueError(f"CHANGELOG.md must have exactly one '# {tag} - Title' section")
    index = matches[0]
    heading = headings[index]
    end = headings[index + 1].start() if index + 1 < len(headings) else len(changelog)
    if not changelog[heading.end():end].strip():
        raise ValueError(f"CHANGELOG.md section for {tag} is empty")
    return changelog[heading.start():end].strip()


def package(label):
    source = io.BytesIO(subprocess.check_output(["git", "archive", "--format=zip", "HEAD"], cwd=ROOT))
    with zipfile.ZipFile(source, "a") as sources:
        sources.write(ROOT / "dist/ffmpeg-7.0.tar.xz", arcname="ffmpeg-7.0.tar.xz")
    notices = generate(ROOT)
    name = f"matineevr-{quote(label, safe='')}-aarch64-unknown-linux-gnu"
    destination = Path(tempfile.mkdtemp(prefix="package-", dir=ROOT / "dist"))
    archive = destination / f"{name}.zip"
    with zipfile.ZipFile(archive, "x", compression=zipfile.ZIP_DEFLATED) as bundle:
        bundle.write(ROOT / "dist/matineevr", arcname="matineevr")
        for directory in ("ffmpeg", "mesa"):
            for path in sorted((ROOT / "dist" / directory).iterdir()):
                bundle.write(path, arcname=f"{directory}/{path.name}")
        bundle.write(ROOT / "README.user.md", arcname="README.md")
        bundle.writestr("LICENSES", notices)
        bundle.writestr("source.zip", source.getvalue())
        for filename in ("deploy.py", "deploy.cmd", "launch.sh", "scripts/install.py"):
            bundle.write(ROOT / filename, arcname=Path(filename).name)
    return archive


def request(route, data=None, headers=None, method=None):
    api = os.environ["GITHUB_API_URL"] + "/repos/" + os.environ["GITHUB_REPOSITORY"]
    headers = {"Authorization": "token " + os.environ["GITHUB_TOKEN"], **(headers or {})}
    if isinstance(data, dict):
        data = json.dumps(data).encode()
        headers["Content-Type"] = "application/json"
    with urlopen(Request(api + route, data=data, headers=headers, method=method), timeout=300) as response:
        return json.load(response)


def publish(archive, tag, notes):
    try:
        release = request("/releases/tags/" + quote(tag, safe=""))
    except HTTPError as error:
        error.close()
        if error.code != 404:
            raise
        release = request("/releases", {"tag_name": tag, "name": tag, "body": notes})
    if release.get("body") != notes:
        request(f"/releases/{release['id']}", {"body": notes}, method="PATCH")
    route = f"/releases/{release['id']}/assets"
    for asset in request(route):
        if asset["name"] == archive.name:
            print("Keeping existing release asset:", asset["browser_download_url"])
            return
    with archive.open("rb") as data:
        asset = request(route + "?" + urlencode({"name": archive.name}), data, {
            "Content-Type": "application/octet-stream",
            "Content-Length": str(archive.stat().st_size),
        })
    print(asset["browser_download_url"])


def main():
    parser = argparse.ArgumentParser(description="Package the ARM64 artifacts in dist/.")
    parser.add_argument("--publish", action="store_true")
    args = parser.parse_args()
    label = git("rev-parse", "--short", "HEAD")
    if git("status", "--porcelain"):
        if args.publish:
            raise ValueError("Publishing requires a clean Git working tree")
        label += "-dirty"
    if args.publish:
        ref = os.environ["GITHUB_REF"]
        if not ref.startswith("refs/tags/"):
            raise ValueError("Publishing requires a tag push")
        if git("rev-parse", ref + "^{commit}") != git("rev-parse", "HEAD"):
            raise ValueError("Release tag does not match the checked-out commit")
        label = ref.removeprefix("refs/tags/")
        notes = release_notes((ROOT / "CHANGELOG.md").read_text(encoding="utf-8"), label)
        if not os.environ.get("BUTLER_API_KEY"):
            raise ValueError("Publishing requires the BUTLER_API_KEY secret")
    archive = package(label)
    print(archive)
    if args.publish:
        publish(archive, label, notes)
        itch.publish(archive, label)


if __name__ == "__main__":
    main()
