import argparse
import json
import os
from pathlib import Path
import shlex
import shutil
import subprocess
import sys


ROOT = Path(__file__).resolve().parent
GAME = "MatineeVR"
DIRECTORY = f"devkit-game/{GAME}"


def devkit_key():
    if sys.platform == "win32":
        base = Path(os.environ.get("LOCALAPPDATA", Path.home() / "AppData/Local")) / "steamos-devkit"
    elif sys.platform == "darwin":
        base = Path.home() / "Library/Application Support"
    else:
        base = Path(os.environ.get("XDG_CONFIG_HOME", Path.home() / ".config"))
    key = base / "steamos-devkit/devkit_rsa"
    return str(key) if key.is_file() else None


def deploy(args, player_args):
    binary = ROOT / "matineevr"
    if not binary.is_file():
        binary = ROOT / "dist/matineevr"
    if not binary.is_file():
        raise FileNotFoundError("matineevr is missing. Extract the complete ZIP or run ./build.sh.")
    libraries = [binary.parent / name for name in ("ffmpeg", "mesa")]
    for directory in libraries:
        if not directory.is_dir():
            raise FileNotFoundError(f"Bundled {directory.name} is missing. Extract the complete ZIP or run ./build.sh.")
    for tool in ("ssh", "scp"):
        if not shutil.which(tool):
            raise FileNotFoundError(f"{tool} is missing. Install OpenSSH and add it to PATH.")
    options = ["-o", "ConnectTimeout=10"]
    if args.key:
        key = Path(args.key).expanduser().resolve()
        if not key.is_file():
            raise FileNotFoundError(f"SSH key not found: {key}")
        options += ["-i", str(key)]
    host = args.host if "@" in args.host else f"steamos@{args.host}"

    def remote(command):
        return subprocess.run(["ssh", *options, "--", host, command], check=True,
                              stdout=subprocess.PIPE, text=True, timeout=120).stdout.strip()

    print(f"Connecting to {host}...", flush=True)
    directory = remote(f'''set -eu
command -v python3 >/dev/null
command -v fuser >/dev/null
test -f "$HOME/devkit-utils/steam-client-create-shortcut" || {{
    echo 'Enable Developer Mode and pair the SteamOS Devkit Client first.' >&2
    exit 1
}}
mkdir -p "$HOME/{DIRECTORY}"
if fuser "$HOME/{DIRECTORY}/matineevr" >/dev/null 2>&1; then
    echo 'Stopping the player before deploying...' >&2
    fuser -k "$HOME/{DIRECTORY}/matineevr" >/dev/null 2>&1 || true
    timeout 10 sh -c 'while fuser "$1" >/dev/null 2>&1; do sleep 0.1; done' sh "$HOME/{DIRECTORY}/matineevr"
fi
printf '%s\\n' "$HOME/{DIRECTORY}"
''')
    print("Copying matineevr...", flush=True)
    subprocess.run(["scp", *options, "-r", "--", f"./{binary.relative_to(ROOT).as_posix()}", "./launch.sh",
                    *(f"./{directory.relative_to(ROOT).as_posix()}" for directory in libraries),
                    f"{host}:{DIRECTORY}/"], cwd=ROOT, check=True, timeout=120)
    request = {
        "gameid": GAME, "directory": directory, "argv": ["./launch.sh", *player_args],
        "env": {}, "settings": {"steam_play": "0", "compat_tool": ""},
    }
    print("Adding the player to your Steam library...", flush=True)
    response = json.loads(remote(
        f"chmod u+x {DIRECTORY}/matineevr {DIRECTORY}/launch.sh && " + shlex.join([
            "python3", "-B", "devkit-utils/steam-client-create-shortcut", "--parms", json.dumps(request)
        ])))
    if "error" in response:
        raise RuntimeError(response["error"])
    if args.launch:
        print("Starting the player...", flush=True)
        remote(f"python3 -B devkit-utils/steam-devkit-rpc run-game gameid={GAME}")
    print(f"Ready: Steam > Library > Non-Steam > Devkit Game: {GAME}", flush=True)


def main():
    parser = argparse.ArgumentParser(
        description="Deploy to Steam Frame. Remaining arguments are passed to the player.")
    parser.add_argument("--host", default=os.environ.get("FRAME_HOST", "steamos@frame"))
    parser.add_argument("--key", default=os.environ.get("FRAME_SSH_KEY") or devkit_key())
    parser.add_argument("--launch", action="store_true", help="Start after installing")
    args, player_args = parser.parse_known_args()
    if player_args[:1] == ["--"]:
        player_args = player_args[1:]
    try:
        deploy(args, player_args)
    except subprocess.CalledProcessError as error:
        print(f"Deployment failed: {error.cmd[0]} exited with code {error.returncode}.", file=sys.stderr)
        return 1
    except subprocess.TimeoutExpired:
        print("Deployment timed out. Check the connection and keep the Frame awake.", file=sys.stderr)
        return 1
    except (OSError, ValueError, RuntimeError) as error:
        print(f"Deployment failed: {error}", file=sys.stderr)
        return 1
    except KeyboardInterrupt:
        print("Deployment cancelled.", file=sys.stderr)
        return 130
    return 0


if __name__ == "__main__":
    sys.exit(main())
