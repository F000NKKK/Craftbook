<div align="center">

<a id="top"></a>

# 📖 Craftbook

### A practical engineering handbook for idiomatic Rust — from libraries to operating systems

![Rust](https://img.shields.io/badge/language-Rust-CE422B)
![Edition 2024](https://img.shields.io/badge/edition-2024-informational)
![Handbook v2](https://img.shields.io/badge/handbook-v2-blue)
[![CI](https://github.com/F000NKKK/Craftbook/actions/workflows/craftbook-v2.yml/badge.svg)](https://github.com/F000NKKK/Craftbook/actions/workflows/craftbook-v2.yml)
![cargo-craftcheck](https://img.shields.io/badge/cargo--craftcheck-experimental-yellow)

🇺🇸 **English** | 🇷🇺 [Русский](README.ru.md)

[Overview](#-overview) · [Principles](#-principles) · [Handbook](#-handbook) · [Getting started](#-getting-started) · [Tooling](#-tooling) · [Architecture](#-architecture) · [Development](#-development)

</div>

---

## 📖 Overview

**Craftbook** is a project-neutral Rust engineering handbook, combining design guidance, architectural choices and tools. It is useful for libraries, CLIs, services, games, engines, embedded systems, kernels and desktop applications.

**Not a framework:** no required DDD hierarchy, one-type-per-file rule or universal code-size limit. The goal is to design for actual constraints while preserving Rust's strengths: ownership, sound abstractions, explicit failure and predictable resource lifetimes.

## 🎯 Principles

| Level | Meaning |
|---|---|
| **MUST** | Correctness, soundness, safety and explicit project contracts |
| **SHOULD** | Idiomatic defaults, with justified deviations |
| **MAY** | Optional approaches and tools |
| **PROJECT** | Targets, performance, compatibility, security and reliability requirements |

- 🦀 **Rust first:** enums, traits, ownership, composition and modules rather than mechanically translated C# hierarchies.
- 🧩 **Patterns are choices:** DDD, Hexagonal, CQRS, Actors and data-oriented design are all optional.
- 🛡️ **Explicit guarantees:** document invariants, errors, unsafe conditions and lifecycle assumptions.
- 🔬 **Verification by risk:** use tests, benchmarks and reviews for meaningful failure modes instead of arbitrary quotas.
- 🔧 **Tools for machine-checkable properties:** rely on Cargo, rustc, rustfmt and Clippy; add specialized checks when justified.

## 📚 Handbook

| Foundation | What it covers |
|---|---|
| [01 · Modules and crates](rust/01-files-and-modules.md) | Modules, visibility and workspaces |
| [02 · Functions and abstractions](rust/02-functions.md) | Methods, traits, free functions |
| [03 · Naming and API](rust/03-naming.md) | Names, conversions and contracts |
| [04 · Types and errors](rust/04-types-and-errors.md) | State, safety and resource handling |
| [05 · Documentation](rust/05-docs-and-comments.md) | Rustdoc and design rationale |

| Engineering | What it covers |
|---|---|
| [06 · Tooling](rust/06-tooling.md) | CI, formatting and diagnostics |
| [07 · Architecture](rust/07-architecture-ddd.md) | DDD, Hexagonal and other profiles |
| [08 · Testing and quality](rust/08-testing-and-quality.md) | Verification, fuzzing and benchmarks |
| [09 · Code review](rust/09-review-checklist.md) | Context-aware review checklist |
| [10 · READMEs](rust/10-readmes.md) | Project documentation |

| Advanced Rust | What it covers |
|---|---|
| [11 · Ownership and lifetimes](rust/11-ownership-and-lifetimes.md) | Borrowing, sharing, RAII, Pin |
| [12 · Async and concurrency](rust/12-async-and-concurrency.md) | Futures, actors, cancellation |
| [13 · Unsafe and FFI](rust/13-unsafe-and-ffi.md) | Soundness and low-level interfaces |
| [14 · Cargo and compatibility](rust/14-cargo-and-compatibility.md) | Features, semver, portability |

## 🚀 Getting started

Read the [principles](#-principles), select a suitable [architecture profile](rust/07-architecture-ddd.md) and use [the example policy](templates/CraftBook.toml) as a starting point. No checker is required just to apply the handbook.

For the experimental native Rust policy checker:

```bash
git clone https://github.com/F000NKKK/Craftbook.git
cd Craftbook
cargo install --path tools/cargo-craftcheck
cargo craftcheck check --config templates/CraftBook.toml
```

The checker validates **policy configuration** and optional direct Cargo Workspace dependency rules; it does not inspect Rust source architecture.

## 🛠️ Tooling

| Tool | Purpose | Status |
|---|---|---|
| `cargo fmt` | Format Rust sources | Standard tooling |
| `cargo clippy` | Compiler-aware diagnostics | Standard tooling |
| [`cargo-craftcheck`](tools/cargo-craftcheck/README.md) | Validate v2 policy and opt-in workspace edges | Experimental |
| [Legacy craftcheck](tools/craftcheck) | Historical v1 checks | Migration only |

```bash
cargo fmt --all --check --manifest-path tools/cargo-craftcheck/Cargo.toml
cargo clippy --all-targets --manifest-path tools/cargo-craftcheck/Cargo.toml -- -D warnings
cargo test --manifest-path tools/cargo-craftcheck/Cargo.toml
```

Configuration: [project policy](templates/CraftBook.toml) · [workspace lints](templates/workspace-lints.toml) · [Clippy](templates/clippy.toml) · [rustfmt](templates/rustfmt.toml) · [CI template](templates/ci.yml).

## 🏗️ Architecture

Craftbook adapts to the system rather than forcing every project into a layered architecture.

| Scenario | Possible choices |
|---|---|
| Library or CLI | Cohesive modules, direct functions, simple composition |
| Business service | DDD, Hexagonal, Clean Architecture |
| Audit or event-heavy systems | CQRS, Event Sourcing, message-driven design |
| Concurrent services | Actor Model, channels, explicit task ownership |
| Game or simulation | ECS, data-oriented design, measurable hot paths |
| Kernel or embedded | `no_std`, RAII, typestate, safe unsafe boundaries |

Choose combinations based on invariants and tradeoffs. See [architecture guidance](rust/07-architecture-ddd.md).

## 👩‍💻 Development

Craftbook v2 evolves alongside real Rust code. Review chapter updates together with examples and tool behavior. The native checker has a [dedicated CI workflow](.github/workflows/craftbook-v2.yml); its successful execution is not implied by the existence of the workflow.

Moving from v1? Read the [migration guide](MIGRATION.md) ([на русском](MIGRATION.ru.md)). Old Python rules are intentionally not mandatory for v2.

Craftbook is a handbook, not a correctness proof: project requirements, measurements and code review still matter.

---

<div align="center">

[⬆ Back to top](#top) · [Русская версия](README.ru.md) · [Migration guide](MIGRATION.md)

</div>

The native checker also offers an experimental opt-in dependency rule check: `cargo craftcheck deps --config CraftBook.toml --manifest-path Cargo.toml`. It uses Cargo Metadata and checks direct workspace dependencies only.

Nested `CraftBook.toml` files override parent settings; run `cargo craftcheck check` or `cargo craftcheck deps` from any configured directory to check its subtree. [Configuration details](tools/cargo-craftcheck/README.md#hierarchical-configuration).
