import os
from pathlib import Path
import sys
import tempfile
from urllib.parse import urlencode

sys.path.insert(0, "/usr/share/steamos-devkit/hooks")
import devkit_utils as steam


def register(base: Path, media: Path):
    game = "HalcyonFrameCinema"
    directory = Path.home() / "devkit-game" / game
    directory.mkdir(parents=True, exist_ok=True)
    launcher = base / "steam-frame" / "native-launch.sh"
    if not launcher.is_file() or not media.is_file():
        raise ValueError("Native launcher and sample media must exist")
    steam.validate_steam_client()
    steam.save_argv(game, [
        f'"{os.path.relpath(launcher, directory)}"',
        f'"{media}"', "--projection", "flat", "--stereo", "mono", "--stats",
    ])
    steam.save_settings(game, {"settings": {"steam_play": "0", "compat_tool": ""}})
    response = str(Path(tempfile.mkdtemp(prefix="halcyon-shortcut-")) / "response")
    steam.execute_steam_client_command("create-shortcut?" + urlencode({
        "response": response, "gameid": game, "force_appid": "0",
    }))
    with steam.wait_on_file_response(response, timeout=20):
        pass
    print("Updated Steam entry: Devkit Game: HalcyonFrameCinema")


if __name__ == "__main__":
    base = Path(__file__).resolve().parent.parent
    register(base, base / "media" / "sample-h264.mp4")
