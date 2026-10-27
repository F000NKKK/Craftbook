# 10. READMEs

A README is the front door: someone who has never seen the project decides in
a minute whether it is for them and how to start. Every product repository and
every Rust crate has one, written to the shape below. The references are
Substrate-Platform, Yog-IDLE and Yog-Mod-Loader; the templates are in
[`templates/readme/`](../templates/readme/).

## 10.1 Product README (repository root)

Two files, kept in step: `README.md` (English) and `README.ru.md` (Russian).
The Russian file is a translation of the same structure, not a summary; a
change to one is a change to both in the same commit.

Shape:

1. **Header block** in a centred `<div align="center">` with an `<a id="top">`
   anchor: icon or emoji, name, one-line tagline (what it is, for whom), badge
   row (licence, language/toolchain, version or release, platforms), the
   language switch (`🇺🇸 **English** | 🇷🇺 [Русский](README.ru.md)`), and a
   link row to the sections below. Closed with `</div>` and `---`.
2. **Overview** (`## 📖 Overview`): what it is in a short paragraph, a table
   or diagram of the parts (Mermaid is fine), and one screenshot or diagram
   when there is something to see. Say what it is *not* when that prevents a
   wrong guess ("knows nothing about any product").
3. **Features or parts**: grouped, each with one sentence of what it gives
   the reader, not how it is built. Tables for lists of modules or crates.
4. **Getting started** (`## 🚀 Getting Started`): the shortest path to a
   running result: install or add the dependency, a minimal working example
   that compiles, how to run it. Examples are real code that is built in CI
   (`cargo test --doc`, `examples/`), never pseudo-code.
5. **Architecture** (when there is more than one crate): the layers and
   contexts of [chapter 7](07-architecture-ddd.md) in one picture plus the
   crate table; link to `docs/` for the rest.
6. **Development** (`## 🛠️ Development`): build, test and check commands
   exactly as CI runs them, the pinned toolchain, and a link to Craftbook,
   `CONTRIBUTING.md` and `docs/`.
7. **Status** when the project is not stable: what works, what does not,
   what is next. Honest, one table.
8. **License** (`## 📄 License`): one paragraph and links.
9. Footer: `---` and a centred `[⬆ Back to Top](#top)`.

Rules:

- Section titles carry one emoji each, the same set in both languages.
- The README says what is true today. Plans live in `docs/roadmap.md`; the
  README links there.
- No history ("we rewrote…"), no marketing adjectives, no instructions that
  CI does not run.
- Images live in `docs/images/` with alt text; width 900 for screenshots.
- Every relative link resolves; a link checker runs in CI.
- Numbers (supported versions, crate counts) are written once; where they
  must repeat they are generated or checked.

## 10.2 Crate README (`crates/<name>/README.md`)

Every Rust crate in a workspace has its own `README.md` next to its
`Cargo.toml`, and `readme = "README.md"` in the manifest so crates.io and
docs.rs show it. English only; the product README.ru lists the crates in
Russian. Short (30–80 lines), in this order:

1. `# <crate-name>` and a one-sentence description (the same as
   `description` in the manifest).
2. **Badges** for crates.io/docs.rs only if the crate is published.
3. **Layer and context**: one line, from `craftbook.toml`
   (e.g. *domain · physics*), so the reader knows what it may depend on.
4. **What it is for / what it is not**: two or three bullets. The "not" line
   names the neighbour that owns the thing people would look for here.
5. **Usage**: a minimal example that compiles. For a crate whose API is
   large, link to docs.rs and show only the entry point. The example is the
   crate's doc test, included with `#![doc = include_str!("../README.md")]`
   in `lib.rs`, so it cannot rot.
6. **Features and platform notes** if any (cargo features table).
7. **Depends on / used by**: the direct workspace edges, as links to the
   neighbours' READMEs.
8. **License** line.

A crate README is not a design document: rationale goes to `docs/` and an ADR.
When a crate is renamed, split or merged, its README moves in the same
commit; `craftcheck` reports a crate without one (R801) and a README whose
title differs from the crate name (R802).

## 10.3 Other READMEs

- `docs/README.md` indexes the documents (read order, ADRs, glossary, context
  map).
- A directory of examples has a README listing each example, what it shows
  and the command that runs it.
- Generated directories are not given READMEs.
