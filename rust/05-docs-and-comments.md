# 5. Documentation and comments

## 5.1 What must be documented

| Item | Required doc |
|---|---|
| Crate (`lib.rs`) | `//!`: what the crate is, its layer and bounded context, what it must not know about |
| Every file | `//!`: one paragraph, what this file's primary item is for |
| Every `pub` item | `///` summary sentence (`missing_docs` is denied) |
| Fallible `pub fn` | `# Errors` section: each error variant and when (`clippy::missing_errors_doc`) |
| `pub fn` that can panic | `# Panics` section (`clippy::missing_panics_doc`) |
| `unsafe fn` | `# Safety` section |
| Non-obvious public API | `# Examples` with a doc test that compiles and runs |
| Math | the formula or the reference (paper, book, section) the code implements |

## 5.2 How long

- The first line of a doc is one sentence, ≤ 100 characters, that says what
  the item **is** or **does**, in the present tense: "Rotates a vector by this
  rotor." Not "This function is used to…".
- A file's `//!` is at most 30 lines. Design rationale, history and
  alternatives belong in `docs/` (design notes) or an ADR, and the module doc
  links there.
- A doc says what and why, and what the caller must know (invariants, units,
  cost, thread-safety). It does not restate the signature.

## 5.3 Comments in code

A comment explains **why**, never **what**. If a comment is needed to
explain what code does, rename or extract until it is not.

Required comments:

- `// SAFETY:` before every `unsafe` block.
- The reason on every `#[expect(…, reason = "…")]`.
- A short note where code deliberately deviates from the obvious approach
  ("not using `sort_unstable`: order of equal keys is observable in the
  replay file").

Forbidden:

- Commented-out code. Version control remembers it.
- History narration: "changed X to Y", "was …", "new:", "fix for …".
- Banner and separator comments (`// ===== helpers =====`). If a file needs
  sections, it is several files.
- `TODO`/`FIXME` without an issue reference: write `// TODO(#123): …`.
  A `TODO` without one is rejected by the checker.
- Names of people, dates, ticket systems other than the issue reference.

## 5.4 Project documents

Each project keeps, at minimum:

| File | Content |
|---|---|
| `README.md` | what it is, how to build, how to run tests |
| `AGENTS.md` / `CLAUDE.md` | link to Craftbook + only the project's own rules |
| `docs/glossary.md` | the ubiquitous language, per bounded context ([§7.1](07-architecture-ddd.md#71-ubiquitous-language)) |
| `docs/context-map.md` | the bounded contexts and how they relate ([§7.2](07-architecture-ddd.md#72-bounded-contexts-and-the-context-map)) |
| `docs/adr/NNN-title.md` | one accepted decision per file: context, decision, consequences |
| `craftbook.toml` | the project's layer map and checker configuration ([chapter 6](06-tooling.md)) |

Docs are updated in the same change that makes them wrong.
