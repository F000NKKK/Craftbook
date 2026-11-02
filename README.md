<div align="center">

<a id="top"></a>

# 📖 Craftbook

### A practical engineering handbook for idiomatic Rust — from libraries to operating systems

[![cargo-craftcheck](https://img.shields.io/crates/v/cargo-craftcheck.svg?label=cargo-craftcheck&cacheSeconds=86400)](https://crates.io/crates/cargo-craftcheck)
[![docs.rs](https://img.shields.io/docsrs/cargo-craftcheck?cacheSeconds=86400)](https://docs.rs/cargo-craftcheck)
[![Downloads](https://img.shields.io/crates/d/cargo-craftcheck.svg?cacheSeconds=86400)](https://crates.io/crates/cargo-craftcheck)
[![CI](https://github.com/F000NKKK/Craftbook/actions/workflows/craftbook-v2.yml/badge.svg)](https://github.com/F000NKKK/Craftbook/actions/workflows/craftbook-v2.yml)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](tools/cargo-craftcheck/Cargo.toml)
![Rust](https://img.shields.io/badge/language-Rust-CE422B)
![Edition 2024](https://img.shields.io/badge/edition-2024-informational)
![Handbook v2](https://img.shields.io/badge/handbook-v2-blue)
![Platforms](https://img.shields.io/badge/platforms-Linux%20%7C%20Windows%20%7C%20macOS-informational)

🇺🇸 **English** | 🇷🇺 [Русский](README.ru.md)

[Overview](#-overview) · [Principles](#-principles) · [Handbook](#-handbook) · [Getting started](#-getting-started) · [Tooling](#-tooling) · [Architecture](#-architecture) · [Development](#development)

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

Install the Craftbook policy checker:

```bash
cargo install cargo-craftcheck --version 2.0.0-rc9
cargo craftcheck check
```

The checker validates **policy configuration** and optional direct Cargo Workspace dependency rules; it does not inspect Rust source architecture.

To run the same checks in GitHub Actions:

```yaml
steps:
  - uses: actions/checkout@v4
  - uses: dtolnay/rust-toolchain@stable
  - uses: F000NKKK/Craftbook@v2.0.0-rc9
    with:
      command: both
      report-format: sarif
```

The action uploads check reports as a workflow artifact by default. A `CraftBook.toml` policy is required in the repository root unless `config` points elsewhere.

## 🛠️ Tooling

| Tool | Purpose | Status |
|---|---|---|
| `cargo fmt` | Format Rust sources | Standard tooling |
| `cargo clippy` | Compiler-aware diagnostics | Standard tooling |
| [`cargo-craftcheck`](tools/cargo-craftcheck/README.md) | Validate v2 policy and opt-in workspace edges | Release candidate |

```bash
cargo fmt --all --check --manifest-path tools/cargo-craftcheck/Cargo.toml
cargo clippy --all-targets --manifest-path tools/cargo-craftcheck/Cargo.toml -- -D warnings
cargo test --manifest-path tools/cargo-craftcheck/Cargo.toml
```

Configuration: [Craftbook's own policy](CraftBook.toml) · [example policy](templates/CraftBook.toml) · [workspace lints](templates/workspace-lints.toml) · [Clippy](templates/clippy.toml) · [rustfmt](templates/rustfmt.toml) · [CI template](templates/ci.yml).

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

## Development

The handbook does not replace project-specific requirements, measurement or code review.
