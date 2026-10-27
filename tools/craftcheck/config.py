"""Reads a project's craftbook.toml: limits, the crate layer map, the context
map and the exemptions."""

import dataclasses
import fnmatch
import re
import tomllib
from dataclasses import dataclass, field
from pathlib import Path

LAYERS = ("shared-kernel", "domain", "app", "infra", "interface", "mixed")
RELATIONS = ("shared-kernel", "customer-supplier", "acl", "published-language")
PROFILES = ("", "perf")
RENAMED = {"kernel": "shared-kernel"}
CODE = re.compile(r"^[A-Z]\d{3}$")


@dataclass
class Limits:
    file_lines: int = 400
    file_lines_target: int = 250
    test_module_lines: int = 150
    wiring_lines: int = 100
    files_per_dir: int = 12
    module_depth: int = 4
    module_doc_lines: int = 30


@dataclass
class Crate:
    name: str
    layer: str
    context: str
    profile: str = ""


@dataclass(frozen=True)
class Exempt:
    path: str  # fnmatch pattern
    reason: str
    codes: tuple = ()  # empty: every code

    def covers(self, rel: str, code: str) -> bool:
        return fnmatch.fnmatch(rel, self.path) and (not self.codes or code in self.codes)


@dataclass
class Config:
    limits: Limits = field(default_factory=Limits)
    crates: dict = field(default_factory=dict)
    edges: dict = field(default_factory=dict)  # (from, to) -> relation
    exempt: list = field(default_factory=list)

    def exempted(self, rel: str, code: str) -> bool:
        return any(e.covers(rel, code) for e in self.exempt)

    def fully_exempt(self, rel: str) -> bool:
        return any(not e.codes and fnmatch.fnmatch(rel, e.path) for e in self.exempt)


def load(repo: Path, path: Path = None) -> Config:
    path = path or repo / "craftbook.toml"
    if not path.is_file():
        return Config()
    raw = tomllib.loads(path.read_text())
    crates = {c.name: c for c in map(_crate, raw.get("crate", []))}
    edges = dict(map(_edge, raw.get("context_edge", [])))
    exempt = [_exempt(e) for e in raw.get("exempt", [])]
    return Config(_limits(raw.get("limits", {})), crates, edges, exempt)


def _limits(raw: dict) -> Limits:
    defaults = {f.name: f.default for f in dataclasses.fields(Limits)}
    values = {}
    for key, value in raw.items():
        name = key.replace("-", "_")
        if name not in defaults:
            raise ValueError(f"craftbook.toml [limits]: unknown limit {key!r}")
        if not isinstance(value, int) or isinstance(value, bool) or value < 1:
            raise ValueError(f"craftbook.toml [limits]: {key} must be a positive integer, got {value!r}")
        if value > defaults[name]:
            raise ValueError(f"craftbook.toml [limits]: {key} = {value} is above Craftbook's {defaults[name]}; limits may only be lowered (raising one needs a Craftbook PR)")
        values[name] = value
    return Limits(**values)


def _crate(entry: dict) -> Crate:
    crate = Crate(entry["name"], entry["layer"], entry.get("context", ""), entry.get("profile", ""))
    if crate.layer not in LAYERS:
        hint = f" (renamed to {RENAMED[crate.layer]!r})" if crate.layer in RENAMED else ""
        raise ValueError(f"craftbook.toml: crate {crate.name}: unknown layer {crate.layer!r}{hint}; expected one of {', '.join(LAYERS)}")
    if crate.profile not in PROFILES:
        hint = " (renamed to 'perf')" if crate.profile == "kernel" else ""
        raise ValueError(f"craftbook.toml: crate {crate.name}: unknown profile {crate.profile!r}{hint}; expected 'perf' or none")
    return crate


def _edge(entry: dict) -> tuple:
    if entry.get("relation") not in RELATIONS:
        raise ValueError(f"craftbook.toml: edge {entry}: unknown relation; expected one of {', '.join(RELATIONS)}")
    return (entry["from"], entry["to"]), entry["relation"]


def _exempt(entry: dict) -> Exempt:
    reason = str(entry.get("reason", "")).strip()
    if not reason:
        raise ValueError(f"craftbook.toml [[exempt]] {entry.get('path')!r}: `reason` is required and must not be empty")
    codes = entry.get("codes", [])
    if not isinstance(codes, list) or not all(isinstance(c, str) and CODE.match(c) for c in codes):
        raise ValueError(f"craftbook.toml [[exempt]] {entry['path']!r}: `codes` must be a list of rule codes like \"A501\"")
    return Exempt(entry["path"], reason, tuple(codes))
