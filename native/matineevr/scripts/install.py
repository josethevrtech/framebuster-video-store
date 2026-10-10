#!/usr/bin/env python3
import logging
import os
import subprocess
import sys
import tempfile
from pathlib import Path
from urllib.parse import urlencode


def install(directory, game="MatineeVR"):
    for name in ("matineevr", "launch.sh"):
        if not (directory / name).is_file():
            raise FileNotFoundError(f"Missing {name}. Extract the complete ZIP first.")
    base = Path.home() / "devkit-game" / game
    launcher = os.path.relpath(directory / "launch.sh", base)
    if any(character in launcher for character in '\\"$`\n\r'):
        raise ValueError(
            "Use a folder path without double quotes, backslashes, "
            "dollar signs, backticks or line breaks."
        )
    sys.path.insert(0, "/usr/share/steamos-devkit/hooks")
    import devkit_utils as steam

    steam.validate_steam_client()
    base.mkdir(parents=True, exist_ok=True)
    steam.save_argv(game, [f'"{launcher}"'])
    steam.save_settings(game, {"settings": {"steam_play": "0", "compat_tool": ""}})
    response = tempfile.mkdtemp(prefix="matineevr-install-", dir="/tmp") + "/response"
    steam.execute_steam_client_command(
        "create-shortcut?" + urlencode({"response": response, "gameid": game})
    )
    with steam.wait_on_file_response(response):
        pass


def main():
    try:
        install(Path(__file__).resolve().parent)
        message = "Ready: Steam > Library > Non-Steam > Devkit Game: MatineeVR"
        result = 0
    except Exception as error:
        logging.getLogger(__name__).exception("Installation failed")
        message = f"Installation failed: {str(error) or type(error).__name__}"
        result = 1
    print(message, file=sys.stderr if result else sys.stdout)
    if os.environ.get("DISPLAY") or os.environ.get("WAYLAND_DISPLAY"):
        dialog = ["--error", message] if result else ["--passivepopup", message, "8"]
        subprocess.run(["kdialog", "--title", "MatineeVR", *dialog], check=True)
    return result


if __name__ == "__main__":
    sys.exit(main())
