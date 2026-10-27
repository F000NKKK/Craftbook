"""Rules about the crate graph: layers and the context map (Craftbook ch. 7)."""

from pathlib import Path

import io_rules
from report import Finding
from workspace import root_manifest

EVERY_LAYER = {"shared-kernel", "domain", "app", "infra", "interface", "mixed"}
ALLOWED = {
    "shared-kernel": {"shared-kernel"},
    "domain": {"shared-kernel", "domain"},
    "app": {"shared-kernel", "domain", "app"},
    "infra": {"shared-kernel", "domain", "app", "infra"},
    "interface": EVERY_LAYER,
    "mixed": EVERY_LAYER,
}
DOMAIN_RELATIONS = ("customer-supplier", "shared-kernel")
IO_CRATES = {"tokio", "async-std", "wgpu", "cpal", "winit", "reqwest", "hyper", "portable-pty", "notify", "git2", "tauri", "libc", "nix", "windows", "windows-sys", "rodio", "ash", "vulkano"}


def check_graph(repo: Path, packages: list, config) -> list:
    out = _lint_findings(repo, packages)
    names = {p.name for p in packages}
    for pkg in packages:
        manifest = str((pkg.root / "Cargo.toml").relative_to(repo))
        crate = config.crates.get(pkg.name)
        if crate is None:
            if config.crates:
                out.append(Finding(manifest, 1, "L601", "warning", f"crate `{pkg.name}` is not in craftbook.toml's layer map", pkg.name))
            continue
        if crate.layer in ("shared-kernel", "domain"):
            for dep in sorted(pkg.deps & IO_CRATES):
                out.append(Finding(manifest, 1, "L604", "error", f"{crate.layer} crate depends on I/O crate `{dep}`: put it behind a port in an adapter crate", dep))
            out += io_rules.scan_package(repo, pkg, crate.layer, config)
        for dep in sorted(pkg.deps & names):
            out += _edge(manifest, crate, config.crates.get(dep), config)
    return [f for f in out if not config.exempted(f.path, f.code)]


def _lint_findings(repo: Path, packages: list) -> list:
    workspace = root_manifest(repo).get("workspace")
    if workspace is None:
        return []
    if "lints" not in workspace:
        return [Finding("Cargo.toml", 1, "L607", "warning", "workspace has no [workspace.lints]: declare the lint set once for every crate")]
    out = []
    for pkg in packages:
        if not pkg.has_workspace_lints:
            manifest = str((pkg.root / "Cargo.toml").relative_to(repo))
            out.append(Finding(manifest, 1, "L606", "warning", "crate does not inherit the workspace lint set (`[lints] workspace = true`)"))
    return out


def _edge(manifest: str, src, dst, config) -> list:
    if dst is None:
        return []
    if dst.layer not in ALLOWED[src.layer]:
        return [Finding(manifest, 1, "L602", "error", f"{src.layer} crate `{src.name}` depends on {dst.layer} crate `{dst.name}`: dependencies point inwards (ch.7 §7.3)", dst.name)]
    crosses = src.context and dst.context and src.context != dst.context
    if not crosses or dst.layer == "shared-kernel" or "mixed" in (src.layer, dst.layer):
        return []
    relation = config.edges.get((src.context, dst.context))
    if src.layer == dst.layer == "domain":
        return [] if relation in DOMAIN_RELATIONS else [_domain_edge(manifest, src, dst, relation)]
    if relation is None:
        return [Finding(manifest, 1, "L603", "error", f"context `{src.context}` uses `{dst.context}` (`{dst.name}`) with no edge in the context map", dst.name)]
    return []


def _domain_edge(manifest: str, src, dst, relation) -> Finding:
    found = "no edge in the context map" if relation is None else f"a `{relation}` edge"
    return Finding(manifest, 1, "L603", "error", f"domain `{src.name}` uses domain `{dst.name}` of context `{dst.context}` with {found}: the relation must be customer-supplier", dst.name)
