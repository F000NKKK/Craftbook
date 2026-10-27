"""craftcheck — Craftbook's checker for what clippy and rustfmt cannot see.

    python3 tools/craftcheck check  REPO [--baseline FILE] [--write-baseline FILE] [--journal FILE]
    python3 tools/craftcheck deps   REPO [--baseline FILE] [--write-baseline FILE]
    python3 tools/craftcheck crates REPO --layer LAYER

All take --config FILE to use another craftbook.toml than REPO/craftbook.toml.

`check` applies the file, module, naming and comment rules to every tracked
.rs file; `deps` applies the layer and context-map rules to the crate graph,
using REPO/craftbook.toml; `crates` prints the packages mapped to one layer,
one per line. Exit status 1 when an error that is not in the baseline is
found. A baseline lets a project adopt the rules gradually: it counts known
findings per file, rule and subject, and only a rising count fails.
"""

import argparse
import sys
from pathlib import Path

import config as config_mod
import deps_rules
import report
import workspace
from checker import run_check


def parse_args(argv: list):
    parser = argparse.ArgumentParser(prog="craftcheck", description=__doc__.split("\n\n")[0])
    parser.add_argument("command", choices=["check", "deps", "crates"])
    parser.add_argument("repo", type=Path)
    parser.add_argument("--config", type=Path, help="craftbook.toml to use instead of REPO/craftbook.toml")
    parser.add_argument("--baseline", type=Path)
    parser.add_argument("--write-baseline", type=Path)
    parser.add_argument("--journal", type=Path)
    parser.add_argument("--layer", choices=config_mod.LAYERS, help="crates: the layer to list")
    parser.add_argument("--quiet", action="store_true", help="print errors only")
    args = parser.parse_args(argv)
    if args.command == "crates" and not args.layer:
        parser.error("crates needs --layer")
    return args


def crates_in_layer(cfg, layer: str) -> list:
    return [c.name for c in cfg.crates.values() if c.layer == layer]


def main(argv: list) -> int:
    args = parse_args(argv)
    repo = args.repo.resolve()
    cfg = config_mod.load(repo, args.config)
    if args.command == "crates":
        for name in crates_in_layer(cfg, args.layer):
            print(name)
        return 0
    if args.command == "check":
        files, findings = run_check(repo, cfg)
        if args.journal:
            report.write_journal(args.journal, files, findings)
    else:
        findings = deps_rules.check_graph(repo, workspace.discover(repo), cfg)
    if args.write_baseline:
        report.write_baseline(args.write_baseline, findings)
    return _print(args, findings)


def _print(args, findings: list) -> int:
    known = report.read_baseline(args.baseline) if args.baseline else {}
    new = report.new_findings(findings, known)
    for f in sorted(new, key=lambda f: (f.path, f.line, f.code)):
        if not args.quiet or f.level == "error":
            print(f)
    errors = sum(1 for f in new if f.level == "error")
    warnings = len(new) - errors
    print(f"craftcheck {args.command}: {errors} errors, {warnings} warnings ({len(findings) - len(new)} in baseline)", file=sys.stderr)
    return 1 if errors else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
