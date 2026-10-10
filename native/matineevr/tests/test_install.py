import unittest
from pathlib import Path
from unittest.mock import patch

from scripts import install


class InstallTests(unittest.TestCase):
    def test_incomplete_package_fails_before_registration(self):
        with (
            patch.object(Path, "is_file", return_value=False),
            self.assertRaisesRegex(FileNotFoundError, "Extract the complete ZIP"),
        ):
            install.install(Path("/package"))

    def test_unsafe_launch_paths_fail_before_registration(self):
        with patch.object(Path, "is_file", return_value=True):
            for character in '\\"$`\n\r':
                with (
                    self.subTest(character=character),
                    self.assertRaisesRegex(ValueError, "folder path"),
                ):
                    install.install(Path("/package" + character))

    def test_registration_error_is_reported_in_desktop_dialog(self):
        with (
            patch.object(
                install, "install", side_effect=RuntimeError("Steam unavailable")
            ),
            patch.dict(install.os.environ, {"DISPLAY": ":0"}),
            patch.object(install.subprocess, "run") as dialog,
            patch.object(install.logging, "getLogger"),
            patch("builtins.print"),
        ):
            self.assertEqual(install.main(), 1)
        dialog.assert_called_once_with(
            [
                "kdialog",
                "--title",
                "MatineeVR",
                "--error",
                "Installation failed: Steam unavailable",
            ],
            check=True,
        )


if __name__ == "__main__":
    unittest.main()
