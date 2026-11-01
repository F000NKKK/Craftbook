# 6. Tooling and validation

The default loop is `cargo fmt --all --check`, `cargo clippy --workspace --all-targets`, `cargo test --workspace` and `cargo doc --no-deps`, where the supported targets allow them. Do not assume `--all-features` works when features are mutually exclusive.

Use rustfmt for formatting and rustc/Clippy for Rust semantics. Curate warnings and use `-D warnings` after assessing existing warnings. `#[expect]` with an explanatory reason is useful; an unconditional ban on `#[allow]` is not universal.

Run Miri for relevant unsafe code, fuzzers for untrusted input, sanitizers where supported, and Loom/model checking for synchronization logic. Benchmark representative workloads, account for noise, and avoid universal regression thresholds.

`cargo deny` and advisory checks support project-specific license and supply-chain policy. Library versus application lockfile practices follow Cargo and deployment needs; `--locked` protects reproducible application builds.

Craftbook v2 uses only the native Rust `cargo-craftcheck` command. The legacy Python implementation is available in historical Git tags, not in `main`. Rust syntax is validated by rustc and Clippy; direct workspace dependency edges can be checked with Cargo metadata.

Example: `python3 tools/craftcheck_v2.py templates/CraftBook.toml`.

A hard CI gate needs a demonstrable correctness, security, compatibility or declared project-contract reason. Arbitrary line limits, naming blacklists and test percentages are advisory signals, not universal failures.

## Native Cargo subcommand

The experimental [`cargo-craftcheck`](../tools/cargo-craftcheck/README.md) implements v2 policy validation in Rust, independently of Python. Its automated tests are in the [v2 CI workflow](../.github/workflows/craftbook-v2.yml). The tool supports policy validation and explicit direct workspace dependency checks; source-level architecture analysis is not yet implemented.

### Dependency policies (experimental)

`cargo craftcheck deps --config CraftBook.toml --manifest-path Cargo.toml` checks optional `[[dependency_rule]]` entries against direct workspace dependencies resolved by Cargo Metadata. Each entry supplies `from` and `disallow` package names. No dependency direction is imposed unless the project specifies it. The current output is human-readable; it is not yet JSON or SARIF.

### Hierarchical `CraftBook.toml`

The native CLI finds the nearest ancestor configuration and scans down the selected directory tree. Subdirectories can contain their own `CraftBook.toml`; TOML tables merge, while scalars and lists override their inherited counterparts. Run `cargo craftcheck check` or `cargo craftcheck deps` without explicit file arguments. See the [native tool README](../tools/cargo-craftcheck/README.md).
