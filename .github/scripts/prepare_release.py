#!/usr/bin/env python3
"""Prepare a release commit from the current prerelease and Unreleased notes."""

import argparse
import datetime
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
FILES = (
    "tools/cargo-craftcheck/Cargo.toml",
    "tools/cargo-craftcheck/Cargo.lock",
    "action.yml",
    "README.md",
    "README.ru.md",
    "tools/cargo-craftcheck/README.md",
)


def next_rc(version: str) -> str:
    match = re.fullmatch(r"(\d+\.\d+\.\d+)-rc(\d+)", version)
    if not match:
        raise ValueError("Automatic release requires a current RC version")
    return f"{match.group(1)}-rc{int(match.group(2)) + 1}"


def prepare_changelog(contents: str, version: str, date: str) -> str:
    marker = "## Unreleased"
    if contents.count(marker) != 1:
        raise ValueError("Expected exactly one Unreleased heading")
    head, tail = contents.split(marker, 1)
    next_heading = re.search(r"(?m)^## ", tail)
    if next_heading is None:
        raise ValueError("No previous release section")
    pending = tail[:next_heading.start()].strip()
    if not pending or not any(line.strip().startswith("- ") for line in pending.splitlines()):
        raise ValueError("Unreleased must contain at least one changelog item")
    if re.search(rf"(?m)^## v?{re.escape(version)}(?:\s|$)", contents):
        raise ValueError("Target release section already exists")
    return (
        f"{head}{marker}\n\n## v{version} — {date}\n\n"
        f"{pending}\n\n{tail[next_heading.start():]}"
    )


def prepare(files: dict[str, str], today: str) -> tuple[str, dict[str, str]]:
    import tomllib

    manifest = tomllib.loads(files["tools/cargo-craftcheck/Cargo.toml"])
    lock = tomllib.loads(files["tools/cargo-craftcheck/Cargo.lock"])
    current = manifest["package"]["version"]
    lock_version = next(p["version"] for p in lock["package"] if p["name"] == "cargo-craftcheck")
    if lock_version != current:
        raise ValueError("Cargo.toml and Cargo.lock do not match")
    target = next_rc(current)
    changed = {}
    for name in FILES:
        content = files[name]
        if current not in content:
            raise ValueError(f"{name} does not contain {current}")
        changed[name] = content.replace(current, target)
    changed["CHANGELOG.md"] = prepare_changelog(files["CHANGELOG.md"], target, today)
    return target, changed


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    paths = (*FILES, "CHANGELOG.md")
    original = {name: (ROOT / name).read_text(encoding="utf-8") for name in paths}
    target, updated = prepare(original, datetime.date.today().isoformat())
    for name, content in updated.items():
        (ROOT / name).write_text(content, encoding="utf-8")
    args.output.write_text(f"version={target}\ntag=v{target}\n", encoding="utf-8")


if __name__ == "__main__":
    main()
