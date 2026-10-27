"""Rules about the crate graph: layers and the context map (Craftbook ch. 7)."""

import fnmatch
import re
import tomllib
from dataclasses import dataclass
from pathlib import Path

from report import Finding

ALLOWED = {
    "kernel": {"kernel"},
    "domain": {"kernel", "domain"},
    "app": {"kernel", "domain", "app"},
    "infra": {"kernel", "domain", "app", "infra"},
    "interface": {"kernel", "domain", "app", "infra", "interface", "mixed"},
    "mixed": {"kernel", "domain", "app", "infra", "interface", "mixed"},
}
IO_CRATES = {"tokio", "async-std", "wgpu", "cpal", "winit", "reqwest", "hyper", "portable-pty", "notify", "git2", "tauri", "libc", "nix", "windows", "windows-sys", "rodio", "ash", "vulkano"}
IO_SOURCE = re.compile(r"\bstd::(fs|process|net|env)\b|\btokio::|\basync\s+fn\b|\bstd::thread::sleep\b")


@dataclass
class Package:
    name: str
    root: Path
    deps: set
    has_workspace_lints: bool


def discover(repo: Path) -> list:
    out = []
    for manifest in repo.rglob("Cargo.toml"):
        if any(part in ("target", "node_modules", ".git") for part in manifest.parts):
            continue
        raw = tomllib.loads(manifest.read_text())
        if "package" not in raw:
            continue
        deps = set()
        tables = [raw.get("dependencies", {})]
        tables += [t.get("dependencies", {}) for t in raw.get("target", {}).values()]
        for table in tables:
            for key, spec in table.items():
                deps.add(spec.get("package", key) if isinstance(spec, dict) else key)
        lints = raw.get("lints", {}).get("workspace") is True
        out.append(Package(raw["package"]["name"], manifest.parent, deps, lints))
    return out


def workspace_has_lints(repo: Path) -> bool:
    root = repo / "Cargo.toml"
    return root.is_file() and "lints" in tomllib.loads(root.read_text()).get("workspace", {})


def check_graph(repo: Path, packages: list, config) -> list:
    out = []
    names = {p.name for p in packages}
    lints_expected = workspace_has_lints(repo)
    for pkg in packages:
        manifest = str((pkg.root / "Cargo.toml").relative_to(repo))
        if lints_expected and not pkg.has_workspace_lints:
            out.append(Finding(manifest, 1, "L606", "warning", "crate does not inherit the workspace lint set (`[lints] workspace = true`)"))
        crate = config.crates.get(pkg.name)
        if crate is None:
            if config.crates:
                out.append(Finding(manifest, 1, "L601", "warning", f"crate `{pkg.name}` is not in craftbook.toml's layer map"))
            continue
        if crate.layer in ("kernel", "domain"):
            for dep in sorted(pkg.deps & IO_CRATES):
                out.append(Finding(manifest, 1, "L604", "error", f"{crate.layer} crate depends on I/O crate `{dep}`: put it behind a port in an adapter crate"))
            out += _io_in_source(repo, pkg, crate.layer, config.exempt)
        for dep in sorted(pkg.deps & names):
            out += _edge(manifest, crate, config.crates.get(dep), dep, config)
    return out


def _edge(manifest: str, src, dst, dep: str, config) -> list:
    if dst is None:
        return []
    if dst.layer not in ALLOWED[src.layer]:
        return [Finding(manifest, 1, "L602", "error", f"{src.layer} crate `{src.name}` depends on {dst.layer} crate `{dep}`: dependencies point inwards (ch.7 §7.3)")]
    crosses = src.context and dst.context and src.context != dst.context
    if crosses and dst.layer != "kernel" and "mixed" not in (src.layer, dst.layer) and (src.context, dst.context) not in config.edges:
        return [Finding(manifest, 1, "L603", "error", f"context `{src.context}` uses `{dst.context}` (`{dep}`) with no edge in the context map")]
    return []


def _io_in_source(repo: Path, pkg: Package, layer: str, exempt: list) -> list:
    out = []
    for path in sorted((pkg.root / "src").rglob("*.rs")):
        rel = str(path.relative_to(repo))
        if any(fnmatch.fnmatch(rel, pattern) for pattern, _ in exempt):
            continue
        for n, line in enumerate(path.read_text(errors="replace").split("\n"), 1):
            if line.lstrip().startswith("//"):
                continue
            m = IO_SOURCE.search(line)
            if m:
                out.append(Finding(rel, n, "L605", "error", f"`{m.group(0)}` in a {layer} crate: I/O and async belong to adapters"))
    return out
