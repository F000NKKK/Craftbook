"""L605: I/O and async in the source of a shared-kernel or domain crate
(Craftbook ch. 7 §7.3). Reads code only, never comments or strings, and
skips the test module."""

import re
from pathlib import Path

import rust_source
from report import Finding
from rust_uses import expand, use_declarations

IO_PATHS = ("std::fs", "std::process", "std::net", "std::env", "std::thread::sleep", "tokio")
IO_SOURCE = re.compile(r"\bstd::(fs|process|net|env)\b|\bstd::thread::sleep\b|\btokio::|\basync\s+fn\b")


def scan_package(repo: Path, pkg, layer: str, config) -> list:
    out = []
    for path in sorted((pkg.root / "src").rglob("*.rs")):
        rel = str(path.relative_to(repo))
        if path.name == "tests.rs" or config.exempted(rel, "L605"):
            continue
        out += io_findings(rel, rust_source.parse(path.read_text(errors="replace")), layer)
    return out


def io_findings(rel: str, src, layer: str) -> list:
    out, in_use = [], set()
    for decl in use_declarations(src.code):
        in_use.update(range(decl.first, decl.last + 1))
        if src.in_tests(decl.first):
            continue
        for hit in sorted({h for h in map(_io_path, expand(decl.tree)) if h}):
            out.append(_finding(rel, decl.first, hit, layer))
    for i, code in enumerate(src.code):
        if i in in_use or src.in_tests(i):
            continue
        m = IO_SOURCE.search(code)
        if m:
            out.append(_finding(rel, i, _normalise(m.group(0)), layer))
    return out


def _io_path(path: str) -> str:
    for io in IO_PATHS:
        if path == io or path.startswith(io + "::"):
            return io
    return ""


def _normalise(match: str) -> str:
    return "async fn" if match.startswith("async") else match.rstrip(":")


def _finding(rel: str, index: int, what: str, layer: str) -> Finding:
    return Finding(rel, index + 1, "L605", "error", f"`{what}` in a {layer} crate: I/O and async belong to adapters", what)
