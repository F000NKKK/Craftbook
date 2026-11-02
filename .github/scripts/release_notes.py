#!/usr/bin/env python3
"""Generate GitHub release notes from CHANGELOG.md and the Git history."""

import argparse
import re
import subprocess
from pathlib import Path


def changelog_section(text: str, tag: str) -> str:
    version = tag.removeprefix("v")
    aliases = {version, tag, "v" + version}
    match = re.fullmatch(r"2\.0\.0-rc(\d+)", version)
    if match:
        aliases.add(f"v2-rc{match.group(1)}")
    lines = text.splitlines()
    start = next(
        (i for i, line in enumerate(lines) if line.startswith("## ") and
         re.split(r"\s+[—-]\s+", line[3:].strip(), maxsplit=1)[0] in aliases),
        None,
    )
    if start is None:
        raise ValueError(f"Release {tag} has no section in CHANGELOG.md")
    end = next((i for i in range(start + 1, len(lines)) if lines[i].startswith("## ")), len(lines))
    section = "\n".join(lines[start + 1:end]).strip()
    if not section:
        raise ValueError(f"Release {tag} has an empty changelog section")
    return section


def git(*args: str) -> str:
    return subprocess.check_output(["git", *args], text=True).strip()


def commit_summary(tag: str) -> str:
    previous = git("describe", "--tags", "--abbrev=0", f"{tag}^") if git(
        "rev-list", "--count", f"{tag}^"
    ) != "0" else ""
    if not previous:
        return ""
    commits = git("log", "--format=%s", f"{previous}..{tag}")
    return "\n".join(f"- {subject}" for subject in commits.splitlines() if subject)


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--tag", required=True)
    parser.add_argument("--changelog", type=Path, default=Path("CHANGELOG.md"))
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    section = changelog_section(args.changelog.read_text(encoding="utf-8"), args.tag)
    subjects = commit_summary(args.tag)
    notes = f"# Craftbook {args.tag}\n\n{section}\n"
    if subjects:
        notes += f"\n## Changes since previous tag\n\n{subjects}\n"
    args.output.write_text(notes, encoding="utf-8")


if __name__ == "__main__":
    main()
