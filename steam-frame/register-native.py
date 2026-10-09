import os
from pathlib import Path
import sys
import tempfile
from urllib.parse import urlencode

sys.path.insert(0, "/usr/share/steamos-devkit/hooks")
import devkit_utils as steam


def register(base: Path, media=None, store=False):
    game = "HalcyonFrameStore" if store else "HalcyonFrameCinema" if media else "HalcyonFrame"
    directory = Path.home() / "devkit-game" / game
    directory.mkdir(parents=True, exist_ok=True)
    launcher = base / "steam-frame" / ("store-launch.sh" if store else "native-launch.sh" if media else "launch.sh")
    if not launcher.is_file() or media and not media.is_file():
        raise ValueError("Native launcher and sample media must exist")
    steam.validate_steam_client()
    argv = [f'"{os.path.relpath(launcher, directory)}"']
    if media:
        argv.extend([f'"{media}"', "--projection", "flat", "--stereo", "mono", "--stats"])
    steam.save_argv(game, argv)
    steam.save_settings(game, {"settings": {"steam_play": "0", "compat_tool": ""}})
    response = str(Path(tempfile.mkdtemp(prefix="halcyon-shortcut-")) / "response")
    steam.execute_steam_client_command("create-shortcut?" + urlencode({
        "response": response, "gameid": game, "force_appid": "0",
    }))
    with steam.wait_on_file_response(response, timeout=20):
        pass
    print(f"Updated Steam entry: Devkit Game: {game}")


if __name__ == "__main__":
    base = Path(__file__).resolve().parent.parent
    register(base, base / "media" / "sample-h264.mp4")
    register(base)
    register(base, store=True)
