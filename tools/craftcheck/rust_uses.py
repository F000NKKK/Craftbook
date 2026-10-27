"""`use` declarations: find them in stripped code and expand their brace
groups into plain paths (`use std::{fs, io::Read}` -> `std::fs`, `std::io::Read`)."""

import re
from dataclasses import dataclass

USE_START = re.compile(r"^\s*(pub(\([^)]*\))?\s+)?use\s+")


@dataclass
class UseDecl:
    first: int  # 0-based line where the declaration starts
    last: int  # 0-based line where it ends
    tree: str  # the use tree, without `use` and `;`


def use_declarations(code: list) -> list:
    """Every `use` declaration in stripped code, joined across lines."""
    out, i = [], 0
    while i < len(code):
        m = USE_START.match(code[i])
        if not m:
            i += 1
            continue
        text, j = code[i][m.end():], i
        while ";" not in text and j + 1 < len(code):
            j += 1
            text += " " + code[j]
        out.append(UseDecl(i, j, text.split(";", 1)[0]))
        i = j + 1
    return out


def expand(tree: str) -> list:
    """All paths a use tree names; `as` renames are dropped."""
    tree = re.sub(r"\s+as\s+\w+", "", tree)
    tree = re.sub(r"\s+", "", tree).lstrip(":")
    return _expand(tree)


def _expand(tree: str) -> list:
    brace = tree.find("{")
    if brace < 0:
        return [tree] if tree else []
    prefix = tree[:brace]
    body = tree[brace + 1: tree.rfind("}")]
    out = []
    for part in _split_top_level(body):
        if part == "self":
            out.append(prefix.rstrip(":"))
        elif part:
            out += [prefix + p for p in _expand(part)]
    return out


def _split_top_level(body: str) -> list:
    parts, depth, start = [], 0, 0
    for i, ch in enumerate(body):
        if ch == "{":
            depth += 1
        elif ch == "}":
            depth -= 1
        elif ch == "," and depth == 0:
            parts.append(body[start:i])
            start = i + 1
    parts.append(body[start:])
    return parts
