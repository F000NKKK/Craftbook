# Changelog

Release history is maintained only in this file.

## Unreleased

- CI publishes downloadable self-check artifacts containing actual JSON, SARIF and text reports, configuration details, negative-fixture diagnostics and command exit codes.
- Linux, Windows and macOS test jobs upload their execution logs, including on failed runs.

- CI self-check fixture verifies invalid-policy diagnostics in text, JSON and SARIF, including exact source coordinates.
- Text diagnostics include source line and column; syntax errors inherited from parent configuration point to their defining file.

- Canonical resolution of CLI configuration and manifest paths.
- Stable ordering of diagnostics and regression tests for path equivalence and repeatability.

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
