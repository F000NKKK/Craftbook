"""Walks a repository's .rs files and applies the file and directory rules."""

import fnmatch
import subprocess
from pathlib import Path

import deps_rules
import file_rules
import rust_source

SKIP_PARTS = {"target", "node_modules", ".git", "fuzz"}


def rust_files(repo: Path) -> list:
    tracked = subprocess.run(["git", "-C", str(repo), "ls-files", "*.rs"], capture_output=True, text=True)
    if tracked.returncode == 0 and tracked.stdout.strip():
        return sorted(tracked.stdout.split())
    return sorted(str(p.relative_to(repo)) for p in repo.rglob("*.rs") if not SKIP_PARTS & set(p.parts))


def is_wiring(path: Path) -> bool:
    """lib.rs/mod.rs/main.rs of a module that has submodules of its own."""
    if path.name not in ("lib.rs", "mod.rs", "main.rs"):
        return False
    siblings = [p for p in path.parent.iterdir() if p != path and (p.suffix == ".rs" or p.is_dir())]
    return any(p.name not in ("tests.rs", "bin") for p in siblings)


def crate_of(rel: str, packages: list, repo: Path):
    best = None
    for pkg in packages:
        root = str(pkg.root.relative_to(repo))
        prefix = "" if root == "." else root + "/"
        if rel.startswith(prefix) and (best is None or len(prefix) > len(best[0])):
            best = (prefix, pkg)
    return best


def run_check(repo: Path, cfg) -> tuple:
    packages = deps_rules.discover(repo)
    files = [f for f in rust_files(repo) if not any(fnmatch.fnmatch(f, pat) for pat, _ in cfg.exempt)]
    findings, dirs = [], {}
    for rel in files:
        path = repo / rel
        src = rust_source.parse(path.read_text(errors="replace"))
        if src.is_generated() or rel.endswith("_table.rs"):
            continue
        owner = crate_of(rel, packages, repo)
        layer = ""
        if owner:
            crate = cfg.crates.get(owner[1].name)
            layer = crate.layer if crate else ""
        findings += file_rules.check_file(rel, src, is_wiring(path), cfg.limits, layer)
        parts = Path(rel).parts
        if "src" in parts:
            dirs.setdefault(str(Path(rel).parent), []).append(rel)
    for rel_dir, members in dirs.items():
        parts = Path(rel_dir).parts
        depth = len(parts) - parts.index("src") - 1
        findings += file_rules.check_dir(rel_dir, members, cfg.limits, depth)
    return files, findings
