"""Tests for craftcheck's file rules on small source texts."""

import unittest

from support import check_text, codes, config, file_rules, with_code


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

    def test_wiring_file_allows_pub_type_and_macro_rules(self):
        text = "//! doc\nmod a;\npub type Result<T> = std::result::Result<T, a::Error>;\nmacro_rules! m { () => {} }\n"
        self.assertEqual(check_text(text, rel="crate/src/lib.rs", wiring=True), [])
        for item in ("const X: u8 = 1;", "static Y: u8 = 1;", "trait T {}", "enum E {}", "impl A {}", "type Private = u8;"):
            self.assertIn("C103", codes(check_text(f"//! doc\nmod a;\n{item}\n", rel="crate/src/lib.rs", wiring=True)), item)

    def test_missing_module_doc(self):
        self.assertIn("D301", codes(check_text("pub struct RenderGraph;\n")))

    def test_licence_lines_may_precede_the_module_doc(self):
        text = "// SPDX-License-Identifier: MPL-2.0\n// Copyright 2026 Someone\n\n//! The render graph.\npub struct RenderGraph;\n"
        self.assertNotIn("D301", codes(check_text(text)))
        self.assertIn("D301", codes(check_text("/// item doc\n//! late\npub struct RenderGraph;\n")))

    def test_todo_needs_issue(self):
        self.assertIn("D303", codes(check_text("//! d\n// TODO: later\n")))
        self.assertNotIn("D303", codes(check_text("//! d\n// TODO(#12): later\n")))

    def test_todo_only_in_comments(self):
        self.assertNotIn("D303", codes(check_text('//! d\nconst S: &str = "TODO: not a comment // FIXME";\n')))
        self.assertIn("D303", codes(check_text("//! d\nfn f() { /* TODO */ }\n")))
        self.assertIn("D303", codes(check_text("//! d\n/*\n  HACK: spans lines\n*/\n")))
        self.assertIn("D303", codes(check_text("//! d\n// XXX later\n")))

    def test_commented_out_code_needs_code_shape(self):
        self.assertIn("D304", codes(check_text("//! d\n// let x = 1;\n")))
        self.assertIn("D304", codes(check_text("//! d\n// foo.bar(1);\n")))
        self.assertIn("D304", codes(check_text("//! d\n// Kind::A => 1,\n")))
        self.assertIn("D304", codes(check_text("//! d\n// Registry::install(codec)\n")))
        for prose in ("// use the cached value", "// see Foo::bar() for details", "// if this fails we retry", "// Note: x; y",
                      "// azimuth = atan2(z, x). -60 degrees => z = -sin(60), x = cos(60).", "// Foo::bar(x) is the fast path"):
            self.assertNotIn("D304", codes(check_text(f"//! d\n{prose}\n")), prose)
        self.assertNotIn("D304", codes(check_text("//! d\n/// let x = 1;\npub struct RenderGraph;\n")))

    def test_get_prefix_and_noise_words(self):
        found = codes(check_text("//! d\npub struct MeshManager;\nimpl MeshManager {\n    pub fn get_mesh(&self) {}\n}\n", rel="c/src/mesh_manager.rs"))
        self.assertIn("N202", found)
        self.assertIn("N203", found)
        for name in ("get_mut", "get_or_insert_with", "get_unchecked_mut", "get_key_value", "get_disjoint_mut"):
            self.assertNotIn("N203", codes(check_text(f"//! d\nfn {name}(&mut self) {{}}\n")), name)
        for name in ("get_index_of", "get_or_default", "get_many"):
            self.assertIn("N203", codes(check_text(f"//! d\nfn {name}(&self) {{}}\n")), name)

    def test_noise_words_everywhere(self):
        self.assertIn("N202", codes(check_text("//! d\nmod a;\nstruct PassInfo;\n", rel="c/src/lib.rs", wiring=True)))
        self.assertIn("N202", codes(check_text("//! d\nfn f() {\n    enum StateData { A }\n}\n")))
        self.assertIn("N202", codes(check_text("//! d\n#[cfg(test)]\nmod tests {\n    struct FakeHelper;\n}\n", rel="c/tests/it.rs")))
        self.assertNotIn("N202", codes(check_text("//! d\nstruct Metadata;\n")))

    def test_primary_item_and_companions(self):
        text = "//! d\npub struct RenderGraph;\npub enum RenderGraphError {}\npub struct RenderGraphIter;\npub struct Pass;\npub struct RenderGraphStuff;\n"
        found = with_code(check_text(text), "C108")
        self.assertEqual(sorted(f.subject for f in found), ["Pass", "RenderGraphStuff"])
        self.assertEqual(with_code(check_text("//! d\npub struct Scene;\n", rel="c/src/scene/mod.rs"), "C108"), [])
        self.assertEqual(len(with_code(check_text("//! d\npub struct Other;\n", rel="c/src/scene/mod.rs"), "C108")), 1)

    def test_namespace_struct_and_globals(self):
        found = codes(check_text("//! d\npub struct GitUtil;\nimpl GitUtil {\n    pub fn run() {}\n}\nstatic STATE: Mutex<u8> = Mutex::new(0);\n", rel="c/src/git_util.rs"))
        self.assertIn("A502", found)
        self.assertIn("A501", found)

    def test_namespace_struct_with_braces_and_generic_impl(self):
        self.assertIn("A502", codes(check_text("//! d\npub struct Ns {}\nimpl Ns {\n    pub fn run() {}\n}\n", rel="c/src/ns.rs")))
        self.assertIn("A502", codes(check_text("//! d\npub struct Ns;\nimpl<T> Ns<T> {\n    pub fn run(t: T) {}\n}\n", rel="c/src/ns.rs")))
        self.assertNotIn("A502", codes(check_text("//! d\npub struct Ns {}\nimpl Ns {\n    pub fn run(&self) {}\n}\n", rel="c/src/ns.rs")))

    def test_global_state_forms(self):
        flagged = (
            "static mut N: u8 = 0;",
            "static N: AtomicUsize = AtomicUsize::new(0);",
            "pub static CACHE: OnceLock<Mutex<Vec<u8>>> = OnceLock::new();",
            "static T: LazyLock<RwLock<u8>> = LazyLock::new(|| RwLock::new(0));",
            "lazy_static! { static ref TABLE: Vec<u8> = vec![]; }",
            "thread_local! { static C: RefCell<u8> = RefCell::new(0); }",
        )
        for line in flagged:
            self.assertEqual(len(with_code(check_text(f"//! d\n{line}\n"), "A501")), 1, line)
        multi = "//! d\nthread_local! {\n    static C: RefCell<u8> = RefCell::new(0);\n}\n"
        self.assertEqual(len(with_code(check_text(multi), "A501")), 1)
        for line in ("static TABLE: OnceLock<Vec<u8>> = OnceLock::new();", "static NAMES: LazyLock<HashMap<u8, u8>> = LazyLock::new(HashMap::new);", "static N: u8 = 1;"):
            self.assertNotIn("A501", codes(check_text(f"//! d\n{line}\n")), line)

    def test_anyhow_allowed_only_in_interface(self):
        sig = "//! d\npub fn open() -> anyhow::Result<()> {\n    Ok(())\n}\n"
        self.assertIn("E404", codes(check_text(sig, layer="domain")))
        self.assertNotIn("E404", codes(check_text(sig, layer="interface")))

    def test_anyhow_imports_paths_and_multiline_signatures(self):
        self.assertIn("E404", codes(check_text("//! d\nuse anyhow::{Context, Result};\n", layer="infra")))
        self.assertIn("E404", codes(check_text("//! d\npub(crate) fn f() -> anyhow::Result<()> {\n    Ok(())\n}\n", layer="app")))
        multi = with_code(check_text("//! d\npub fn open(\n    path: &Path,\n) -> anyhow::Result<()> {\n    Ok(())\n}\n", layer="domain"), "E404")
        self.assertEqual([(f.line, f.subject) for f in multi], [(2, "open")])
        self.assertIn("E404", codes(check_text("//! d\nuse anyhow::Result;\n", rel="c/src/lib.rs", layer="")))
        self.assertNotIn("E404", codes(check_text("//! d\nuse anyhow::Result;\n", rel="c/src/main.rs", layer="")))
        self.assertNotIn("E404", codes(check_text("//! d\nuse anyhow::Result;\n", rel="c/src/bin/tool.rs", layer="")))
        self.assertNotIn("E404", codes(check_text('//! d\nconst S: &str = "anyhow::Error";\n#[cfg(test)]\nmod tests {\n    use anyhow::Result;\n}\n')))

    def test_stringly_typed_code(self):
        found = codes(check_text('//! d\nfn f(ns: &str, p: &str) -> String {\n    format!("{ns}:{p}")\n}\nfn g(k: &str) {\n    match k {\n        "open" => {}\n        _ => {}\n    }\n}\n'))
        self.assertIn("S701", found)
        self.assertIn("S702", found)
        parser = '//! d\nimpl FromStr for Kind {\n    fn from_str(s: &str) {\n        match s {\n            "open" => {}\n        }\n    }\n}\n'
        self.assertNotIn("S702", codes(check_text(parser, rel="c/src/kind.rs")))


class DirRulesTest(unittest.TestCase):
    def test_directory_rules(self):
        files = [f"c/src/audio/audio_{i}.rs" for i in range(13)] + ["c/src/audio/utils.rs"]
        found = codes(file_rules.check_dir("c/src/audio", files, config.Limits(), 1))
        self.assertEqual(found, ["C104", "C106", "N201"])

    def test_module_depth_message(self):
        found = file_rules.check_dir("c/src/a/b/c/d/e", ["c/src/a/b/c/d/e/x.rs"], config.Limits(), 5)
        self.assertIn("module depth 5", with_code(found, "C105")[0].message)

    def test_single_file_module_is_a_warning(self):
        found = with_code(file_rules.check_dir("c/src/scene", ["c/src/scene/mod.rs"], config.Limits(), 1), "C109")
        self.assertEqual([f.level for f in found], ["warning"])
        self.assertEqual(codes(file_rules.check_dir("c/src/scene", ["c/src/scene/mod.rs"], config.Limits(), 1, has_subdirs=True)), [])
        self.assertEqual(codes(file_rules.check_dir("c/src/scene", ["c/src/scene/mod.rs", "c/src/scene/tests.rs"], config.Limits(), 1)), [])


if __name__ == "__main__":
    unittest.main()
