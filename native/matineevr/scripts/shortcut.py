import json
import subprocess
import sys
from pathlib import Path


if len(sys.argv) > 1 and sys.argv[1] == "--request":
    game, *arguments = sys.argv[2:]
    print(json.dumps({"gameid": game, "argv": ["./matineevr", *arguments]}))
else:
    request = json.load(sys.stdin)
    game = request["gameid"]
    if not game.replace("_", "").isalnum():
        raise SystemExit("Invalid game ID")
    request.update({
        "directory": str(Path.home() / "devkit-game" / game),
        "env": {},
        "settings": {"steam_play": "0", "compat_tool": ""},
    })
    helper = Path.home() / "devkit-utils/steam-client-create-shortcut"
    response = json.loads(subprocess.check_output(
        ["python3", "-B", str(helper), "--parms", json.dumps(request)], text=True))
    if "error" in response:
        raise SystemExit(response["error"])
    print(f"Registered native Steam shortcut: {game}")
