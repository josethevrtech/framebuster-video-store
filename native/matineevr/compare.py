import argparse
import csv
import datetime
import io
import json
import math
import os
from pathlib import Path
import shlex
import subprocess
import time

from deploy import ROOT, devkit_key


def main():
    parser = argparse.ArgumentParser(description="Build and compare GLES/Vulkan on Steam Frame.")
    parser.add_argument("video", help="Absolute path on the Frame")
    parser.add_argument("--host", default=os.environ.get("FRAME_HOST", "steamos@frame"))
    parser.add_argument("--key", default=os.environ.get("FRAME_SSH_KEY") or devkit_key())
    parser.add_argument("--warmup", type=float, default=30)
    parser.add_argument("--seconds", type=float, default=120)
    parser.add_argument("--verify-only", action="store_true")
    parser.add_argument("--quick", action="store_true", help="GLES then Vulkan: 5s warmup + 25s measurement each")
    parser.add_argument("--projection", choices=["flat", "180", "360"], default="180")
    parser.add_argument("--stereo", choices=["mono", "sbs", "tb"], default="sbs")
    args = parser.parse_args()
    if args.quick:
        args.warmup, args.seconds = 5, 25
    if not (0 <= args.warmup <= 3600 and 0 < args.seconds <= 3600):
        parser.error("Use 0–3600 seconds warmup and 0–3600 seconds measurement (positive).")
    subprocess.run(["cargo", "build", "--locked", "--release", "--target",
                    "aarch64-unknown-linux-gnu", "--features", "render-compare",
                    "--example", "render_compare"], cwd=ROOT, check=True, timeout=600)
    run = datetime.datetime.now().strftime("render-compare-%Y%m%d-%H%M%S-%f")
    output = ROOT / "artifacts" / run
    output.mkdir(parents=True, exist_ok=False)
    host = args.host if "@" in args.host else f"steamos@{args.host}"
    options = ["-o", "BatchMode=yes", "-o", "ConnectTimeout=5"]
    if args.key:
        options += ["-i", str(Path(args.key).expanduser())]

    def remote(command):
        return subprocess.run(["ssh", *options, "--", host, command], check=True,
                              capture_output=True, text=True, timeout=20).stdout.strip()

    game = "MatineeVR_Benchmark"
    directory = f"devkit-game/{game}/{run}"
    remote("if pgrep -x matineevr >/dev/null; then echo 'Stop MatineeVR before benchmarking' >&2; exit 1; fi")
    remote(f"mkdir -p {shlex.quote(directory)}")
    binary = ROOT / "target/aarch64-unknown-linux-gnu/release/examples/render_compare"
    subprocess.run(["scp", *options, "--", str(binary), f"{host}:{directory}/matineevr"],
                   check=True, timeout=120)
    subprocess.run(["scp", *options, "--", str(ROOT / "launch.sh"), f"{host}:{directory}/"],
                   check=True, timeout=120)
    absolute = remote(f"cd {shlex.quote(directory)} && pwd")
    remote(f"chmod u+x {shlex.quote(directory)}/matineevr {shlex.quote(directory)}/launch.sh")
    dimensions = None
    modes = ["verify"] if args.verify_only else ["verify", "gl", "vk"] + ([] if args.quick else ["vk", "gl"])
    for index, api in enumerate(modes):
        stem = f"{index}-{api}"
        limit = math.ceil(args.warmup + args.seconds + 60)
        argv = ["timeout", str(limit), "./launch.sh", api, args.video,
                "--warmup", str(args.warmup), "--seconds", str(args.seconds),
                "--projection", args.projection, "--stereo", args.stereo]
        command = (f"#!/bin/sh\ncd {shlex.quote(absolute)} || exit 1\n"
                   f"{shlex.join(argv)} >{stem}.csv 2>{stem}.log\n"
                   f"printf '%s\\n' \"$?\" >{stem}.status")
        remote(f"printf %s {shlex.quote(command)} >{directory}/{stem}.sh && chmod u+x {directory}/{stem}.sh")
        request = {"gameid": game, "directory": absolute, "argv": [f"./{run}/{stem}.sh"],
                   "env": {}, "settings": {"steam_play": "0", "compat_tool": ""}}
        response = json.loads(remote(shlex.join([
            "python3", "-B", "devkit-utils/steam-client-create-shortcut", "--parms", json.dumps(request)])))
        if "error" in response:
            raise RuntimeError(response["error"])
        print(f"Run {index}: {api}; keep the headset worn and still. Saving {output}", flush=True)
        remote(f"python3 -B devkit-utils/steam-devkit-rpc run-game gameid={game}")
        deadline = time.monotonic() + limit + 20
        status = ""
        while not status and time.monotonic() < deadline:
            time.sleep(5)
            status = remote(f"if test -f {directory}/{stem}.status; then cat {directory}/{stem}.status; fi")
        for suffix in ("csv", "log"):
            text = remote(f"cat {directory}/{stem}.{suffix}")
            (output / f"{stem}.{suffix}").write_text(text + "\n")
        log = (output / f"{stem}.log").read_text()
        print("\n".join(line for line in log.splitlines() if line.startswith(
            ("Summary:", "Timing:", "Power:", "Verification passed", "Error:"))), flush=True)
        if status != "0":
            raise RuntimeError(f"Run {stem} failed (status {status or 'missing'}); see {output}")
        if api == "verify":
            if "Verification passed:" not in log:
                raise RuntimeError(f"Missing pixel verification: {output}")
            continue
        eyes = [line for line in log.splitlines() if line.startswith("Eye:")]
        if len(eyes) != 2 or dimensions is not None and eyes != dimensions:
            raise RuntimeError("Swapchain dimensions differ; discard comparison")
        dimensions = eyes
        rows = list(csv.DictReader(io.StringIO((output / f"{stem}.csv").read_text())))
        if len(rows) < 2 or any(row["api"] != api for row in rows):
            raise RuntimeError(f"Missing or mismatched frame measurements: {output}")
    print(f"Saved {output}. The normal MatineeVR shortcut is unchanged.")


if __name__ == "__main__":
    main()
