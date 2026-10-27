"""Crate README rules (Craftbook ch. 10 §10.2): R801 a crate has no README.md,
R802 its title is not the crate's name."""

import re
import tomllib
from pathlib import Path

from report import Finding

TITLE = re.compile(r"^#\s+(\S+)", re.M)


def check_readmes(repo: Path, packages: list) -> list:
    out = []
    for pkg in packages:
        manifest = str((pkg.root / "Cargo.toml").relative_to(repo))
        readme = pkg.root / "README.md"
        if pkg.root.resolve() == repo.resolve():
            continue  # a crate at the repository root is described by the product README
        if not readme.is_file():
            out.append(Finding(manifest, 1, "R801", "warning", f"crate `{pkg.name}` has no README.md next to its Cargo.toml", pkg.name))
            continue
        title = TITLE.search(readme.read_text(errors="replace"))
        if title and title.group(1).strip("`") != pkg.name:
            out.append(Finding(str(readme.relative_to(repo)), 1, "R802", "warning", f"README title `{title.group(1)}` is not the crate name `{pkg.name}`", pkg.name))
    return out

