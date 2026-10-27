#!/usr/bin/env python3
"""Fails when the mutation score in a cargo-mutants output is below a threshold.

    python3 tools/mutation_score.py mutants.out/outcomes.json 80

Score = caught / (caught + missed). Unviable mutants (they do not compile) and
timeouts are left out: a timeout is reported, not counted.
"""

import json
import sys
from collections import Counter


def score(outcomes: dict) -> tuple:
    counts = Counter(o.get("summary", "") for o in outcomes.get("outcomes", []) if o.get("scenario") != "Baseline")
    caught, missed = counts["CaughtMutant"], counts["MissedMutant"]
    tested = caught + missed
    return (100.0 * caught / tested if tested else 100.0), counts


def main(argv: list) -> int:
    if len(argv) != 2:
        print(__doc__, file=sys.stderr)
        return 2
    with open(argv[0]) as fh:
        value, counts = score(json.load(fh))
    threshold = float(argv[1])
    print(f"mutation score {value:.1f} % (caught {counts['CaughtMutant']}, missed {counts['MissedMutant']}, "
          f"timeout {counts['Timeout']}, unviable {counts['Unviable']}); threshold {threshold:.0f} %")
    return 0 if value >= threshold else 1


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
