# Craftbook

Engineering standards for our Rust projects. One book, read by people and by
coding agents, that answers the questions review keeps asking: how long a file
may get, when a module becomes a directory, how long a function may be, what
things are called, and when a function belongs on a type and when it does not.

The standard is written from first principles, at the strictness of the
.NET Framework Design Guidelines and with domain-driven design as the
architecture method. It is not a description of how our code looks today;
today's code is measured against it, and the gap is a migration plan.

Craftbook is project-neutral. Each project that follows it links here from its
`CONTRIBUTING.md` / `AGENTS.md` / `CLAUDE.md` and keeps only its *own* rules
(dependency graph, domain boundaries) locally. A project does not
relax a rule locally: the only exceptions are the kinds Craftbook itself
defines (performance kernels, chapter 7 §7.8; exempt paths with a reason in
`craftbook.toml`). A rule that is wrong for a project is changed here, for
everyone.

## The rules

| Chapter | Covers |
|---|---|
| [1. Files and modules](rust/01-files-and-modules.md) | file length, one primary item per file, `lib.rs`/`mod.rs`, when to split into submodules, where tests live |
| [2. Functions](rust/02-functions.md) | function length, nesting, parameters, free functions vs methods vs traits |
| [3. Naming](rust/03-naming.md) | casing, words, types, functions, modules, crates |
| [4. Types and errors](rust/04-types-and-errors.md) | newtypes, derives, visibility, errors, panics, `unsafe`, globals, async |
| [5. Docs and comments](rust/05-docs-and-comments.md) | what is documented, how long, what a comment is for |
| [6. Tooling](rust/06-tooling.md) | `rustfmt`, `clippy`, workspace lints, `craftcheck`, CI |
| [7. Architecture: DDD](rust/07-architecture-ddd.md) | ubiquitous language, bounded contexts, layers, entities, value objects, aggregates, ports |
| [8. Testing and quality](rust/08-testing-and-quality.md) | coverage per layer, test pyramid, contract tests, CI gates, definition of done |
| [9. Review checklist](rust/09-review-checklist.md) | what a reviewer checks by hand |

## Limits at a glance

| What | Target | Hard limit | Enforced by |
|---|---|---|---|
| Production lines in a `.rs` file (without its test module) | ≤ 250 | **400** | `craftcheck` C101 |
| Inline `#[cfg(test)]` module | ≤ 150 | **150**, then `tests.rs` | `craftcheck` C102 |
| `lib.rs` / `mod.rs` of a split module | ≤ 60 | **100**, wiring only | `craftcheck` C103 |
| Files in one module directory | ≤ 7 | **12** | `craftcheck` C104 |
| Lines in a function body (tests too) | ≤ 25 | **50** | `clippy::too_many_lines` |
| Parameters, `self` included | ≤ 4 | **5** | `clippy::too_many_arguments` |
| Nested blocks in a method or test (free function: +1) | ≤ 2 | **3** | `clippy::excessive_nesting` |
| Module depth below `src/` | ≤ 3 | **4** | `craftcheck` C105 |
| Cognitive complexity | ≤ 10 | **15** | `clippy::cognitive_complexity` |
| Line coverage: shared kernel / domain / application / adapters | | **95 / 90 / 85 / 70 %** | `cargo llvm-cov` |
| Coverage of changed lines in shared-kernel, domain, app crates | | **90 %** | CI |
| Mutation score: shared kernel / domain / application | | **85 / 80 / 70 %** | `cargo mutants` |

Hard limits are not targets. Code at the hard limit is a review finding that
just has not been written yet.

## How a project adopts it

1. Copy [`templates/clippy.toml`](templates/clippy.toml),
   [`templates/rustfmt.toml`](templates/rustfmt.toml) and
   [`templates/deny.toml`](templates/deny.toml) to the workspace root, and
   add the `[workspace.lints]` block from
   [`templates/workspace-lints.toml`](templates/workspace-lints.toml) to the
   root `Cargo.toml` (each member gets `[lints] workspace = true`).
2. Write the project's `craftbook.toml` (layer map and context map), record
   today's findings as a baseline, and add [`templates/ci.yml`](templates/ci.yml)
   as the CI workflow ([chapter 6 §6.3](rust/06-tooling.md#63-craftcheck)).
3. Add `adoption/<project>.md` here: what the project measures today, its
   bounded contexts, and the migration plan in PR-sized waves.
4. Link this repository from the project's agent and contributor guides.

Current adoption pages: [Meridian-Engine](adoption/meridian-engine.md),
[Substrate-Platform](adoption/substrate-platform.md).

## Changing a rule

A rule changes by pull request to this repository, with the reason in the PR.
Projects follow the change in their own next PR; nobody keeps an old version of
a rule alive locally.

Русская версия кратко: [README.ru.md](README.ru.md).
