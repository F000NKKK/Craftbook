"""Finds a repository's packages: the members of its Cargo workspace, or every
Cargo.toml when there is no workspace."""

import fnmatch
import tomllib
from dataclasses import dataclass
from pathlib import Path

SKIP_PARTS = {"target", "node_modules", ".git"}


@dataclass
class Package:
    name: str
    root: Path
    deps: set
    has_workspace_lints: bool


def root_manifest(repo: Path) -> dict:
    path = repo / "Cargo.toml"
    return tomllib.loads(path.read_text()) if path.is_file() else {}


def discover(repo: Path) -> list:
    root = root_manifest(repo)
    workspace = root.get("workspace")
    manifests = _member_manifests(repo, root, workspace) if workspace is not None else _all_manifests(repo)
    return [pkg for pkg in map(_package, manifests) if pkg]


def _all_manifests(repo: Path) -> list:
    return sorted(m for m in repo.rglob("Cargo.toml") if not SKIP_PARTS & set(m.relative_to(repo).parts))


def _member_manifests(repo: Path, root: dict, workspace: dict) -> list:
    exclude = [_norm(e) for e in workspace.get("exclude", [])]
    found = [repo / "Cargo.toml"] if "package" in root else []
    for pattern in workspace.get("members", []):
        for member in sorted(repo.glob(_norm(pattern))):
            rel = _norm(str(member.relative_to(repo)))
            manifest = member / "Cargo.toml"
            if manifest.is_file() and not any(fnmatch.fnmatch(rel, e) for e in exclude) and manifest not in found:
                found.append(manifest)
    return found


def _norm(path: str) -> str:
    path = path.strip().rstrip("/")
    return path[2:] if path.startswith("./") else path


def _package(manifest: Path):
    raw = tomllib.loads(manifest.read_text())
    if "package" not in raw:
        return None
    deps = set()
    tables = [raw.get("dependencies", {})]
    tables += [t.get("dependencies", {}) for t in raw.get("target", {}).values()]
    for table in tables:
        for key, spec in table.items():
            deps.add(spec.get("package", key) if isinstance(spec, dict) else key)
    lints = raw.get("lints", {}).get("workspace") is True
    return Package(raw["package"]["name"], manifest.parent, deps, lints)
