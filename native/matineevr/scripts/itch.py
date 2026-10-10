import hashlib
import io
from pathlib import Path
import subprocess
from urllib.request import urlopen
import zipfile


TARGET = "embedding-shapes/matineevr:steam-frame-linux-arm64"
BUTLER_VERSION = "15.31.0"
BUTLER_SHA256 = "4f2a3f22b12f870923504d4b6935535cad377b45859f5fe9419e3adc0611a48c"


def install(directory):
    url = f"https://broth.itch.zone/butler/linux-amd64/{BUTLER_VERSION}/archive/default"
    with urlopen(url, timeout=120) as response:
        data = response.read()
    if hashlib.sha256(data).hexdigest() != BUTLER_SHA256:
        raise ValueError("Butler download checksum mismatch")
    executable = Path(directory) / "butler"
    with zipfile.ZipFile(io.BytesIO(data)) as archive:
        with executable.open("xb") as output:
            output.write(archive.read("butler"))
    executable.chmod(0o755)
    return executable


def publish(archive, tag):
    executable = install(archive.parent)
    subprocess.run([str(executable), "push", str(archive), TARGET, "--userversion", tag], check=True)
