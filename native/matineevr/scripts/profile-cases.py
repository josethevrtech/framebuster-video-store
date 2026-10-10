import hashlib
import json
import os
import subprocess
import sys
from pathlib import Path

from profile_results import collect

ROOT = Path(__file__).resolve().parent
TOOLS = ROOT.parent
VRCMD = "/opt/steamvr/bin/linuxarm64/vrcmd"


def setting(*args):
    return subprocess.check_output(
        [VRCMD, "--background", *args],
        text=True,
        timeout=15,
        env=os.environ | {"LD_LIBRARY_PATH": "/opt/steamvr/bin/linuxarm64"},
    ).strip()


def digest(path):
    with path.open("rb") as file:
        return hashlib.file_digest(file, "sha256").hexdigest()


def main():
    if len(sys.argv) < 3:
        raise SystemExit(
            "Usage: profile-cases.sh AUDIO_VIDEO VARIANT_VIDEO [PROFILE_ARGS...]"
        )
    file, variant, *args = sys.argv[1:]
    args = args or ["1.25", "6.6", "0", "--seconds", "1", "--repeat", "3"]
    if (
        subprocess.run(
            ["pgrep", "-x", "matineevr"], stdout=subprocess.DEVNULL, check=False
        ).returncode
        == 0
    ):
        raise SystemExit("Close MatineeVR before profiling")
    env = os.environ | {
        "LD_LIBRARY_PATH": f"{TOOLS}/ffmpeg:/opt/steamvr/bin/linuxarm64",
        "XR_RUNTIME_JSON": "/opt/steamvr/steamxr_linuxarm64.json",
        "VK_DRIVER_FILES": str(TOOLS / "mesa/freedreno_icd.aarch64.json"),
        "FRAME_TEST_VIDEO": file,
        "FRAME_TEST_VARIANT": variant,
        "FRAME_TEST_ARGS": " ".join([*args, "--thumbnails", "--stats"]),
    }
    files = [
        ROOT / "cases",
        ROOT / "verify",
        *sorted((TOOLS / "ffmpeg").glob("*.so.*")),
        *sorted((TOOLS / "mesa").glob("*")),
    ]
    key = "steamvr.performanceProfile"
    original = setting("--settings-string", key).split("=", 1)[1]
    manifest = {
        "libraries": {
            str(path.relative_to(TOOLS)): digest(path)
            for path in files
            if path.is_file()
        },
        "inputs": [
            {
                "bytes": Path(path).stat().st_size,
                "mtime_ns": Path(path).stat().st_mtime_ns,
            }
            for path in [file, variant]
        ],
        "args": args,
        "cache": "uncontrolled",
        "kernel": os.uname().release,
        "original_profile": original,
    }
    results = {}
    try:
        setting("--set-settings-string", key, "Performance")
        manifest["test_profile"] = setting("--settings-string", key)
        (ROOT / "manifest.json").write_text(json.dumps(manifest, indent=2) + "\n")
        for case, module in [
            ("audio_reset", "seek_cases"),
            ("capture_pressure", "seek_cases"),
            ("pipeline_lifetime", "pipeline_case"),
        ]:
            command = [
                str(ROOT / "cases"),
                "--ignored",
                "--exact",
                f"{module}::{case}",
                "--nocapture",
                "--quiet",
                "--test-threads=1",
            ]
            results[case] = collect(ROOT, case, command, env)
        for index, source in enumerate([file, variant]):
            results[f"pixels-{index}"] = collect(
                ROOT,
                f"pixels-{index}",
                [str(ROOT / "verify"), source, "1.25", "0.1"],
                env,
            )
        trace_env = env | {"FRAME_TEST_ARGS": env["FRAME_TEST_ARGS"] + " --repeat 1"}
        results["allocation-trace"] = collect(
            ROOT,
            "allocation-trace",
            [
                "strace",
                "-f",
                "-T",
                "-yy",
                "-e",
                "trace=ioctl",
                "-o",
                str(ROOT / "allocation.trace"),
                str(ROOT / "cases"),
                "--ignored",
                "--exact",
                "seek_cases::capture_pressure",
                "--nocapture",
                "--quiet",
                "--test-threads=1",
            ],
            trace_env,
        )
    finally:
        setting("--set-settings-string", key, original)
        manifest["restored_profile"] = setting("--settings-string", key)
        (ROOT / "manifest.json").write_text(json.dumps(manifest, indent=2) + "\n")
        (ROOT / "summary.json").write_text(json.dumps(results, indent=2) + "\n")
    return int(any(result["exit"] or result["failures"] for result in results.values()))


if __name__ == "__main__":
    sys.exit(main())
