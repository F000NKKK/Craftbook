# 6. Tooling and validation

The default loop is `cargo fmt --all --check`, `cargo clippy --workspace --all-targets`, `cargo test --workspace` and `cargo doc --no-deps`, where the supported targets allow them. Do not assume `--all-features` works when features are mutually exclusive.

Use rustfmt for formatting and rustc/Clippy for Rust semantics. Curate warnings and use `-D warnings` after assessing existing warnings. `#[expect]` with an explanatory reason is useful; an unconditional ban on `#[allow]` is not universal.

Run Miri for relevant unsafe code, fuzzers for untrusted input, sanitizers where supported, and Loom/model checking for synchronization logic. Benchmark representative workloads, account for noise, and avoid universal regression thresholds.

`cargo deny` and advisory checks support project-specific license and supply-chain policy. Library versus application lockfile practices follow Cargo and deployment needs; `--locked` protects reproducible application builds.

The historical `tools/craftcheck` enforces Craftbook v1 and is retained **only for migration**. It is not an authority for v2. The small `tools/craftcheck_v2.py` validates v2 project policy without pretending to parse Rust using regular expressions. Let rustc and Cargo validate source and dependencies.

Example: `python3 tools/craftcheck_v2.py templates/craftbook.toml`.

A hard CI gate needs a demonstrable correctness, security, compatibility or declared project-contract reason. Arbitrary line limits, naming blacklists and test percentages are advisory signals, not universal failures.
