# Migrating from Craftbook v1 to v2

Craftbook v2 changes policy semantics, not just formatting.

1. Preserve existing projects and their baselines; do not automatically split or merge crates.
2. Stop using old numeric limits (file/function/field counts, coverage percentages) as mandatory gates.
3. Remove the assumption that every project has a domain/application/infrastructure/interface hierarchy.
4. Review `craftbook.toml` manually and select relevant profiles. See `templates/craftbook.toml`.
5. Keep `cargo fmt`, tests and carefully scoped Clippy rules; assess other gates against actual project risk.
6. Keep legacy `tools/craftcheck` only to compare historical results until a project migrates. Do not make its v1 findings mandatory for v2.
7. Retain useful DDD models, ports and boundaries where domain complexity warrants them.
8. Record externally observable behavior before a large rewrite; migrate with regression, contract and performance tests.
9. Record migrations in an ADR if they change public API, build profile, storage or deployment behavior.

## Old assumptions removed

One public type per file; `lib.rs`/ `mod.rs` as re-export-only files;
one context per crate; every I/O call through a port; fixed line/module-depth
and nesting limits; unconditional `Send + Sync`; all public types implementing
`Clone`/`Debug`; global prohibitions on ordinary names and text keys;
required 90%+ coverage and nightly mutation score in every project.

These can still be explicit PROJECT constraints when technically justified.
