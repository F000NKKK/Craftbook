"""Comment rules (Craftbook ch. 5): D303 TODO without an issue, D304
commented-out code, D305 banners. They read comment text only, never string
literals."""

import re

from report import Finding

TODO = re.compile(r"\b(TODO|FIXME|XXX|HACK)\b(?!\(#\d+\))")
BANNER = re.compile(r"^\s*//\s*([=\-*#/~]{4,})")
CODE_START = re.compile(
    r"^(let\s|(pub(\([^)]*\))?\s+)?(async\s+|unsafe\s+|const\s+)*fn\s+\w|use\s+[\w:{]|impl\b|"
    r"(if|while|match)\s|for\s.*\bin\b|else\b|loop\b|return\b|break\b|[{}]|#\[|"
    r"[\w.:&*\[\]]+\s*(\(|[+\-*/|&]?=\s|\.\w))"
)
MATCH_ARM = re.compile(r"^[\w:\"'|() ]+=>")
PATH_CALL = re.compile(r"^[\w:]+::\w+\s*\(.*\)[;,?]*$")


def comment_findings(rel: str, src) -> list:
    out = []
    for i, (line, code, comment) in enumerate(zip(src.lines, src.code, src.comments)):
        n = i + 1
        m = TODO.search(comment)
        if m:
            out.append(Finding(rel, n, "D303", "error", f"{m.group(1)} without an issue reference: write `{m.group(1)}(#123): …`"))
        if is_commented_code(line, code, comment):
            out.append(Finding(rel, n, "D304", "error", "commented-out code: delete it, version control keeps it"))
        if BANNER.match(line):
            out.append(Finding(rel, n, "D305", "warning", "banner comment: a file that needs sections is several files"))
    return out


def is_commented_code(line: str, code: str, comment: str) -> bool:
    """A whole-line `//` comment (not a doc comment) whose text both starts
    like code and has the shape of code."""
    stripped = line.lstrip()
    if code.strip() or not stripped.startswith("//") or stripped.startswith(("///", "//!")):
        return False
    return looks_like_code(comment.strip())


def looks_like_code(text: str) -> bool:
    """Code shape: ends with `;`, `{` or `}` and starts like a statement, or
    is a match arm (`=>`), or a path call (`::` with parentheses)."""
    if text.endswith((";", "{", "}")) and CODE_START.match(text):
        return True
    return bool(MATCH_ARM.match(text) or PATH_CALL.match(text))
