"""Tests for the crate-graph rules and the repository walk, on generated repos."""

import contextlib
import importlib.util
import io
import subprocess
import tempfile
import unittest
from pathlib import Path

from support import codes, config, with_code, write

import deps_rules
import io_rules
import rust_source
import workspace
from checker import is_wiring, run_check

MAIN = Path(__file__).resolve().parent.parent / "__main__.py"


def crate_toml(crates):
    return "".join(f'[[crate]]\nname = "{n}"\nlayer = "{layer}"\ncontext = "{ctx}"\n{extra}' for n, layer, ctx, extra in crates)


def make_crates(root: Path, deps: dict, sources: dict = None):
    for name, crate_deps in deps.items():
        lines = "\n".join(f'{d} = {{ path = "../{d}" }}' if d != "tokio" else 'tokio = "1"' for d in crate_deps)
        write(root, f"{name}/Cargo.toml", f'[package]\nname = "{name}"\nversion = "0.1.0"\n\n[dependencies]\n{lines}\n')
        write(root, f"{name}/src/lib.rs", "//! d\n" + (sources or {}).get(name, ""))


def graph(root: Path):
    return deps_rules.check_graph(root, workspace.discover(root), config.load(root))


class DepsRulesTest(unittest.TestCase):
    def make_repo(self, root: Path):
        write(root, "craftbook.toml", crate_toml([
            ("geo", "shared-kernel", "geometry", ""), ("phys-domain", "domain", "physics", ""),
            ("phys-wgpu", "infra", "physics", ""), ("audio-domain", "domain", "audio", ""),
        ]))
        make_crates(root, {"geo": [], "phys-domain": ["geo", "phys-wgpu", "audio-domain", "tokio"], "phys-wgpu": ["phys-domain"], "audio-domain": []},
                    {"phys-domain": "pub async fn tick() {}\n"})

    def test_layer_context_and_io_rules(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            self.make_repo(root)
            self.assertEqual(codes(graph(root)), ["L602", "L603", "L604", "L605"])

    def test_domain_to_domain_needs_customer_supplier(self):
        for relation, expected in (("acl", ["L603"]), ("published-language", ["L603"]), ("customer-supplier", []), ("shared-kernel", []), (None, ["L603"])):
            with tempfile.TemporaryDirectory() as tmp:
                root = Path(tmp)
                edge = f'[[context_edge]]\nfrom = "physics"\nto = "audio"\nrelation = "{relation}"\n' if relation else ""
                write(root, "craftbook.toml", crate_toml([("p", "domain", "physics", ""), ("a", "domain", "audio", "")]) + edge)
                make_crates(root, {"p": ["a"], "a": []})
                found = graph(root)
                self.assertEqual(codes(found), expected, relation)
                if expected:
                    self.assertIn("must be customer-supplier", found[0].message)

    def test_app_to_app_across_contexts_needs_any_edge(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write(root, "craftbook.toml", crate_toml([("p", "app", "physics", ""), ("a", "app", "audio", "")]))
            make_crates(root, {"p": ["a"], "a": []})
            self.assertEqual(codes(graph(root)), ["L603"])
            with open(root / "craftbook.toml", "a") as fh:
                fh.write('[[context_edge]]\nfrom = "physics"\nto = "audio"\nrelation = "acl"\n')
            self.assertEqual(codes(graph(root)), [])

    def test_io_source_reads_code_only(self):
        text = '//! d\nuse std::{fs, io};\nuse std::thread::{sleep, spawn};\n// std::fs is for adapters\nconst S: &str = "tokio::spawn";\n#[cfg(test)]\nmod tests {\n    use std::fs;\n}\n'
        found = io_rules.io_findings("c/src/lib.rs", rust_source.parse(text), "domain")
        self.assertEqual(sorted((f.line, f.subject) for f in found), [(2, "std::fs"), (3, "std::thread::sleep")])

    def test_workspace_without_lints_is_one_warning(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write(root, "Cargo.toml", '[workspace]\nmembers = ["a", "b"]\n')
            make_crates(root, {"a": [], "b": []})
            self.assertEqual([(f.code, f.level) for f in graph(root)], [("L607", "warning")])
            write(root, "Cargo.toml", '[workspace]\nmembers = ["a", "b"]\n[workspace.lints.rust]\nunsafe_code = "forbid"\n')
            self.assertEqual([f.code for f in graph(root)], ["L606", "L606"])

    def test_discover_reads_workspace_members(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write(root, "Cargo.toml", '[workspace]\nmembers = ["crates/*"]\nexclude = ["crates/old"]\n[package]\nname = "top"\nversion = "0.1.0"\n')
            make_crates(root / "crates", {"a": [], "old": []})
            make_crates(root / "fuzz", {"fz": []})
            self.assertEqual(sorted(p.name for p in workspace.discover(root)), ["a", "top"])
            (root / "Cargo.toml").unlink()
            self.assertEqual(sorted(p.name for p in workspace.discover(root)), ["a", "fz", "old"])


class RepoWalkTest(unittest.TestCase):
    def test_run_check_walks_a_repo(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write(root, "craftbook.toml", crate_toml([("geo", "shared-kernel", "geometry", "")]))
            make_crates(root, {"geo": [], "phys": []})
            files, findings = run_check(root, config.load(root))
        self.assertEqual(len(files), 2)
        self.assertEqual(codes(findings), [])

    def test_perf_profile_and_exempt_codes(self):
        body = "//! d\npub struct Ns;\nimpl Ns {\n    pub fn run() {}\n}\npub struct Other;\nstatic mut N: u8 = 0;\n"
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write(root, "craftbook.toml", crate_toml([("k", "shared-kernel", "g", 'profile = "perf"\n'), ("d", "domain", "g", "")])
                  + '[[exempt]]\npath = "d/src/ns.rs"\nreason = "test"\ncodes = ["A501"]\n')
            make_crates(root, {"k": [], "d": []})
            write(root, "k/src/ns.rs", body)
            write(root, "d/src/ns.rs", body)
            _, findings = run_check(root, config.load(root))
        by_file = {rel: codes(f for f in findings if f.path == rel) for rel in ("k/src/ns.rs", "d/src/ns.rs")}
        self.assertEqual(by_file, {"k/src/ns.rs": ["A501"], "d/src/ns.rs": ["A502", "C108"]})

    def test_module_depth_is_counted_from_the_crate_src(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            make_crates(root / "src", {"c": []})
            write(root, "src/c/src/a/b/c/d/e/x.rs", "//! d\n")
            _, findings = run_check(root, config.load(root))
        self.assertEqual([f.message.split(" (")[0] for f in with_code(findings, "C105")], ["module depth 5"])

    def test_lib_next_to_main_is_not_wiring(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write(root, "c/src/lib.rs", "//! d\n")
            write(root, "c/src/main.rs", "//! d\n")
            write(root, "c/src/assets/shader.wgsl", "")
            self.assertFalse(is_wiring(root / "c/src/lib.rs"))
            write(root, "c/src/scene.rs", "//! d\n")
            self.assertTrue(is_wiring(root / "c/src/lib.rs"))

    def test_git_file_names_with_unusual_characters(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            make_crates(root, {"c": []})
            write(root, "c/src/naïve file.rs", "//! d\n")
            subprocess.run(["git", "init", "-q", str(root)], check=True)
            subprocess.run(["git", "-C", str(root), "add", "."], check=True)
            files, _ = run_check(root, config.load(root))
        self.assertIn("c/src/naïve file.rs", files)


class CratesCommandTest(unittest.TestCase):
    def run_main(self, argv):
        spec = importlib.util.spec_from_file_location("craftcheck_main", MAIN)
        module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)
        out = io.StringIO()
        with contextlib.redirect_stdout(out):
            status = module.main(argv)
        return status, out.getvalue()

    def test_lists_crates_of_a_layer(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write(root, "cb.toml", crate_toml([("geo", "shared-kernel", "g", ""), ("phys", "domain", "p", ""), ("ids", "shared-kernel", "g", "")]))
            self.assertEqual(self.run_main(["crates", tmp, "--config", str(root / "cb.toml"), "--layer", "shared-kernel"]), (0, "geo\nids\n"))
            self.assertEqual(self.run_main(["crates", tmp, "--config", str(root / "cb.toml"), "--layer", "infra"]), (0, ""))


if __name__ == "__main__":
    unittest.main()
