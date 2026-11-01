# cargo-craftcheck

Native Rust implementation of the Craftbook v2 policy validator. No Python runtime is required.

Install from this repository:

```sh
cargo install --path tools/cargo-craftcheck
cargo craftcheck check --config templates/CraftBook.toml
```

Run its tests: `cargo test --manifest-path tools/cargo-craftcheck/Cargo.toml`.

## Current scope

Validates schema, v2 version, nonempty project, supported profiles, unique profiles and boolean assurance/constraint options. Unknown keys fail validation. Does **not** inspect Rust source semantics. The optional `deps` command checks direct resolved workspace edges. The historical Python checker is available from the v1 and v2-rc0 Git tags; main contains only the Rust checker.

## Workspace dependency rules

Define forbidden **direct** edges between workspace package names in a v2 policy:

```toml
[[dependency_rule]]
from = "domain"
disallow = "infrastructure"
```

Run:

```sh
cargo craftcheck deps --config CraftBook.toml --manifest-path Cargo.toml
```

This uses `cargo metadata --format-version 1` and checks only direct resolved dependencies between members of the selected Cargo workspace. It deliberately does not infer DDD layers, prohibit external crates, or inspect the Rust AST. The rule is opt-in. The tool currently prints text diagnostics; JSON/SARIF are not yet supported.

## Hierarchical configuration

A project uses a case-sensitive `CraftBook.toml`. Place one next to `Cargo.toml` or in a parent directory. Nested configuration files may override inherited values: tables merge recursively; scalar values and arrays replace their parents. In particular, `[[dependency_rule]]` in a child replaces the inherited rule list.

From any directory containing `CraftBook.toml` (or below one):

```sh
cargo craftcheck check
cargo craftcheck deps
```

By default, commands start at the directory containing the nearest ancestor `CraftBook.toml`, descend recursively (excluding `.git`, `target`, `node_modules`, `.idea`, `.venv`), and process nested configurations. `deps` discovers Cargo manifests and resolves workspace dependencies through Cargo metadata. Use `--manifest-path` to limit dependency checks to one Cargo manifest. `--config PATH` selects another configuration root.

Note: package-specific policy is taken from the package's own directory; no DDD layering is implicit. Source-code semantics still belong to rustc and Clippy.

## Inspecting policies

Run `cargo craftcheck config` to print the validated effective TOML policy for the current working directory. Run `cargo craftcheck explain` to show the ordered source files and merged policy. Unlike `check` and `deps`, these inspection commands do not descend into sibling folders. `--config` selects a policy root; configuration is resolved for the current directory when it lies beneath that root.
