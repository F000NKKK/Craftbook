"""Findings, their printing, the baseline and the review journal."""

import csv
from collections import Counter
from dataclasses import dataclass
from pathlib import Path


@dataclass(frozen=True)
class Finding:
    path: str
    line: int
    code: str
    level: str  # "error" or "warning"
    message: str  # for humans; may hold numbers that change with every edit
    subject: str = ""  # stable name of what the finding is about, or ""

    def key(self) -> tuple:
        """Identity that survives unrelated edits: no line, no numbers."""
        return (self.path, self.code, self.subject)

    def __str__(self) -> str:
        return f"{self.path}:{self.line}: {self.code} {self.level}: {self.message}"


def read_baseline(path: Path) -> dict:
    """Baseline file: one `path<TAB>code<TAB>subject<TAB>count` line per key."""
    if not path.is_file():
        return {}
    counts = {}
    for n, line in enumerate(path.read_text().splitlines(), 1):
        if not line.strip():
            continue
        fields = line.split("\t")
        if len(fields) != 4 or not fields[3].isdigit():
            raise ValueError(f"{path}:{n}: expected `path<TAB>code<TAB>subject<TAB>count`")
        counts[tuple(fields[:3])] = int(fields[3])
    return counts


def write_baseline(path: Path, findings: list) -> None:
    counts = Counter(f.key() for f in findings)
    path.write_text("".join("\t".join(key) + f"\t{n}\n" for key, n in sorted(counts.items())))


def new_findings(findings: list, baseline: dict) -> list:
    """Findings of every key whose count rose above the baseline's."""
    counts = Counter(f.key() for f in findings)
    return [f for f in findings if counts[f.key()] > baseline.get(f.key(), 0)]


def write_journal(path: Path, files: list, findings: list) -> None:
    """One row per file: its findings and a review status kept across runs.
    A file with findings is never `ok`."""
    old = {}
    if path.is_file():
        with open(path, newline="") as fh:
            old = {row["path"]: row for row in csv.DictReader(fh, delimiter="\t")}
    by_file = {}
    for f in findings:
        by_file.setdefault(f.path, []).append(f.code)
    path.parent.mkdir(parents=True, exist_ok=True)
    with open(path, "w", newline="") as fh:
        writer = csv.DictWriter(fh, fieldnames=["path", "findings", "status"], delimiter="\t")
        writer.writeheader()
        for rel in files:
            codes = " ".join(sorted(set(by_file.get(rel, []))))
            status = old.get(rel, {}).get("status", "todo")
            writer.writerow({"path": rel, "findings": codes, "status": "todo" if codes else status})
