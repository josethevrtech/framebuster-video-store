import hashlib
import io
import os
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch
from urllib.error import HTTPError
import zipfile

from scripts import itch, package


class ReleaseTests(unittest.TestCase):
    def test_package_embeds_committed_source_without_working_tree_changes(self):
        artifacts = package.ROOT / "artifacts"
        artifacts.mkdir(exist_ok=True)
        root = Path(tempfile.mkdtemp(prefix="source-test-", dir=artifacts))
        files = {
            ".gitattributes": (package.ROOT / ".gitattributes").read_text(),
            ".gitignore": "/dist/\n",
            "src/main.rs": "fn main() {}\n",
            "Cargo.lock": "version = 4\n",
            "LICENSE": "license text\n",
            "build.sh": "build instructions\n",
            "README.user.md": "user guide\n",
            "deploy.py": "deployment script\n",
            "deploy.cmd": "deployment launcher\n",
            "launch.sh": "VR launch script\n",
            "scripts/install.py": "on-device installer\n",
        }
        for name, content in files.items():
            path = root / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(content)
        (root / "scripts/install.py").chmod(0o755)
        (root / "investigation/logs").mkdir(parents=True)
        (root / "investigation/logs/device.log").write_text("private diagnostics\n")
        subprocess.run(["git", "init", "-q", str(root)], check=True)
        subprocess.run(["git", "add", "."], cwd=root, check=True)
        subprocess.run([
            "git", "-c", "user.name=Test", "-c", "user.email=test@example.invalid",
            "-c", "commit.gpgsign=false", "commit", "-qm", "Source fixture",
        ], cwd=root, check=True)
        commit = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=root).strip()
        (root / "src/main.rs").write_text("uncommitted change\n")
        (root / "untracked.txt").write_text("not source\n")
        (root / "dist").mkdir()
        (root / "dist/ffmpeg").mkdir()
        (root / "dist/mesa").mkdir()
        (root / "dist/matineevr").write_bytes(b"test executable")
        (root / "dist/ffmpeg-7.0.tar.xz").write_bytes(b"FFmpeg source")
        with patch.object(package, "ROOT", root), \
                patch.object(package, "generate", return_value="license notices"):
            archive = package.package("test-dirty")
        with zipfile.ZipFile(archive) as bundle, \
                zipfile.ZipFile(io.BytesIO(bundle.read("source.zip"))) as source:
            self.assertEqual(bundle.read("install.py"), files["scripts/install.py"].encode())
            self.assertEqual(bundle.getinfo("install.py").external_attr >> 16 & 0o777, 0o755)
            self.assertEqual(source.comment, commit)
            self.assertFalse(any(name.startswith("investigation/") for name in source.namelist()))
            self.assertNotIn("ffmpeg-7.0.tar.xz", bundle.namelist())
            self.assertEqual(source.read("ffmpeg-7.0.tar.xz"), b"FFmpeg source")
            self.assertEqual({name for name in source.namelist() if not name.endswith("/")},
                             set(files) | {"ffmpeg-7.0.tar.xz"})
            for name, content in files.items():
                self.assertEqual(source.read(name), content.encode())

    def test_dirty_checkout_stops_before_packaging_or_publishing(self):
        for status in (" M src/main.rs", "M  Cargo.lock", "?? src/new.rs"):
            with self.subTest(status=status), \
                    patch("sys.argv", ["package.py", "--publish"]), \
                    patch.object(package, "git", side_effect=["abc", status]), \
                    patch.object(package, "package") as bundle, \
                    patch.object(package, "publish") as publish, \
                    patch.object(itch, "publish") as upload:
                with self.assertRaisesRegex(ValueError, "clean Git working tree"):
                    package.main()
                bundle.assert_not_called()
                publish.assert_not_called()
                upload.assert_not_called()

    def test_notes_select_exact_version_and_preserve_markdown(self):
        changelog = "# 0.0.40 - Newer\nNew\n\n# 0.0.4 - Release\n\n## Fixes\n- Café\n\n# 0.0.3 - Older\nOld"
        expected = "# 0.0.4 - Release\n\n## Fixes\n- Café"
        for tag in ("0.0.4", "v0.0.4"):
            with self.subTest(tag=tag):
                self.assertEqual(package.release_notes(changelog, tag), expected)
        self.assertEqual(package.release_notes(changelog, "0.0.3"), "# 0.0.3 - Older\nOld")

    def test_invalid_notes_fail(self):
        for changelog in (
            "# 0.0.40 - Wrong version\nNotes",
            "# 0.0.4 - Empty\n\n# 0.0.3 - Older\nNotes",
            "# 0.0.4 - Duplicate\nNotes\n# 0.0.4 - Duplicate\nNotes",
        ):
            with self.subTest(changelog=changelog), self.assertRaises(ValueError):
                package.release_notes(changelog, "0.0.4")

    def test_forgejo_creates_release_with_notes_and_uploads_archive(self):
        missing = HTTPError("https://forgejo/releases/tags/0.0.4", 404, "missing", {}, None)
        archive = Path(__file__)
        with patch.object(package, "request", side_effect=[
            missing, {"id": 4, "body": "notes"}, [], {"browser_download_url": "https://forgejo/build.zip"}
        ]) as request, patch("builtins.print"):
            package.publish(archive, "0.0.4", "notes")
        self.assertEqual(request.call_args_list[1].args, (
            "/releases", {"tag_name": "0.0.4", "name": "0.0.4", "body": "notes"}
        ))
        self.assertIn("/releases/4/assets?name=", request.call_args_list[-1].args[0])

    def test_forgejo_retry_updates_notes_and_keeps_asset(self):
        archive = Path("build.zip")
        with patch.object(package, "request", side_effect=[
            {"id": 4, "body": "old"}, {},
            [{"name": archive.name, "browser_download_url": "https://forgejo/build.zip"}],
        ]) as request, patch("builtins.print"):
            package.publish(archive, "0.0.4", "notes")
        self.assertEqual(request.call_count, 3)
        self.assertEqual(request.call_args_list[1].kwargs, {"method": "PATCH"})
        self.assertEqual(request.call_args_list[1].args, ("/releases/4", {"body": "notes"}))

    def test_itch_runs_only_after_successful_forgejo_publication(self):
        for failure in (None, RuntimeError("Forgejo unavailable")):
            with self.subTest(failure=failure), \
                    patch("sys.argv", ["package.py", "--publish"]), \
                    patch.dict(os.environ, {"GITHUB_REF": "refs/tags/0.0.4", "BUTLER_API_KEY": "test"}), \
                    patch.object(package, "git", side_effect=["abc", "", "commit", "commit"]), \
                    patch.object(Path, "read_text", return_value="# 0.0.4 - Release\nNotes"), \
                    patch.object(package, "package", return_value=Path("build.zip")), \
                    patch.object(package, "publish", side_effect=failure), \
                    patch.object(itch, "publish") as upload, patch("builtins.print"):
                if failure:
                    with self.assertRaises(RuntimeError):
                        package.main()
                    upload.assert_not_called()
                else:
                    package.main()
                    upload.assert_called_once_with(Path("build.zip"), "0.0.4")

    def test_missing_secret_stops_before_packaging(self):
        with patch("sys.argv", ["package.py", "--publish"]), \
                patch.dict(os.environ, {"GITHUB_REF": "refs/tags/0.0.4", "BUTLER_API_KEY": ""}), \
                patch.object(package, "git", side_effect=["abc", "", "commit", "commit"]), \
                patch.object(Path, "read_text", return_value="# 0.0.4 - Release\nNotes"), \
                patch.object(package, "package") as bundle, self.assertRaisesRegex(ValueError, "BUTLER_API_KEY"):
            package.main()
        bundle.assert_not_called()

    def test_butler_receives_archive_target_and_tag_and_propagates_failure(self):
        archive = Path("/release/build.zip")
        with patch.object(itch, "install", return_value=Path("/release/butler")), \
                patch.object(itch.subprocess, "run", side_effect=subprocess.CalledProcessError(1, "butler")) as run:
            with self.assertRaises(subprocess.CalledProcessError):
                itch.publish(archive, "0.0.4")
        run.assert_called_once_with([
            "/release/butler", "push", str(archive), itch.TARGET, "--userversion", "0.0.4"
        ], check=True)

    def test_butler_download_is_verified_before_installation(self):
        artifacts = package.ROOT / "artifacts"
        artifacts.mkdir(exist_ok=True)
        directory = Path(tempfile.mkdtemp(prefix="itch-test-", dir=artifacts))
        data = io.BytesIO()
        with zipfile.ZipFile(data, "w") as archive:
            archive.writestr("butler", b"test executable")
        downloaded = data.getvalue()
        with patch.object(itch, "urlopen", return_value=io.BytesIO(downloaded)):
            with self.assertRaisesRegex(ValueError, "checksum"):
                itch.install(directory)
        self.assertFalse((directory / "butler").exists())
        with patch.object(itch, "urlopen", return_value=io.BytesIO(downloaded)), \
                patch.object(itch, "BUTLER_SHA256", hashlib.sha256(downloaded).hexdigest()):
            executable = itch.install(directory)
        self.assertEqual(executable.read_bytes(), b"test executable")
        self.assertEqual(executable.stat().st_mode & 0o777, 0o755)


if __name__ == "__main__":
    unittest.main()
