"""Rules that read an item across lines: A501 global mutable state, A502
namespace structs, E404 `anyhow` in library code (Craftbook ch. 2, 4)."""

import re

from report import Finding

STATIC = re.compile(r"^\s*(pub(\([^)]*\))?\s+)?static\s+(?P<mut>mut\s+)?(?P<name>\w+)\s*:")
STATIC_REF = re.compile(r"\bstatic\s+ref\s+(\w+)")
THREAD_LOCAL = re.compile(r"\bthread_local!")
MUTABLE = re.compile(r"\b(Mutex|RwLock|RefCell|Cell|UnsafeCell|Atomic\w*)\b")
EMPTY_STRUCT = re.compile(r"^\s*(pub(\([^)]*\))?\s+)?struct\s+(\w+)\s*(;|\{\s*\})")
ANYHOW = re.compile(r"\banyhow::|\buse\s+anyhow\b")
PUB_FN = re.compile(r"^\s*pub\s+((const|async|unsafe)\s+|extern\s+\"\"\s+)*fn\s+(\w+)")
BINARY_OR_TEST = re.compile(r"(^|/)(src/bin|tests|examples|benches)/|(^|/)(main|build|tests)\.rs$")


def global_state_findings(rel: str, src) -> list:
    out, macro_depth = [], None
    for i, code in enumerate(src.code):
        if macro_depth is not None and src.depth[i] > macro_depth:
            continue  # inside thread_local!: the macro itself was reported
        macro_depth = None
        if src.in_tests(i):
            continue
        what, name = _global(src.code, i)
        if what:
            out.append(Finding(rel, i + 1, "A501", "error", f"global mutable state ({what}): pass an owned value instead", name))
        if THREAD_LOCAL.search(code) and code.count("{") > code.count("}"):
            macro_depth = src.depth[i]
    return out


def _global(code: list, i: int) -> tuple:
    """(description, name) of the global declared on line i, or ("", "")."""
    line = code[i]
    if THREAD_LOCAL.search(line):
        return "`thread_local!`", "thread_local"
    ref = STATIC_REF.search(line)
    if ref:
        return f"`{ref.group(1)}` in `lazy_static!`; a constant is a `LazyLock`", ref.group(1)
    m = STATIC.match(line)
    if not m:
        return "", ""
    name = m.group("name")
    if m.group("mut"):
        return f"`static mut {name}`", name
    kind = MUTABLE.search(_static_type(code, i, m.end()))
    return (f"`static {name}` holds a `{kind.group(1)}`", name) if kind else ("", "")


def _static_type(code: list, i: int, start: int) -> str:
    """The declared type of a `static`, up to its `=` or `;`, across lines."""
    text = code[i][start:]
    for j in range(i + 1, min(i + 6, len(code))):
        if "=" in text or ";" in text:
            break
        text += " " + code[j]
    return re.split(r"=|;", text, maxsplit=1)[0]


def namespace_struct_findings(rel: str, src) -> list:
    """A field-less struct whose inherent impls never take `self` and that
    implements no trait is a namespace pretending to be a type (ch.2 §2.3)."""
    out = []
    text = "\n".join(src.code)
    for i, code in enumerate(src.code):
        m = EMPTY_STRUCT.match(code)
        if not m or src.depth[i] != 0:
            continue
        name = m.group(3)
        if re.search(rf"\bimpl\b[^{{;]*\bfor\s+{name}\b", text):
            continue
        bodies = [b.group(1) for b in re.finditer(rf"\bimpl\s*(?:<[^{{]*?>)?\s*{name}\s*(?:<[^{{]*?>)?\s*(?:where\b[^{{]*)?\{{(.*?)\n\}}", text, re.S)]
        if any("fn " in b for b in bodies) and not any(re.search(r"\bself\b", b) for b in bodies):
            out.append(Finding(rel, i + 1, "A502", "error", f"`struct {name}` only groups functions: use a module and free functions", name))
    return out


def anyhow_findings(rel: str, src, layer: str) -> list:
    """`anyhow` is for binaries and the interface layer; library code returns
    typed errors. An unmapped crate counts as a library."""
    if layer == "interface" or BINARY_OR_TEST.search(rel):
        return []
    out, seen = [], set()
    for i, code in enumerate(src.code):
        m = PUB_FN.match(code)
        if not m or src.in_tests(i):
            continue
        last = _signature_end(src.code, i)
        if ANYHOW.search(" ".join(src.code[i:last + 1])):
            seen.update(range(i, last + 1))
            out.append(Finding(rel, i + 1, "E404", "error", "`anyhow` in a public signature: return a typed error", m.group(3)))
    for i, code in enumerate(src.code):
        if i not in seen and not src.in_tests(i) and ANYHOW.search(code):
            out.append(Finding(rel, i + 1, "E404", "error", "`anyhow` in library code: define a typed error (anyhow belongs to binaries and the interface layer)"))
    return out


def _signature_end(code: list, i: int) -> int:
    j = i
    while j < len(code) - 1 and "{" not in code[j] and ";" not in code[j]:
        j += 1
    return j
