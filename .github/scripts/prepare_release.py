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


def next_version(version: str, bump: str, channel: str) -> str:
    match = re.fullmatch(r"(\d+)\.(\d+)\.(\d+)(?:-(alpha|beta|rc)(\d+))?", version)
    if not match:
        raise ValueError(f"Invalid current version: {version}")
    major, minor, patch = (int(match.group(i)) for i in (1, 2, 3))
    current_channel, current_number = match.group(4), match.group(5)
    if bump == "current":
        if current_channel is None:
            raise ValueError("Current bump requires a prerelease")
        target_base = f"{major}.{minor}.{patch}"
    elif bump == "patch":
        target_base = f"{major}.{minor}.{patch + 1}"
    elif bump == "minor":
        target_base = f"{major}.{minor + 1}.0"
    elif bump == "major":
        target_base = f"{major + 1}.0.0"
    else:
        raise ValueError(f"Unknown bump: {bump}")
    if channel == "stable":
        if bump == "current" and current_channel is None:
            raise ValueError("Already stable")
        return target_base
    if channel not in ("alpha", "beta", "rc"):
        raise ValueError(f"Unknown channel: {channel}")
    order = {"alpha": 0, "beta": 1, "rc": 2}
    if bump == "current" and current_channel is not None:
        if order[channel] < order[current_channel]:
            raise ValueError("Cannot downgrade a prerelease channel for the same version")
        number = int(current_number) + 1 if channel == current_channel else 1
    else:
        number = 1
    return f"{target_base}-{channel}{number}"


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


def prepare(files: dict[str, str], today: str, bump: str = "current", channel: str = "rc") -> tuple[str, dict[str, str]]:
    import tomllib

    manifest = tomllib.loads(files["tools/cargo-craftcheck/Cargo.toml"])
    lock = tomllib.loads(files["tools/cargo-craftcheck/Cargo.lock"])
    current = manifest["package"]["version"]
    lock_version = next(p["version"] for p in lock["package"] if p["name"] == "cargo-craftcheck")
    if lock_version != current:
        raise ValueError("Cargo.toml and Cargo.lock do not match")
    target = next_version(current, bump, channel)
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
    parser.add_argument("--bump", choices=("current", "patch", "minor", "major"), default="current")
    parser.add_argument("--channel", choices=("alpha", "beta", "rc", "stable"), default="rc")
    args = parser.parse_args()
    paths = (*FILES, "CHANGELOG.md")
    original = {name: (ROOT / name).read_text(encoding="utf-8") for name in paths}
    target, updated = prepare(original, datetime.datetime.now(datetime.timezone.utc).date().isoformat(), args.bump, args.channel)
    for name, content in updated.items():
        (ROOT / name).write_text(content, encoding="utf-8")
    args.output.write_text(f"version={target}\ntag=v{target}\n", encoding="utf-8")


if __name__ == "__main__":
    main()
