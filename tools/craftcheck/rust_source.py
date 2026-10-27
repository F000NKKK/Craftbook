"""Just enough Rust lexing for file-level rules: code without literals and
comments, the comment text of each line, brace depth per line, the test
module's span, top-level items."""

import re
from dataclasses import dataclass

RAW_START = re.compile(r'b?r(#*)"')
CHAR = re.compile(r"'(?:\\u\{[0-9a-fA-F]+\}|\\.|[^'\\])'")
TEST_ATTR = r"#\[cfg\((?:test|all\(test\b|any\(test\b)"
MOD_HEAD = r"(pub(\([^)]*\))?\s+)?mod\s+\w+\s*\{"
TEST_CFG = re.compile(r"^\s*" + TEST_ATTR)
TEST_MOD_ONE_LINE = re.compile(r"^\s*" + TEST_ATTR + r"[^\]]*\]\s*" + MOD_HEAD)
MOD_BLOCK = re.compile(r"^\s*" + MOD_HEAD)
ITEM = re.compile(
    r"^(?P<vis>pub(\([^)]*\))?\s+)?(?:unsafe\s+|const\s+|async\s+|extern\s+\"[^\"]*\"\s+)*"
    r"(?P<kind>fn|struct|enum|trait|impl|type|const|static|union|mod|macro_rules!)\b\s*(?P<name>\w*)"
)
GENERATED = re.compile(r"^//\s*@generated")
TYPE_KINDS = ("struct", "enum", "trait", "union")


@dataclass
class Item:
    line: int  # 1-based
    kind: str
    name: str
    public: bool


@dataclass
class Source:
    lines: list
    code: list  # each line with comments and literals blanked out
    comments: list  # the text of the comments on each line ("" if none)
    depth: list  # brace depth at the start of each line
    test_span: tuple  # (start, end) 0-based, end exclusive; (n, n) if none

    def production_lines(self) -> int:
        start, end = self.test_span
        return len(self.lines) - (end - start)

    def is_generated(self) -> bool:
        """Only a first line `// @generated …` marks a generated file."""
        return bool(self.lines) and bool(GENERATED.match(self.lines[0]))

    def in_tests(self, index: int) -> bool:
        start, end = self.test_span
        return start <= index < end


@dataclass
class _Lexer:
    """State carried from one line to the next: an open block comment or
    string literal."""

    block: int = 0  # nesting depth of /* */
    closer: str = ""  # what ends the string literal we are inside
    escapes: bool = False

    def line(self, line: str) -> tuple:
        code, comment, i = [], [], 0
        while i < len(line):
            i = self._step(line, i, code, comment)
        return "".join(code), "".join(comment)

    def _step(self, line: str, i: int, code: list, comment: list) -> int:
        if self.closer:
            return self._in_string(line, i, code)
        if self.block:
            return self._in_block(line, i, comment)
        if line.startswith("//", i):
            comment.append(line[i + 2:])
            return len(line)
        if line.startswith("/*", i):
            self.block = 1
            return i + 2
        return self._in_code(line, i, code)

    def _in_string(self, line: str, i: int, code: list) -> int:
        while i < len(line):
            if self.escapes and line[i] == "\\":
                i += 2
                continue
            if line.startswith(self.closer, i):
                end = i + len(self.closer)
                self.closer = ""
                code.append('""')
                return end
            i += 1
        return i

    def _in_block(self, line: str, i: int, comment: list) -> int:
        if line.startswith("*/", i):
            self.block -= 1
            return i + 2
        if line.startswith("/*", i):
            self.block += 1
            return i + 2
        comment.append(line[i])
        return i + 1

    def _in_code(self, line: str, i: int, code: list) -> int:
        after_ident = i > 0 and (line[i - 1].isalnum() or line[i - 1] == "_")
        raw = RAW_START.match(line, i)
        if raw and not after_ident:
            self.closer, self.escapes = '"' + raw.group(1), False
            return raw.end()
        if line[i] == '"':
            self.closer, self.escapes = '"', True
            return i + 1
        char = CHAR.match(line, i) if line[i] == "'" else None
        if char:
            code.append("''")
            return char.end()
        code.append(line[i])
        return i + 1


def lex(lines: list) -> tuple:
    """(code, comments): each line with comments and literals blanked out,
    and the comment text of each line."""
    lexer, code, comments = _Lexer(), [], []
    for line in lines:
        c, k = lexer.line(line)
        code.append(c)
        comments.append(k)
    return code, comments


def strip_code(lines: list) -> list:
    """Blank out comments and string/char literals, keeping line count."""
    return lex(lines)[0]


def parse(text: str) -> Source:
    lines = text.split("\n")
    code, comments = lex(lines)
    depth, d = [], 0
    for c in code:
        depth.append(d)
        d += c.count("{") - c.count("}")
    return Source(lines, code, comments, depth, _test_span(code, depth))


def _test_span(code: list, depth: list) -> tuple:
    n = len(code)
    for i, c in enumerate(code):
        if depth[i] != 0 or not TEST_CFG.match(c):
            continue
        k = i if TEST_MOD_ONE_LINE.match(c) else _skip_attributes(code, i + 1)
        if k == i or (k < n and MOD_BLOCK.match(code[k])):
            end = k + 1
            while end < n and depth[end] > 0:
                end += 1
            return (i, end)
    return (n, n)


def _skip_attributes(code: list, k: int) -> int:
    while k < len(code) and code[k].strip().startswith("#["):
        k += 1
    return k


def top_level_items(src: Source) -> list:
    """Items at brace depth 0, outside the test module."""
    items = []
    for i, c in enumerate(src.code):
        if src.depth[i] != 0 or src.in_tests(i):
            continue
        m = ITEM.match(c)
        if m:
            items.append(Item(i + 1, m.group("kind"), m.group("name"), bool(m.group("vis"))))
    return items


def type_items(src: Source) -> list:
    """Every type definition in the file, at any depth and in tests too;
    `type` aliases only at the top level (nested ones are associated types)."""
    items = []
    for i, c in enumerate(src.code):
        m = ITEM.match(c.lstrip())
        if not m or not m.group("name"):
            continue
        kind = m.group("kind")
        if kind in TYPE_KINDS or (kind == "type" and src.depth[i] == 0):
            items.append(Item(i + 1, kind, m.group("name"), bool(m.group("vis"))))
    return items


def module_doc_lines(src: Source) -> int:
    """Number of leading `//!` lines, after blank lines, `#![…]` attributes
    and plain `//` licence/SPDX lines."""
    count, seen = 0, False
    for line in src.lines:
        s = line.strip()
        if s.startswith("//!"):
            count, seen = count + 1, True
        elif s.startswith("#![") or (not seen and _is_preamble(s)):
            continue
        else:
            break
    return count


def _is_preamble(s: str) -> bool:
    return not s or (s.startswith("//") and not s.startswith("///"))


def camel(snake: str) -> str:
    return "".join(part[:1].upper() + part[1:] for part in snake.split("_"))
