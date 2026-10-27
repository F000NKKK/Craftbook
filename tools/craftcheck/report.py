"""Findings, their printing, the baseline and the review journal."""

import csv
from dataclasses import dataclass
from pathlib import Path


@dataclass(frozen=True)
class Finding:
    path: str
    line: int
    code: str
    level: str  # "error" or "warning"
    message: str

    def key(self) -> str:
        """Identity that survives unrelated edits: no line number."""
        return f"{self.path}\t{self.code}\t{self.message}"

    def __str__(self) -> str:
        return f"{self.path}:{self.line}: {self.code} {self.level}: {self.message}"


def read_baseline(path: Path) -> set:
    if not path.is_file():
        return set()
    return {line for line in path.read_text().splitlines() if line.strip()}


def write_baseline(path: Path, findings: list) -> None:
    path.write_text("".join(f"{k}\n" for k in sorted({f.key() for f in findings})))


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
