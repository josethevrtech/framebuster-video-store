import json
from pathlib import Path
import tempfile
import unittest

from scripts.licenses import PlainText, crate_notices


class LicenseTests(unittest.TestCase):
    def setUp(self):
        artifacts = Path(__file__).resolve().parents[1] / "artifacts"
        artifacts.mkdir(exist_ok=True)
        self.root = Path(tempfile.mkdtemp(prefix="licenses-", dir=artifacts))
        self.crate = {"name": "example", "version": "1", "license": "MIT",
                      "manifest_path": str(self.root / "Cargo.toml")}

    def test_preserves_copyright_and_notice_texts(self):
        (self.root / "LICENSE-MIT").write_text("Copyright Example\nPermission granted")
        (self.root / "NOTICE").write_text("Additional attribution")
        text = crate_notices(self.crate, self.root)
        self.assertIn("Copyright Example\nPermission granted", text)
        self.assertIn("Additional attribution", text)

    def test_missing_text_fails_instead_of_shipping_incomplete_notices(self):
        with self.assertRaisesRegex(ValueError, "Missing license text for example 1"):
            crate_notices(self.crate, self.root)

    def test_openxr_fallback_requires_the_verified_revision(self):
        self.crate["name"] = "openxr"
        (self.root / "licenses").mkdir()
        (self.root / "licenses/openxrs-MIT.txt").write_text("Copyright openxrs")
        vcs = self.root / ".cargo_vcs_info.json"
        vcs.write_text(json.dumps({"git": {"sha1": "f50cf13afc047f96d540d9d85a08e023e0d3647d"}}))
        self.assertIn("Copyright openxrs", crate_notices(self.crate, self.root))
        vcs.write_text(json.dumps({"git": {"sha1": "different revision"}}))
        with self.assertRaisesRegex(ValueError, "Missing license text"):
            crate_notices(self.crate, self.root)

    def test_rust_notices_keep_paragraphs_and_decode_entities(self):
        parser = PlainText()
        parser.feed("<p>Copyright A &amp; B</p><pre>Permission\ngranted</pre>")
        self.assertEqual(parser.parts, ["Copyright A & B", "Permission\ngranted"])
