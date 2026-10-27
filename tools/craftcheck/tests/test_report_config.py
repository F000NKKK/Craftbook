"""Tests for the baseline and craftbook.toml loading."""

import tempfile
import textwrap
import unittest
from pathlib import Path

from support import config

import report
from report import Finding


def load(text):
    with tempfile.TemporaryDirectory() as tmp:
        (Path(tmp) / "craftbook.toml").write_text(textwrap.dedent(text))
        return config.load(Path(tmp))


class BaselineTest(unittest.TestCase):
    def test_key_ignores_numbers_and_counts_per_key(self):
        old = [Finding("a.rs", 3, "C101", "error", "410 production lines"), Finding("a.rs", 9, "C108", "warning", "x", "Pass")]
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "baseline"
            report.write_baseline(path, old + [Finding("a.rs", 20, "C108", "warning", "x", "Pass")])
            self.assertEqual(path.read_text(), "a.rs\tC101\t\t1\na.rs\tC108\tPass\t2\n")
            baseline = report.read_baseline(path)
        grown = [Finding("a.rs", 5, "C101", "error", "420 production lines")]
        self.assertEqual(report.new_findings(grown, baseline), [])
        three = [Finding("a.rs", n, "C108", "warning", "x", "Pass") for n in (1, 2, 3)]
        self.assertEqual(len(report.new_findings(three, baseline)), 3)
        self.assertEqual(report.new_findings(three[:2], baseline), [])

    def test_malformed_baseline_is_rejected(self):
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "baseline"
            path.write_text("a.rs\tC101\tmessage with 3 numbers\n")
            with self.assertRaises(ValueError):
                report.read_baseline(path)


class ConfigTest(unittest.TestCase):
    def test_limits_may_only_go_down(self):
        self.assertEqual(load("[limits]\nfile_lines = 300\n").limits.file_lines, 300)
        with self.assertRaisesRegex(ValueError, "above Craftbook's 400"):
            load("[limits]\nfile_lines = 500\n")
        with self.assertRaisesRegex(ValueError, "unknown limit"):
            load("[limits]\nlines = 5\n")

    def test_exempt_needs_a_reason(self):
        with self.assertRaisesRegex(ValueError, "reason"):
            load('[[exempt]]\npath = "a.rs"\nreason = "  "\n')
        with self.assertRaisesRegex(ValueError, "reason"):
            load('[[exempt]]\npath = "a.rs"\n')

    def test_exempt_codes(self):
        cfg = load('[[exempt]]\npath = "src/log.rs"\nreason = "logger"\ncodes = ["A501"]\n[[exempt]]\npath = "gen/*"\nreason = "vendored"\n')
        self.assertTrue(cfg.exempted("src/log.rs", "A501"))
        self.assertFalse(cfg.exempted("src/log.rs", "D301"))
        self.assertFalse(cfg.fully_exempt("src/log.rs"))
        self.assertTrue(cfg.fully_exempt("gen/x.rs"))
        self.assertTrue(cfg.exempted("gen/x.rs", "C101"))
        with self.assertRaisesRegex(ValueError, "codes"):
            load('[[exempt]]\npath = "a.rs"\nreason = "r"\ncodes = "A501"\n')

    def test_layer_and_profile_names(self):
        cfg = load('[[crate]]\nname = "geo"\nlayer = "shared-kernel"\nprofile = "perf"\n')
        self.assertEqual(cfg.crates["geo"].profile, "perf")
        with self.assertRaisesRegex(ValueError, "renamed to 'shared-kernel'"):
            load('[[crate]]\nname = "geo"\nlayer = "kernel"\n')
        with self.assertRaisesRegex(ValueError, "renamed to 'perf'"):
            load('[[crate]]\nname = "geo"\nlayer = "domain"\nprofile = "kernel"\n')

    def test_edges_keep_their_relation(self):
        cfg = load('[[context_edge]]\nfrom = "a"\nto = "b"\nrelation = "acl"\n')
        self.assertEqual(cfg.edges, {("a", "b"): "acl"})


if __name__ == "__main__":
    unittest.main()
