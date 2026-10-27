"""Reads a project's craftbook.toml: limits, the crate layer map, the context map."""

import tomllib
from dataclasses import dataclass, field
from pathlib import Path

LAYERS = ("kernel", "domain", "app", "infra", "interface", "mixed")
RELATIONS = ("shared-kernel", "customer-supplier", "acl", "published-language")


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


@dataclass
class Config:
    limits: Limits = field(default_factory=Limits)
    crates: dict = field(default_factory=dict)
    edges: set = field(default_factory=set)
    exempt: list = field(default_factory=list)


def load(repo: Path, path: Path = None) -> Config:
    path = path or repo / "craftbook.toml"
    if not path.is_file():
        return Config()
    raw = tomllib.loads(path.read_text())
    limits = Limits(**{k.replace("-", "_"): v for k, v in raw.get("limits", {}).items()})
    crates = {}
    for entry in raw.get("crate", []):
        crate = Crate(entry["name"], entry["layer"], entry.get("context", ""), entry.get("profile", ""))
        if crate.layer not in LAYERS:
            raise ValueError(f"craftbook.toml: crate {crate.name}: unknown layer {crate.layer!r}")
        crates[crate.name] = crate
    edges = set()
    for entry in raw.get("context_edge", []):
        if entry["relation"] not in RELATIONS:
            raise ValueError(f"craftbook.toml: edge {entry}: unknown relation")
        edges.add((entry["from"], entry["to"]))
    exempt = [(e["path"], e["reason"]) for e in raw.get("exempt", [])]
    return Config(limits, crates, edges, exempt)
