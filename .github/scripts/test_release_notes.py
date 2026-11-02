import importlib.util
import unittest
from pathlib import Path

SCRIPT = Path(__file__).with_name("release_notes.py")
spec = importlib.util.spec_from_file_location("release_notes", SCRIPT)
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)


class ReleaseNotesTests(unittest.TestCase):
    def test_full_tag_section(self):
        text = "# Changelog\n\n## v2.0.0-rc8 — 2026-11-02\n\n- New Action\n\n## v2-rc7\n- Previous\n"
        self.assertEqual(module.changelog_section(text, "v2.0.0-rc8"), "- New Action")

    def test_legacy_heading(self):
        self.assertEqual(module.changelog_section("## v2-rc7\n\n- Fix\n", "v2.0.0-rc7"), "- Fix")

    def test_missing_release(self):
        with self.assertRaises(ValueError):
            module.changelog_section("## Unreleased\n- A\n", "v2.0.0-rc9")

    def test_empty_release(self):
        with self.assertRaises(ValueError):
            module.changelog_section("## v2.0.0-rc8\n\n## Other\n", "v2.0.0-rc8")


if __name__ == "__main__":
    unittest.main()
