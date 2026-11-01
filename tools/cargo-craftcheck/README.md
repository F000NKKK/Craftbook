# cargo-craftcheck

Native Rust implementation of the Craftbook v2 policy validator. No Python runtime is required.

Install from this repository:

```sh
cargo install --path tools/cargo-craftcheck
cargo craftcheck check --config templates/craftbook.toml
```

Run its tests: `cargo test --manifest-path tools/cargo-craftcheck/Cargo.toml`.

## Current scope

Validates schema, v2 version, nonempty project, supported profiles, unique profiles and boolean assurance/constraint options. Unknown keys fail validation. Does **not** yet analyze source semantics or cargo dependency graphs. The old Python `tools/craftcheck` uses v1 rules and is not a replacement for this tool.
