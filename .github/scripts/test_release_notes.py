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

PREPARE = Path(__file__).with_name("prepare_release.py")
prep_spec = importlib.util.spec_from_file_location("prepare_release", PREPARE)
prepare_release = importlib.util.module_from_spec(prep_spec)
prep_spec.loader.exec_module(prepare_release)


class PrepareReleaseTests(unittest.TestCase):
    def test_version_channels_and_bumps(self):
        cases = [
            ("2.0.0-rc8", "current", "rc", "2.0.0-rc9"),
            ("2.0.0-rc8", "current", "stable", "2.0.0"),
            ("2.0.0-beta3", "current", "rc", "2.0.0-rc1"),
            ("2.0.0-alpha1", "current", "beta", "2.0.0-beta1"),
            ("2.0.0-rc8", "patch", "alpha", "2.0.1-alpha1"),
            ("2.0.0-rc8", "minor", "beta", "2.1.0-beta1"),
            ("2.0.0-rc8", "major", "stable", "3.0.0"),
            ("2.0.0", "patch", "rc", "2.0.1-rc1"),
        ]
        for version, bump, channel, expected in cases:
            with self.subTest(version=version, bump=bump, channel=channel):
                self.assertEqual(prepare_release.next_version(version, bump, channel), expected)

    def test_reject_channel_downgrade(self):
        with self.assertRaises(ValueError):
            prepare_release.next_version("2.0.0-rc8", "current", "alpha")

    def test_close_unreleased(self):
        before = "# Changelog\n\n## Unreleased\n\n- Improvement\n\n## v2.0.0-rc8\n\n- Old\n"
        after = prepare_release.prepare_changelog(before, "2.0.0-rc9", "2026-11-02")
        self.assertIn("## Unreleased\n\n## v2.0.0-rc9 — 2026-11-02\n\n- Improvement", after)
        self.assertEqual(after.count("## Unreleased"), 1)
        with self.assertRaises(ValueError):
            prepare_release.prepare_changelog(after, "2.0.0-rc10", "2026-11-03")
