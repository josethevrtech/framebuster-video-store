import argparse
import contextlib
import io
import json
import os
from pathlib import Path
import shlex
import shutil
import signal
import subprocess
import tempfile
import time
import unittest
from unittest.mock import patch
import zipfile

import deploy
from scripts import package


class DeploymentTests(unittest.TestCase):
    def setUp(self):
        self.sleep = shutil.which("sleep")
        self.fuser = shutil.which("fuser")
        artifacts = deploy.ROOT / "artifacts"
        artifacts.mkdir(exist_ok=True)
        self.root = Path(tempfile.mkdtemp(prefix="deploy test ", dir=artifacts))
        (self.root / "matineevr").write_bytes(b"test executable")
        (self.root / "ffmpeg").mkdir()
        (self.root / "ffmpeg/libavcodec.so.61").write_bytes(b"test library")
        (self.root / "mesa").mkdir()
        self.args = argparse.Namespace(host="frame", key=None, launch=False)
        self.stdout = io.StringIO()
        self.enterContext(patch.object(deploy, "ROOT", self.root))
        self.enterContext(patch.object(deploy.shutil, "which", return_value="installed"))
        self.enterContext(contextlib.redirect_stdout(self.stdout))
        self.run = self.enterContext(patch.object(deploy.subprocess, "run"))
        self.run.side_effect = [
            subprocess.CompletedProcess([], 0, "/home/steamos/devkit-game/MatineeVR\n"),
            subprocess.CompletedProcess([], 0),
            subprocess.CompletedProcess([], 0, '{"success": true}'),
            subprocess.CompletedProcess([], 0, "success"),
        ]

    def test_packaged_deployment_preserves_paths_arguments_and_launch(self):
        self.args.launch = True
        self.args.key = str(self.root / "key with spaces")
        Path(self.args.key).write_text("test key")
        arguments = ["/home/steamos/Videos/café's $(touch nope); hiking.mp4", "--stereo", "sbs"]
        deploy.deploy(self.args, arguments)
        calls = self.run.call_args_list
        self.assertEqual(calls[1].args[0][-5:], ["./matineevr", "./launch.sh", "./ffmpeg", "./mesa", "steamos@frame:devkit-game/MatineeVR/"])
        self.assertEqual(calls[1].kwargs["cwd"], self.root)
        self.assertIn(self.args.key, calls[1].args[0])
        command = calls[2].args[0][-1].split(" && ", 1)[1]
        request = json.loads(shlex.split(command)[-1])
        self.assertEqual(request["argv"], ["./launch.sh", *arguments])
        self.assertEqual(request["directory"], "/home/steamos/devkit-game/MatineeVR")
        self.assertEqual(request["settings"], {"steam_play": "0", "compat_tool": ""})
        self.assertIn("run-game gameid=MatineeVR", calls[3].args[0][-1])

    def test_failed_preparation_and_copy_stop_deployment(self):
        for completed_steps in (0, 1):
            with self.subTest(completed_steps=completed_steps):
                self.run.reset_mock()
                self.run.side_effect = [
                    subprocess.CompletedProcess([], 0, "/home/steamos/devkit-game/MatineeVR")
                ] * completed_steps + [subprocess.CalledProcessError(1, "ssh/scp")]
                with self.assertRaises(subprocess.CalledProcessError):
                    deploy.deploy(self.args, [])
                self.assertEqual(self.run.call_count, completed_steps + 1)

    def test_preparation_stops_only_the_installed_player_and_also_accepts_idle(self):
        if not self.sleep or not self.fuser:
            self.skipTest("requires sleep and fuser")
        deploy.deploy(self.args, [])
        command = self.run.call_args_list[0].args[0][-1].replace(
            '"$HOME/', shlex.quote(str(self.root)) + '/"')
        shortcut = self.root / "devkit-utils/steam-client-create-shortcut"
        shortcut.parent.mkdir()
        shortcut.touch()
        installed = self.root / deploy.DIRECTORY / "matineevr"
        installed.parent.mkdir(parents=True)
        other = self.root / "matineevr"
        for path in (installed, other):
            shutil.copy2(self.sleep, path)
        processes = [subprocess.Popen([str(path), "30"]) for path in (installed, other)]
        try:
            deadline = time.monotonic() + 5
            while True:
                with subprocess.Popen([self.fuser, str(installed)], stdout=subprocess.DEVNULL,
                                      stderr=subprocess.DEVNULL) as probe:
                    if probe.wait(timeout=2) == 0:
                        break
                self.assertLess(time.monotonic(), deadline)
                time.sleep(0.01)
            for _ in range(2):
                with subprocess.Popen(["sh", "-c", command], stdout=subprocess.PIPE,
                                      stderr=subprocess.PIPE, text=True) as remote:
                    stdout, stderr = remote.communicate(timeout=15)
                    self.assertEqual(remote.returncode, 0, stderr)
                    self.assertEqual(stdout.strip(), str(installed.parent))
                self.assertEqual(processes[0].wait(timeout=2), -signal.SIGKILL)
                self.assertIsNone(processes[1].poll())
        finally:
            for process in processes:
                process.kill()
                process.wait(timeout=2)

    def test_registration_error_prevents_launch(self):
        self.args.launch = True
        self.run.side_effect = [
            subprocess.CompletedProcess([], 0, "/home/steamos/devkit-game/MatineeVR"),
            subprocess.CompletedProcess([], 0),
            subprocess.CompletedProcess([], 0, '{"error":"Steam is not running"}'),
        ]
        with self.assertRaisesRegex(RuntimeError, "Steam is not running"):
            deploy.deploy(self.args, [])
        self.assertEqual(self.run.call_count, 3)

    def test_invalid_key_stops_before_connecting(self):
        self.args.key = str(self.root / "missing key")
        with self.assertRaisesRegex(FileNotFoundError, "SSH key not found"):
            deploy.deploy(self.args, [])
        self.run.assert_not_called()

    def test_key_locations_match_devkit_client(self):
        for platform, relative, environment in [
            ("win32", "Local/steamos-devkit/steamos-devkit/devkit_rsa", {"LOCALAPPDATA": str(self.root / "Local")}),
            ("linux", "config/steamos-devkit/devkit_rsa", {"XDG_CONFIG_HOME": str(self.root / "config")}),
            ("darwin", "Library/Application Support/steamos-devkit/devkit_rsa", {}),
        ]:
            key = self.root / relative
            key.parent.mkdir(parents=True)
            key.write_text("test key")
            with self.subTest(platform=platform), patch.object(deploy.sys, "platform", platform), \
                    patch.object(Path, "home", return_value=self.root), patch.dict(os.environ, environment):
                self.assertEqual(deploy.devkit_key(), str(key))

    def test_cli_reports_failure_without_traceback(self):
        error = io.StringIO()
        with patch.object(deploy.sys, "argv", ["deploy.py", "--key", str(self.root / "missing")]), \
                contextlib.redirect_stderr(error):
            self.assertEqual(deploy.main(), 1)
        self.assertIn("Deployment failed: SSH key not found", error.getvalue())
        self.assertNotIn("Traceback", error.getvalue())

    def test_cli_keeps_remote_commands_out_of_error_messages(self):
        self.run.side_effect = subprocess.CalledProcessError(255, ["ssh", "remote script"])
        error = io.StringIO()
        with patch.object(deploy.sys, "argv", ["deploy.py", "--key", ""]), \
                contextlib.redirect_stderr(error):
            self.assertEqual(deploy.main(), 1)
        self.assertEqual(error.getvalue(), "Deployment failed: ssh exited with code 255.\n")

    def test_package_contains_binary_guide_and_launchers(self):
        (self.root / "dist/ffmpeg").mkdir(parents=True)
        (self.root / "dist/ffmpeg/libavcodec.so.61").write_bytes(b"test library")
        (self.root / "dist/ffmpeg-7.0.tar.xz").write_bytes(b"FFmpeg source")
        (self.root / "dist/mesa").mkdir()
        (self.root / "dist/mesa/libvulkan_freedreno.so").write_bytes(b"private driver")
        (self.root / "dist/mesa/freedreno_icd.aarch64.json").write_text("{}")
        (self.root / "dist/mesa/source.tar.xz").write_bytes(b"Mesa source")
        binary = self.root / "dist/matineevr"
        binary.write_bytes(b"test executable")
        binary.chmod(0o755)
        for name in ("README.user.md", "deploy.py", "deploy.cmd", "launch.sh"):
            (self.root / name).write_text(name)
        (self.root / "scripts").mkdir()
        (self.root / "scripts/install.py").write_text("installer")
        source = io.BytesIO()
        with zipfile.ZipFile(source, "w") as sources:
            sources.writestr("src/main.rs", "source code")
        with patch.object(package, "ROOT", self.root), \
                patch.object(package.subprocess, "check_output", return_value=source.getvalue()), \
                patch.object(package, "generate", return_value="license notices"):
            archive = package.package("test")
        self.assertEqual(archive.name, "matineevr-test-aarch64-unknown-linux-gnu.zip")
        with zipfile.ZipFile(archive) as bundle:
            self.assertEqual(bundle.namelist(), [
                "matineevr", "ffmpeg/libavcodec.so.61", "mesa/freedreno_icd.aarch64.json",
                "mesa/libvulkan_freedreno.so", "mesa/source.tar.xz",
                "README.md", "LICENSES", "source.zip", "deploy.py", "deploy.cmd", "launch.sh", "install.py"
            ])
            self.assertEqual(bundle.read("README.md"), b"README.user.md")
            self.assertEqual(bundle.read("LICENSES"), b"license notices")
            with zipfile.ZipFile(io.BytesIO(bundle.read("source.zip"))) as sources:
                self.assertEqual(sources.read("src/main.rs"), b"source code")
                self.assertEqual(sources.read("ffmpeg-7.0.tar.xz"), b"FFmpeg source")
            self.assertTrue(bundle.getinfo("matineevr").external_attr >> 16 & 0o111)


if __name__ == "__main__":
    unittest.main()
