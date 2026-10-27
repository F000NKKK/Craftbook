"""Line rules outside the test module: N203 getters, E403 swallowed results,
S701/S702 stringly-typed code (Craftbook ch. 3, 4)."""

import re
from pathlib import Path

from report import Finding

STD_GETTERS = {"get_mut", "get_or_insert", "get_or_insert_with", "get_unchecked", "get_unchecked_mut", "get_many_mut", "get_disjoint_mut", "get_key_value"}
GET_PREFIX = re.compile(r"\bfn\s+(get_\w+)")
LET_UNDERSCORE = re.compile(r"^\s*let\s+_\s*=\s*.*\(.*")
ID_FORMAT = re.compile(r'format!\(\s*"[^"]*\{[^}]*\}[:/][^"]*\{')
STRING_ARM = re.compile(r'^\s*"(?:\\.|[^"\\])*"\s*(\|\s*"(?:\\.|[^"\\])*"\s*)*(if\b.*)?=>')
PARSER = re.compile(r"\bimpl\b[^{]*\bFromStr\b|\bfn\s+(from_str|parse\w*)\b")


def line_findings(rel: str, src) -> list:
    out = []
    is_parser = "parse" in Path(rel).stem or any(PARSER.search(c) for c in src.code)
    for i, (line, code) in enumerate(zip(src.lines, src.code)):
        if src.in_tests(i):
            continue
        n = i + 1
        m = GET_PREFIX.search(code)
        if m and m.group(1) not in STD_GETTERS:
            out.append(Finding(rel, n, "N203", "error", f"`{m.group(1)}`: getters have no `get_` prefix", m.group(1)))
        if LET_UNDERSCORE.match(code):
            out.append(Finding(rel, n, "E403", "warning", "`let _ = call(…)` may swallow an error: handle, propagate or comment why"))
        if ID_FORMAT.search(line):
            out.append(Finding(rel, n, "S701", "warning", "id or path built with `format!`: compose it with its type"))
        if STRING_ARM.match(line) and not is_parser:
            out.append(Finding(rel, n, "S702", "warning", "`match` on string literals outside a parser: parse into an enum once"))
    return out
