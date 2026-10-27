"""Walks a repository's .rs files and applies the file and directory rules."""

import subprocess
from pathlib import Path

import file_rules
import rust_source
import workspace

SKIP_PARTS = {"target", "node_modules", ".git", "fuzz"}
RELAXED_BY_PROFILE = {"perf": {"C108", "A502"}}  # ch. 7 §7.8 performance kernels


def rust_files(repo: Path) -> list:
    tracked = subprocess.run(["git", "-C", str(repo), "ls-files", "-z", "*.rs"], capture_output=True, text=True)
    if tracked.returncode == 0 and tracked.stdout.strip("\0"):
        return sorted(p for p in tracked.stdout.split("\0") if p)
    return sorted(str(p.relative_to(repo)) for p in repo.rglob("*.rs") if not SKIP_PARTS & set(p.relative_to(repo).parts))


def is_wiring(path: Path) -> bool:
    """lib.rs/mod.rs/main.rs of a module that has submodules of its own.
    A crate's lib.rs and main.rs side by side are not submodules of each other."""
    if path.name not in ("lib.rs", "mod.rs", "main.rs"):
        return False
    roots = {"lib.rs", "main.rs"} if path.name in ("lib.rs", "main.rs") else set()
    for p in path.parent.iterdir():
        if p == path or p.name in ("tests.rs", "bin") or p.name in roots:
            continue
        if p.suffix == ".rs" or (p.is_dir() and any(p.rglob("*.rs"))):
            return True
    return False


def crate_of(rel: str, packages: list, repo: Path):
    best = None
    for pkg in packages:
        root = str(pkg.root.relative_to(repo))
        prefix = "" if root == "." else root + "/"
        if rel.startswith(prefix) and (best is None or len(prefix) > len(best[0])):
            best = (prefix, pkg)
    return best


def run_check(repo: Path, cfg) -> tuple:
    packages = workspace.discover(repo)
    files = [f for f in rust_files(repo) if not cfg.fully_exempt(f)]
    findings, dirs = [], {}
    for rel in files:
        path = repo / rel
        src = rust_source.parse(path.read_text(errors="replace"))
        if src.is_generated() or rel.endswith("_table.rs"):
            continue
        owner = crate_of(rel, packages, repo)
        crate = cfg.crates.get(owner[1].name) if owner else None
        relaxed = RELAXED_BY_PROFILE.get(crate.profile, set()) if crate else set()
        found = file_rules.check_file(rel, src, is_wiring(path), cfg.limits, crate.layer if crate else "")
        findings += [f for f in found if f.code not in relaxed]
        depth = module_depth(str(Path(rel).parent), owner[0] if owner else "")
        if depth is not None:
            dirs.setdefault(str(Path(rel).parent), (depth, []))[1].append(rel)
    findings += _dir_findings(dirs, cfg.limits)
    return files, [f for f in findings if not cfg.exempted(f.path, f.code)]


def module_depth(rel_dir: str, crate_prefix: str):
    """Number of directories below the crate's src/, or None outside src/.
    Without a known crate, the last `src` in the path is the crate's."""
    inner = rel_dir[len(crate_prefix):] if crate_prefix and rel_dir.startswith(crate_prefix) else rel_dir
    parts = Path(inner).parts
    if not crate_prefix and "src" in parts:
        parts = parts[len(parts) - 1 - parts[::-1].index("src"):]
    return len(parts) - 1 if parts and parts[0] == "src" else None


def _dir_findings(dirs: dict, limits) -> list:
    out = []
    for rel_dir, (depth, members) in dirs.items():
        has_subdirs = any(other.startswith(rel_dir + "/") for other in dirs)
        out += file_rules.check_dir(rel_dir, members, limits, depth, has_subdirs)
    return out
