"""Tests for craftcheck's file and dependency rules, on small generated repos."""

import sys
import tempfile
import textwrap
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))

import config  # noqa: E402
import deps_rules  # noqa: E402
import file_rules  # noqa: E402
import rust_source  # noqa: E402
from checker import run_check  # noqa: E402


def codes(findings):
    return sorted({f.code for f in findings})


def check_text(text, rel="crate/src/render_graph.rs", wiring=False, layer="domain"):
    src = rust_source.parse(textwrap.dedent(text))
    return file_rules.check_file(rel, src, wiring, config.Limits(), layer)


class SourceTest(unittest.TestCase):
    def test_strip_code_blanks_comments_and_strings(self):
        code = rust_source.strip_code(['let s = "{"; // }', "/* { */ let x = 1;"])
        self.assertNotIn("{", code[0])
        self.assertNotIn("}", code[0])
        self.assertEqual(code[1].strip(), "let x = 1;")

    def test_test_span_covers_the_test_module(self):
        src = rust_source.parse("//! doc\nfn a() {}\n#[cfg(test)]\nmod tests {\n    fn t() {}\n}\n")
        self.assertEqual(src.test_span, (2, 6))
        self.assertEqual(src.production_lines(), 3)


class FileRulesTest(unittest.TestCase):
    def test_clean_file_has_no_findings(self):
        findings = check_text("""\
            //! The render graph.

            /// A graph of passes.
            pub struct RenderGraph;

            impl RenderGraph {
                /// Number of passes.
                pub fn len(&self) -> usize {
                    0
                }
            }
            """)
        self.assertEqual(findings, [])

    def test_long_file_is_an_error(self):
        body = "//! doc\n" + "const X: u8 = 1;\n" * 401
        self.assertIn("C101", codes(check_text(body)))

    def test_logic_in_wiring_file_is_an_error(self):
        findings = check_text("//! doc\nmod a;\npub use a::A;\nfn helper() {}\n", rel="crate/src/lib.rs", wiring=True)
        self.assertEqual(codes(findings), ["C103"])

    def test_missing_module_doc(self):
        self.assertIn("D301", codes(check_text("pub struct RenderGraph;\n")))

    def test_todo_needs_issue(self):
        self.assertIn("D303", codes(check_text("//! d\n// TODO: later\n")))
        self.assertNotIn("D303", codes(check_text("//! d\n// TODO(#12): later\n")))

    def test_get_prefix_and_noise_words(self):
        found = codes(check_text("//! d\npub struct MeshManager;\nimpl MeshManager {\n    pub fn get_mesh(&self) {}\n}\n", rel="c/src/mesh_manager.rs"))
        self.assertIn("N202", found)
        self.assertIn("N203", found)
        self.assertNotIn("N203", codes(check_text("//! d\nfn get_mut(&mut self) {}\n")))

    def test_namespace_struct_and_globals(self):
        found = codes(check_text("//! d\npub struct GitUtil;\nimpl GitUtil {\n    pub fn run() {}\n}\nstatic STATE: Mutex<u8> = Mutex::new(0);\n", rel="c/src/git_util.rs"))
        self.assertIn("A502", found)
        self.assertIn("A501", found)

    def test_anyhow_allowed_only_in_interface(self):
        sig = "//! d\npub fn open() -> anyhow::Result<()> {\n    Ok(())\n}\n"
        self.assertIn("E404", codes(check_text(sig, layer="domain")))
        self.assertNotIn("E404", codes(check_text(sig, layer="interface")))

    def test_stringly_typed_code(self):
        found = codes(check_text('//! d\nfn f(ns: &str, p: &str) -> String {\n    format!("{ns}:{p}")\n}\nfn g(k: &str) {\n    match k {\n        "open" => {}\n        _ => {}\n    }\n}\n'))
        self.assertIn("S701", found)
        self.assertIn("S702", found)
        parser = '//! d\nimpl FromStr for Kind {\n    fn from_str(s: &str) {\n        match s {\n            "open" => {}\n        }\n    }\n}\n'
        self.assertNotIn("S702", codes(check_text(parser, rel="c/src/kind.rs")))

    def test_directory_rules(self):
        files = [f"c/src/audio/audio_{i}.rs" for i in range(13)] + ["c/src/audio/utils.rs"]
        found = codes(file_rules.check_dir("c/src/audio", files, config.Limits(), 1))
        self.assertEqual(found, ["C104", "C106", "N201"])


class DepsRulesTest(unittest.TestCase):
    def make_repo(self, root: Path):
        (root / "craftbook.toml").write_text(textwrap.dedent("""\
            [[crate]]
            name = "geo"
            layer = "kernel"
            context = "geometry"
            [[crate]]
            name = "phys-domain"
            layer = "domain"
            context = "physics"
            [[crate]]
            name = "phys-wgpu"
            layer = "infra"
            context = "physics"
            [[crate]]
            name = "audio-domain"
            layer = "domain"
            context = "audio"
            """))
        for name, deps in {"geo": [], "phys-domain": ["geo", "phys-wgpu", "audio-domain", "tokio"], "phys-wgpu": ["phys-domain"], "audio-domain": []}.items():
            crate = root / name
            (crate / "src").mkdir(parents=True)
            lines = "\n".join(f'{d} = {{ path = "../{d}" }}' if d != "tokio" else 'tokio = "1"' for d in deps)
            (crate / "Cargo.toml").write_text(f'[package]\nname = "{name}"\nversion = "0.1.0"\n\n[dependencies]\n{lines}\n')
            (crate / "src" / "lib.rs").write_text("//! d\n" + ("pub async fn tick() {}\n" if name == "phys-domain" else ""))

    def test_layer_context_and_io_rules(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            self.make_repo(root)
            findings = deps_rules.check_graph(root, deps_rules.discover(root), config.load(root))
        self.assertEqual(codes(findings), ["L602", "L603", "L604", "L605"])

    def test_run_check_walks_a_repo(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            self.make_repo(root)
            files, findings = run_check(root, config.load(root))
        self.assertEqual(len(files), 4)
        self.assertEqual(codes(findings), [])


if __name__ == "__main__":
    unittest.main()
