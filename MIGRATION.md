<div align="center">

<a id="top"></a>

# 🔄 Craftbook Migration Guide

### Moving from the prescriptive v1 rules to the adaptable Rust v2 handbook

🇺🇸 **English** | 🇷🇺 [Русский](MIGRATION.ru.md)

[What changed](#-what-changed) · [Migration steps](#-migration-steps) · [Tooling](#-tooling) · [Compatibility](#-compatibility)

</div>

---

## 📖 What changed

Craftbook v2 changes the **meaning of the standard**, not merely its directory layout. v1 focused on strict, DDD-oriented implementation rules; v2 emphasizes Rust correctness, soundness, explicit project contracts and optional architectural profiles.

| v1 expectation | v2 approach |
|---|---|
| One primary type per file | Cohesive modules, with any number of related items |
| Fixed file, function and module limits | Size and complexity as review signals |
| DDD layering for every project | Optional DDD, Hexagonal, actor, systems and performance profiles |
| One bounded context per domain crate | Context boundaries independent of crate boundaries |
| Every I/O operation through a port | Ports when isolation is valuable |
| Universal `Send + Sync` or `Clone` expectations | Implement traits only when meaningful |
| Numeric coverage and mutation gates for every layer | Risk-based verification and explicit project requirements |
| Mandatory legacy Python checker | Native Rust validator for v2 policy; legacy checker remains for historical comparison |

## 🚀 Migration steps

1. **Record current behavior.** Preserve builds, public API guarantees, deployment configuration, known metrics and existing baselines.
2. **Audit old gates.** Remove unconditional failures based solely on line counts, module depth, naming blacklists, coverage percentages or mandatory DDD layers.
3. **Choose relevant profiles.** Pick architectural practices that address the actual domain, runtime, hardware, security and performance constraints.
4. **Create a v2 policy.** Use [the example](templates/craftbook.toml), with a project name and selected profiles. Do not copy v1 layer maps verbatim.
5. **Keep useful checks.** Continue to run rustfmt, rustc, Clippy, relevant tests and compatibility or security checks.
6. **Migrate incrementally.** Preserve useful aggregates, ports and adapters; rewrite only where a real deficiency has been identified. Include regression and contract tests for behavior changes.
7. **Document decisions.** Record significant API, transaction, dependency, data-format and performance decisions in ADRs or project documentation.
8. **Verify supported targets.** Test your promised feature combinations, target platforms, release profile and runtime behavior before retiring old checks.

Do not mechanically merge crates or flatten modules just because the original layout was designed around DDD.

## 🛠️ Tooling

The new experimental Rust utility validates the v2 configuration:

```bash
cargo install --path tools/cargo-craftcheck
cargo craftcheck check --config templates/craftbook.toml
```

It **does not yet validate Rust source architecture or Cargo dependency graphs**. The old `tools/craftcheck` remains available to compare existing v1 findings, but its output is not a mandatory gate for v2.

Verify the new utility independently:

```bash
cargo fmt --all --check --manifest-path tools/cargo-craftcheck/Cargo.toml
cargo clippy --all-targets --manifest-path tools/cargo-craftcheck/Cargo.toml -- -D warnings
cargo test --manifest-path tools/cargo-craftcheck/Cargo.toml
```

## 🔐 Compatibility

A migration is not complete merely because the policy file passes validation. Validate public contracts, serialization, persistence, platform behavior, error handling, resource cleanup and performance where applicable. Project-specific hard requirements remain enforceable when explicit and justified.

The legacy adoption reports are historical snapshots; remeasure before treating their metrics as current.

---

<div align="center">

[⬆ Back to top](#top) · [Craftbook](README.md) · [Русская версия](MIGRATION.ru.md)

</div>
