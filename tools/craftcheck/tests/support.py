"""Shared helpers for craftcheck's tests: import path and small generated inputs."""

import sys
import textwrap
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))

import config  # noqa: E402
import file_rules  # noqa: E402
import rust_source  # noqa: E402


def codes(findings):
    return sorted({f.code for f in findings})


def check_text(text, rel="crate/src/render_graph.rs", wiring=False, layer="domain"):
    src = rust_source.parse(textwrap.dedent(text))
    return file_rules.check_file(rel, src, wiring, config.Limits(), layer)


def with_code(findings, code):
    return [f for f in findings if f.code == code]


def write(root: Path, rel: str, text: str) -> None:
    path = root / rel
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(textwrap.dedent(text))
