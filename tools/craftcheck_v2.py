#!/usr/bin/env python3
"""Validate Craftbook v2 policy; Rust semantics belong to rustc and Cargo."""
import argparse
import pathlib
import sys
import tomllib

PROFILES = {"simple", "layered", "ddd", "hexagonal", "clean", "cqrs",
            "event-sourcing", "actor", "systems", "embedded", "performance",
            "functional-core", "plugin"}
TABLES = {"assurance": {"format", "clippy", "test", "docs"},
          "constraints": {"no_std", "public_api_stable", "unsafe_review_required"}}


def validate(path):
    try:
        raw = tomllib.loads(path.read_text(encoding="utf-8"))
    except (OSError, UnicodeError, tomllib.TOMLDecodeError) as exc:
        return [str(exc)]
    errors = []
    if raw.get("version") != 2:
        errors.append("version must be 2")
    if not isinstance(raw.get("project"), str) or not raw["project"].strip():
        errors.append("project must be nonempty string")
    profiles = raw.get("profiles", [])
    if not isinstance(profiles, list) or not all(isinstance(x, str) for x in profiles):
        errors.append("profiles must be list of strings")
    else:
        errors.extend(f"unknown profile: {p}" for p in profiles if p not in PROFILES)
        if len(profiles) != len(set(profiles)):
            errors.append("duplicate profiles")
    for name, allowed in TABLES.items():
        data = raw.get(name, {})
        if not isinstance(data, dict):
            errors.append(f"{name} must be a table")
            continue
        for key, value in data.items():
            if key not in allowed:
                errors.append(f"unknown {name} option: {key}")
            if not isinstance(value, bool):
                errors.append(f"{name}.{key} must be boolean")
    errors.extend(f"unknown root option: {key}" for key in raw
                  if key not in {"version", "project", "profiles", *TABLES})
    return errors


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("config", nargs="?", type=pathlib.Path, default=pathlib.Path("craftbook.toml"))
    args = p.parse_args()
    errors = validate(args.config)
    for error in errors:
        print(f"error: {error}", file=sys.stderr)
    if errors:
        return 1
    print(f"Craftbook v2 policy valid: {args.config}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
