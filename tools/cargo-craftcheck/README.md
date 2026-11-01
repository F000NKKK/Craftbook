# cargo-craftcheck

Native Rust implementation of the Craftbook v2 policy validator. No Python runtime is required.

Install from this repository:

```sh
cargo install --path tools/cargo-craftcheck
cargo craftcheck check --config templates/craftbook.toml
```

Run its tests: `cargo test --manifest-path tools/cargo-craftcheck/Cargo.toml`.

## Current scope

Validates schema, v2 version, nonempty project, supported profiles, unique profiles and boolean assurance/constraint options. Unknown keys fail validation. Does **not** inspect Rust source semantics. The optional `deps` command checks direct resolved workspace edges. The old Python `tools/craftcheck` uses v1 rules and is not a replacement for this tool.

## Workspace dependency rules

Define forbidden **direct** edges between workspace package names in a v2 policy:

```toml
[[dependency_rule]]
from = "domain"
disallow = "infrastructure"
```

Run:

```sh
cargo craftcheck deps --config craftbook.toml --manifest-path Cargo.toml
```

This uses `cargo metadata --format-version 1` and checks only direct resolved dependencies between members of the selected Cargo workspace. It deliberately does not infer DDD layers, prohibit external crates, or inspect the Rust AST. The rule is opt-in. The tool currently prints text diagnostics; JSON/SARIF are not yet supported.
