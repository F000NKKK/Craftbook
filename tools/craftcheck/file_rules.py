"""Rules about one .rs file or one module directory: chapters 1, 3, 4 and 5
of Craftbook, where clippy cannot see (clippy covers function size,
parameters, nesting, unwrap, allow attributes and docs on items)."""

import re
from pathlib import Path

from report import Finding
from rust_source import Source, camel, module_doc_lines, top_level_items

FORBIDDEN_MODULES = {"util", "utils", "helper", "helpers", "common", "misc", "types", "traits", "structs", "managers", "services", "models"}
NOISE_WORDS = ("Manager", "Helper", "Util", "Utils", "Data", "Info", "Impl", "Wrapper", "Processor", "Object", "Thing", "Base")
COMPANION_SUFFIXES = ("Error", "ErrorKind", "Builder", "Iter", "IterMut", "IntoIter", "Guard", "Ref", "Mut", "Options", "Kind")
STD_GET = re.compile(r"\bfn\s+get(_mut|_or_\w+|_unchecked\w*|_many\w*|_disjoint\w*|_key_value|_index_of)?\b")
GET_PREFIX = re.compile(r"\bfn\s+(get_\w+)")
TODO = re.compile(r"\b(TODO|FIXME|XXX|HACK)\b(?!\(#\d+\))")
COMMENTED_CODE = re.compile(r"^\s*//\s*(let |fn |pub |use |impl |if .*\{|for .*\{|match |return |\}|[\w.:]+\(.*\);\s*$)")
BANNER = re.compile(r"^\s*//\s*([=\-*#/~]{4,})")
LET_UNDERSCORE = re.compile(r"^\s*let\s+_\s*=\s*.*\(.*")
GLOBAL_STATE = re.compile(r"^\s*(pub(\([^)]*\))?\s+)?static\s+(mut\s+\w+|\w+\s*:\s*[\w:<]*(Mutex|RwLock|RefCell|Cell|OnceLock|OnceCell)\b)")
EMPTY_STRUCT = re.compile(r"^\s*(pub(\([^)]*\))?\s+)?struct\s+(\w+)\s*;")
ANYHOW_SIG = re.compile(r"^\s*pub(\([^)]*\))?\s+(async\s+)?fn\b.*\banyhow::")
ID_FORMAT = re.compile(r'format!\(\s*"[^"]*\{[^}]*\}[:/][^"]*\{')
STRING_ARM = re.compile(r'^\s*"(?:\\.|[^"\\])*"\s*(\|\s*"(?:\\.|[^"\\])*"\s*)*(if\b.*)?=>')
PARSER = re.compile(r"\bimpl\b[^{]*\bFromStr\b|\bfn\s+(from_str|parse\w*)\b")
WIRING_KINDS = {"fn", "impl", "struct", "enum", "trait", "const", "static", "union", "type", "macro_rules!"}


def check_file(rel: str, src: Source, is_wiring: bool, limits, layer: str) -> list:
    out = []

    def add(line, code, level, msg):
        out.append(Finding(rel, line, code, level, msg))

    prod = src.production_lines()
    if prod > limits.file_lines:
        add(1, "C101", "error", f"{prod} production lines (limit {limits.file_lines}): split by responsibility")
    elif prod > limits.file_lines_target:
        add(1, "C101", "warning", f"{prod} production lines (target {limits.file_lines_target})")
    start, end = src.test_span
    if end - start > limits.test_module_lines:
        add(start + 1, "C102", "error", f"inline test module of {end - start} lines (limit {limits.test_module_lines}): move to tests.rs")

    items = top_level_items(src)
    if is_wiring:
        logic = [i for i in items if i.kind in WIRING_KINDS]
        for item in logic[:3]:
            add(item.line, "C103", "error", f"`{item.kind} {item.name}` in a wiring file (lib.rs/mod.rs of a split module): move it to its own file")
        if prod > limits.wiring_lines:
            add(1, "C103", "error", f"wiring file has {prod} lines (limit {limits.wiring_lines})")
    else:
        out += _primary_item_rules(rel, items)

    doc = module_doc_lines(src)
    if doc == 0:
        add(1, "D301", "error", "file has no `//!` module doc")
    elif doc > limits.module_doc_lines:
        add(1, "D302", "warning", f"`//!` doc of {doc} lines (limit {limits.module_doc_lines}): move rationale to docs/ or an ADR")

    out += _line_rules(rel, src, layer)
    return out


def _primary_item_rules(rel: str, items: list) -> list:
    stem = Path(rel).stem
    if stem in ("main", "lib", "build", "tests") or "/tests/" in rel or "/benches/" in rel or "/examples/" in rel:
        return []
    primary = camel(stem)
    types = [i for i in items if i.public and i.kind in ("struct", "enum", "trait", "union") and i.name]
    others = [t for t in types if t.name != primary and not t.name.startswith(primary) and not t.name.endswith(COMPANION_SUFFIXES)]
    out = []
    if len(others) >= 2 or (others and primary not in {t.name for t in types}):
        names = ", ".join(t.name for t in others[:4])
        out.append(Finding(rel, others[0].line, "C108", "warning", f"public types besides the file's primary `{primary}`: {names} — one primary item per file"))
    for t in types:
        for word in NOISE_WORDS:
            if re.search(rf"(^|[a-z]){word}($|[A-Z])", t.name):
                out.append(Finding(rel, t.line, "N202", "error", f"type `{t.name}` uses the noise word `{word}`"))
                break
    return out


def _line_rules(rel: str, src: Source, layer: str) -> list:
    out = []
    t_start, t_end = src.test_span
    empty_structs = {}
    is_parser = "parse" in Path(rel).stem or any(PARSER.search(c) for c in src.code)
    for i, (line, code) in enumerate(zip(src.lines, src.code)):
        n = i + 1
        in_tests = t_start <= i < t_end
        if TODO.search(line) and "//" in line:
            out.append(Finding(rel, n, "D303", "error", "TODO/FIXME without an issue reference: write `TODO(#123): …`"))
        if COMMENTED_CODE.match(line) and not line.lstrip().startswith(("///", "//!")):
            out.append(Finding(rel, n, "D304", "error", "commented-out code"))
        if BANNER.match(line):
            out.append(Finding(rel, n, "D305", "warning", "banner comment: a file that needs sections is several files"))
        if in_tests:
            continue
        m = GET_PREFIX.search(code)
        if m and not STD_GET.search(code):
            out.append(Finding(rel, n, "N203", "error", f"`{m.group(1)}`: getters have no `get_` prefix"))
        if LET_UNDERSCORE.match(code):
            out.append(Finding(rel, n, "E403", "warning", "`let _ = call(…)` may swallow an error: handle, propagate or comment why"))
        if GLOBAL_STATE.match(code):
            out.append(Finding(rel, n, "A501", "error", "global mutable state: pass an owned value instead"))
        if ANYHOW_SIG.match(code) and layer not in ("interface", ""):
            out.append(Finding(rel, n, "E404", "error", "`anyhow` in a public library signature: return a typed error"))
        if ID_FORMAT.search(line):
            out.append(Finding(rel, n, "S701", "warning", "id or path built with `format!`: compose it with its type"))
        if STRING_ARM.match(line) and not is_parser:
            out.append(Finding(rel, n, "S702", "warning", "`match` on string literals outside a parser: parse into an enum once"))
        e = EMPTY_STRUCT.match(code)
        if e and src.depth[i] == 0:
            empty_structs[e.group(3)] = n
    out += _manager_structs(rel, src, empty_structs)
    return out


def _manager_structs(rel: str, src: Source, empty: dict) -> list:
    """A field-less struct whose inherent impl never takes `self` and that
    implements no trait is a namespace pretending to be a type (ch.2 §2.3)."""
    out = []
    text = "\n".join(src.code)
    for name, line in empty.items():
        if re.search(rf"\bimpl\b[^{{]*\bfor\s+{name}\b", text):
            continue
        body = re.search(rf"\bimpl\s+{name}\s*\{{(.*?)\n\}}", text, re.S)
        if body and "fn " in body.group(1) and "self" not in body.group(1):
            out.append(Finding(rel, line, "A502", "error", f"`struct {name};` only groups functions: use a module and free functions"))
    return out


def check_dir(rel_dir: str, rs_files: list, limits, depth: int) -> list:
    out = []
    name = Path(rel_dir).name
    counted = [f for f in rs_files if Path(f).name not in ("mod.rs", "tests.rs", "lib.rs", "main.rs")]
    if len(counted) > limits.files_per_dir:
        out.append(Finding(rel_dir, 0, "C104", "error", f"{len(counted)} files in one module directory (limit {limits.files_per_dir}): group into submodules"))
    if depth > limits.module_depth:
        out.append(Finding(rel_dir, 0, "C105", "error", f"module depth {depth} below src/ (limit {limits.module_depth})"))
    for f in rs_files:
        stem = Path(f).stem
        if stem in FORBIDDEN_MODULES:
            out.append(Finding(f, 1, "C106", "error", f"module named `{stem}`: split by responsibility, not by kind"))
        if depth > 0 and stem.startswith(name + "_"):
            out.append(Finding(f, 1, "N201", "error", f"`{name}::{stem}` stutters: name it `{name}::{stem[len(name) + 1:]}`"))
    if depth > 0 and name in FORBIDDEN_MODULES:
        out.append(Finding(rel_dir, 0, "C106", "error", f"module named `{name}`: split by responsibility, not by kind"))
    return out
