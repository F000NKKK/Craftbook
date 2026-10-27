"""Tests for the crate README rules."""

import tempfile
import unittest
from pathlib import Path

from support import codes

import readme_rules
import workspace


def make(root: Path, name: str, readme: str | None):
    crate = root / "crates" / name
    (crate / "src").mkdir(parents=True)
    (crate / "Cargo.toml").write_text(f'[package]\nname = "{name}"\nversion = "0.1.0"\n')
    if readme is not None:
        (crate / "README.md").write_text(readme)


class ReadmeRulesTest(unittest.TestCase):
    def test_missing_and_misnamed(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            (root / "Cargo.toml").write_text('[workspace]\nmembers = ["crates/*"]\n')
            make(root, "good", "# good\n\nText.\n")
            make(root, "bare", None)
            make(root, "wrong", "# other-name\n")
            findings = readme_rules.check_readmes(root, workspace.discover(root))
        self.assertEqual(codes(findings), ["R801", "R802"])
        self.assertEqual(sorted(f.subject for f in findings), ["bare", "wrong"])


if __name__ == "__main__":
    unittest.main()
