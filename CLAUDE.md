# CLAUDE.md — Craftbook (mirrors AGENTS.md)

This repository is the engineering standard itself, not a code project.

- The rules live in `rust/`; `README.md` holds the index and the limits table,
  `README.ru.md` a Russian summary. A change to a number or a rule updates all
  three in the same commit.
- `tools/craftcheck` must stay in sync with `rust/06-tooling.md` (the rule
  table) and with the rules it checks. Run its tests:
  `python3 -m unittest discover -s tools/craftcheck/tests`.
- `adoption/<project>.md` pages are measurements and migration plans. Every
  number in them comes from a command; re-measure rather than edit numbers by
  hand.
- Write rules as instructions with a reason, not as descriptions of current
  code. Keep chapters project-neutral: project names appear only as examples
  and in `adoption/`.
