# cargo-craftcheck

A native Rust checker for Craftbook project policies and Cargo workspace dependency boundaries. No Python runtime is required.

## Installation

From the repository root:

```sh
cargo install --path tools/cargo-craftcheck
```

## Commands

```sh
cargo craftcheck check
cargo craftcheck deps
cargo craftcheck config
cargo craftcheck explain

cargo craftcheck check --format json
cargo craftcheck deps --format sarif
cargo craftcheck deps --manifest-path Cargo.toml
cargo craftcheck check --config CraftBook.toml
```

| Command | Behavior |
| --- | --- |
| `check` | Validate effective policies for all discovered `CraftBook.toml` files |
| `deps` | Check opt-in forbidden direct dependencies between Cargo workspace members |
| `config` | Print the effective policy for the current directory |
| `explain` | Show the policy's source files in inheritance order and the resulting configuration |

The default command is `check`. `--format text|json|sarif` is available for `check` and `deps`. A violation results in a nonzero exit code.

## Configuration

The configuration file must be named `CraftBook.toml` (case-sensitive). Place it beside a `Cargo.toml` or in a parent directory. The checker finds the nearest ancestor configuration and recursively inspects the corresponding subtree. Directories such as `.git`, `target`, `node_modules`, `.idea`, and `.venv` are excluded.

Child configurations inherit parent settings. Tables merge recursively; scalar and array values replace inherited values. In particular, a child `dependency_rule` array replaces the entire parent array.

See [the policy template](../../templates/CraftBook.toml) and [the project's own policy](../../CraftBook.toml).

### Dependency boundaries

Define forbidden direct edges by Cargo package name:

```toml
[[dependency_rule]]
from = "domain"
disallow = "infrastructure"
```

`deps` uses `cargo metadata` and evaluates the effective policy of each workspace member. It scans each distinct Cargo workspace once, even when multiple member manifests are discovered. Independent nested workspaces are analyzed separately. External registry dependencies and transitive edges are not prohibited by these rules.

## Diagnostics

`check` and `deps` emit human-readable text by default. Structured output supports JSON and SARIF 2.1.0.

| ID | Meaning |
| --- | --- |
| `CB001` | Invalid policy |
| `CB002` | Forbidden direct workspace dependency |
| `CB003` | Cargo workspace inspection failed |

For `CB002`, JSON includes the package manifest path, `config_path` and zero-based `rule_index`. When source positions can be determined, it also includes one-based `line`, `column`, and `config_line`. SARIF supplies a primary location in the package's `Cargo.toml`, a related location in the governing `CraftBook.toml`, and `properties.configPath` / `properties.ruleIndex`.

SARIF file URIs are relative to the selected project root where possible. Locations without a reliable source position omit the `region` field rather than reporting a guessed position. No upload to GitHub Code Scanning is performed by the checker itself.

## Scope and limitations

The tool validates policies and explicitly configured direct Cargo workspace dependency boundaries. It does not parse Rust syntax, enforce universal architectural layers, or replace rustc, Clippy, rustfmt, tests, or code review.

## Development and self-check

```sh
cargo fmt --all --check --manifest-path tools/cargo-craftcheck/Cargo.toml
cargo clippy --all-targets --manifest-path tools/cargo-craftcheck/Cargo.toml -- -D warnings
cargo test --manifest-path tools/cargo-craftcheck/Cargo.toml
cargo run --manifest-path tools/cargo-craftcheck/Cargo.toml -- check
cargo run --manifest-path tools/cargo-craftcheck/Cargo.toml -- deps
```

The root and crate-local `CraftBook.toml` files are part of the project's own CI checks.
