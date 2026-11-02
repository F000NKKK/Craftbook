# Changelog

Release history is maintained only in this file.

## Unreleased

- Audited CLI subcommands, option errors and exit codes with integration regression coverage.
- Added a structured-diagnostic regression for malformed Cargo manifests (CB003) without a process panic.
- Documented current CLI, JSON schema v1, SARIF 2.1.0, coordinates and output guarantees.

- Added regression checks for the stable JSON/SARIF diagnostic schema and source provenance.
- Extended repository CI assertions for successful JSON/SARIF reports.

## v2-rc5 — 2026-11-02

- CI template includes an optional `cargo-craftcheck` policy/dependency job with downloadable JSON and SARIF reports.
- Policy template documents nested overrides and opt-in dependency rules; repository CI validates shipped templates.

- Self-check analyzes repository configurations only; invalid-input fixtures remain in automated tests.
- Effective configuration output is valid TOML, verified by an integration test and repository CI self-check.
- Downloadable self-check artifacts contain actual repository policy/dependency reports, effective configuration, provenance and exit codes.
- Separate Linux, Windows and macOS jobs retain their own test logs.
- Text diagnostics include source line and column; inherited TOML syntax failures point to their original file.
- CLI configuration and manifest paths are canonicalized; diagnostic ordering is deterministic.

## v2-rc4 — 2026-11-02

- Parser-backed coordinates for TOML syntax, policy type, structure and semantic errors.
- JSON and SARIF source ranges for policy declarations and Cargo workspace dependencies.
- Improved handling of inherited policies, aliases, CRLF and Unicode.

## v2-rc3 — 2026-11-01

- Native Cargo policy checker with scoped dependency validation and structured diagnostics.
- Regression coverage of nested workspaces and policy inheritance.

## v2-rc2 — 2026-11-01

- Tracked the originating manifest and policy rule index in dependency violations.
- Added nested-workspace and inherited-rule validation coverage.

## v2-rc1 — 2026-11-01

- Native Rust policy checker with hierarchical configuration and source-aware dependency checks.
- JSON/SARIF reporting and integration tests.

## v2-rc0 — 2026-11-01

- Introduced the adaptable Craftbook v2 handbook and its Rust policy checker.
- Included migration documentation alongside the earlier Python implementation.

## v1 — 2026-11-01

- Legacy prescriptive Rust engineering standard and Python checker.
- Rules covering source layout, dependencies, comments and other project conventions.
