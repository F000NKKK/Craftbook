# Craftbook — Rust engineering handbook

A pragmatic, project-neutral standard for Rust libraries, CLIs, services, embedded firmware, kernels, games, scientific computing, desktop applications and tooling.

**Principle:** correctness and explicit contracts are mandatory; architectural patterns are choices, never universal obligations. Rust is not C# with different syntax.

## Read in order

1. [Modules and crates](rust/01-files-and-modules.md)
2. [Functions, ownership and abstraction](rust/02-functions.md)
3. [Naming and API design](rust/03-naming.md)
4. [Types, errors, memory and safety](rust/04-types-and-errors.md)
5. [Documentation](rust/05-docs-and-comments.md)
6. [Tooling and checks](rust/06-tooling.md)
7. [Architecture and optional DDD](rust/07-architecture-ddd.md)
8. [Testing and quality](rust/08-testing-and-quality.md)
9. [Review checklist](rust/09-review-checklist.md)
10. [README conventions](rust/10-readmes.md)

## Policy model

- **MUST:** demonstrated safety, soundness, correctness, compatibility and explicitly declared project contracts.
- **SHOULD:** idiomatic defaults and maintainability guidance; deviations require engineering judgment, not approval bureaucracy.
- **MAY:** optional patterns and architectural profiles selected by the project.
- **PROJECT:** explicit requirements for criticality, platforms, stability, latency, security, CI and availability.

See [architecture profiles](rust/07-architecture-ddd.md) and [sample policy](templates/craftbook.toml). A profile can be combined with others; an embedded device is not required to implement repositories, and a DDD service is not required to split every type into its own crate.

## Adoption

Document goals, failure modes and chosen profiles in the project policy. Run `cargo fmt --all --check`, relevant Clippy checks and tests. The legacy `tools/craftcheck` remains available for migration only: **its historical opinionated checks are not the normative v2 standard**. Do not treat a green legacy report as evidence of architecture quality.

[Russian overview](README.ru.md) · [migration guide](MIGRATION.md)

## Scope and limitations

No standard can guarantee correctness, security, architecture quality or fit for *every* task. Craftbook instead supplies adaptable decisions and explicit verification; project-specific requirements take precedence when justified and documented.
