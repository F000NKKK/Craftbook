"""Rules about one .rs file or one module directory: chapters 1, 3, 4 and 5
of Craftbook, where clippy cannot see (clippy covers function size,
parameters, nesting, unwrap, allow attributes and docs on items)."""

import re
from pathlib import Path

import comment_rules
import item_rules
import line_rules
from report import Finding
from rust_source import Source, camel, module_doc_lines, top_level_items, type_items

FORBIDDEN_MODULES = {"util", "utils", "helper", "helpers", "common", "misc", "types", "traits", "structs", "managers", "services", "models"}
NOISE_WORDS = ("Manager", "Helper", "Util", "Utils", "Data", "Info", "Impl", "Wrapper", "Processor", "Object", "Thing", "Base")
COMPANION_SUFFIXES = ("Error", "ErrorKind", "Builder", "Iter", "IterMut", "IntoIter", "Guard", "Ref", "Mut", "Options", "Kind")
WIRING_LOGIC = {"fn", "impl", "struct", "enum", "trait", "const", "static", "union"}


def check_file(rel: str, src: Source, is_wiring: bool, limits, layer: str) -> list:
    out = _size_rules(rel, src, limits)
    items = top_level_items(src)
    out += _wiring_rules(rel, src, items, limits) if is_wiring else _primary_item_rules(rel, items)
    out += _noise_words(rel, src)
    out += _doc_rules(rel, src, limits)
    out += comment_rules.comment_findings(rel, src)
    out += line_rules.line_findings(rel, src)
    out += item_rules.global_state_findings(rel, src)
    out += item_rules.namespace_struct_findings(rel, src)
    out += item_rules.anyhow_findings(rel, src, layer)
    return out


def _size_rules(rel: str, src: Source, limits) -> list:
    out = []
    prod = src.production_lines()
    if prod > limits.file_lines:
        out.append(Finding(rel, 1, "C101", "error", f"{prod} production lines (limit {limits.file_lines}): split by responsibility"))
    elif prod > limits.file_lines_target:
        out.append(Finding(rel, 1, "C101", "warning", f"{prod} production lines (target {limits.file_lines_target})"))
    start, end = src.test_span
    if end - start > limits.test_module_lines:
        out.append(Finding(rel, start + 1, "C102", "error", f"inline test module of {end - start} lines (limit {limits.test_module_lines}): move to tests.rs"))
    return out


def _wiring_rules(rel: str, src: Source, items: list, limits) -> list:
    """Wiring files hold `mod`, `use`, `pub type` aliases and `macro_rules!`."""
    out = []
    for item in items:
        if item.kind in WIRING_LOGIC or (item.kind == "type" and not item.public):
            what = f"{item.kind} {item.name}".strip()
            out.append(Finding(rel, item.line, "C103", "error", f"`{what}` in a wiring file (lib.rs/mod.rs of a split module): move it to its own file", what))
    prod = src.production_lines()
    if prod > limits.wiring_lines:
        out.append(Finding(rel, 1, "C103", "error", f"wiring file has {prod} lines (limit {limits.wiring_lines})"))
    return out


def _primary_item_rules(rel: str, items: list) -> list:
    path = Path(rel)
    if path.stem in ("main", "lib", "build", "tests") or "/tests/" in rel or "/benches/" in rel or "/examples/" in rel:
        return []
    primary = camel(path.parent.name if path.name == "mod.rs" else path.stem)
    primaries = {primary}
    if path.name != "mod.rs" and path.parent.name != "src":
        # `parse/error.rs` may hold `ParseError`: the module path names it (ch. 1 §1.2).
        primaries.add(camel(path.parent.name) + primary)
    out = []
    for t in items:
        if t.public and t.kind in ("struct", "enum", "trait", "union") and t.name and not any(is_companion(t.name, p) for p in primaries):
            out.append(Finding(rel, t.line, "C108", "warning", f"public type `{t.name}` is neither the file's primary `{primary}` nor a companion (`{primary}Error`, `{primary}Builder`, …): give it its own file", t.name))
    return out


def is_companion(name: str, primary: str) -> bool:
    return name == primary or (name.startswith(primary) and name[len(primary):] in COMPANION_SUFFIXES)


def _noise_words(rel: str, src: Source) -> list:
    out = []
    for t in type_items(src):
        word = next((w for w in NOISE_WORDS if re.search(rf"(^|[a-z]){w}($|[A-Z])", t.name)), None)
        if word:
            out.append(Finding(rel, t.line, "N202", "error", f"type `{t.name}` uses the noise word `{word}`", t.name))
    return out


def _doc_rules(rel: str, src: Source, limits) -> list:
    doc = module_doc_lines(src)
    if doc == 0:
        return [Finding(rel, 1, "D301", "error", "file has no `//!` module doc")]
    if doc > limits.module_doc_lines:
        return [Finding(rel, 1, "D302", "warning", f"`//!` doc of {doc} lines (limit {limits.module_doc_lines}): move rationale to docs/ or an ADR")]
    return []


def check_dir(rel_dir: str, rs_files: list, limits, depth: int, has_subdirs: bool = False) -> list:
    """`depth` is the module depth: the number of directories below src/."""
    out = []
    name = Path(rel_dir).name
    counted = [f for f in rs_files if Path(f).name not in ("mod.rs", "tests.rs", "lib.rs", "main.rs")]
    if len(counted) > limits.files_per_dir:
        out.append(Finding(rel_dir, 0, "C104", "error", f"{len(counted)} files in one module directory (limit {limits.files_per_dir}): group into submodules"))
    if depth > limits.module_depth:
        out.append(Finding(rel_dir, 0, "C105", "error", f"module depth {depth} (directories below src/; limit {limits.module_depth})"))
    if depth > 0 and not has_subdirs and [Path(f).name for f in rs_files] == ["mod.rs"]:
        out.append(Finding(rel_dir, 0, "C109", "warning", "module directory holds only mod.rs: a single-file module is a file, fold it back"))
    for f in rs_files:
        stem = Path(f).stem
        if stem in FORBIDDEN_MODULES:
            out.append(Finding(f, 1, "C106", "error", f"module named `{stem}`: split by responsibility, not by kind", stem))
        if depth > 0 and stem.startswith(name + "_"):
            out.append(Finding(f, 1, "N201", "error", f"`{name}::{stem}` stutters: name it `{name}::{stem[len(name) + 1:]}`", stem))
    if depth > 0 and name in FORBIDDEN_MODULES:
        out.append(Finding(rel_dir, 0, "C106", "error", f"module named `{name}`: split by responsibility, not by kind", name))
    return out
