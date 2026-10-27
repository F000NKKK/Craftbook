# 6. Tooling

Rules that a machine can check are checked by a machine, so review time goes
to design. Three tools cover them:

| Tool | Checks | Config |
|---|---|---|
| `rustfmt` | formatting | [`templates/rustfmt.toml`](../templates/rustfmt.toml) |
| `clippy` + rustc lints | function size, parameters, nesting, complexity, `unwrap`/`panic`, `#[allow]`, docs on public items, `unsafe` comments, wildcard imports, `mod.rs` style | [`templates/clippy.toml`](../templates/clippy.toml), [`templates/workspace-lints.toml`](../templates/workspace-lints.toml) |
| `craftcheck` | everything clippy cannot see: file and module size, wiring files, one primary item per file, module and type naming, comments, global state, layers and the context map | [`tools/craftcheck`](../tools/craftcheck), the project's `craftbook.toml` |

## 6.1 rustfmt

- `cargo fmt --all --check` is a CI gate. Nobody formats by hand.
- `templates/rustfmt.toml` uses stable options only. The nightly-only options
  listed (commented) at its bottom are passed on the command line by the
  `fmt-nightly` job of [`templates/ci.yml`](../templates/ci.yml).

## 6.2 clippy and rustc lints

- The workspace root `Cargo.toml` contains the `[workspace.lints]` block from
  `templates/workspace-lints.toml`; every member has `[lints] workspace = true`
  (`craftcheck deps` reports members that do not).
- `clippy.toml` at the root sets the thresholds of chapter 2.
- CI runs `cargo clippy --workspace --all-targets --all-features -- -D warnings`.
- `#[allow(…)]` is denied (`clippy::allow_attributes`). The only way to
  silence a lint is `#[expect(lint, reason = "…")]` on the smallest item,
  with a reason that names the constraint. When the lint stops firing,
  clippy warns that the expectation is unfulfilled, and CI's `-D warnings`
  turns that into a failure, so stale suppressions cannot accumulate. The one
  crate-level exception is `#![expect(unsafe_code, reason = "…")]` in a crate
  whose job is unsafe (chapter 4 §4.6).
- `println!`/`eprintln!` are denied everywhere. A CLI binary prints its
  output with `writeln!(std::io::stdout().lock(), …)?`, which also surfaces
  a closed pipe as an error instead of a panic; libraries log.
- `clippy::pedantic` is on as a group; a pedantic lint that a project finds
  wrong is turned off in Craftbook by PR, for everyone, not locally.

## 6.3 craftcheck

```sh
python3 path/to/craftbook/tools/craftcheck check .            # file/module/naming/comment rules
python3 path/to/craftbook/tools/craftcheck deps .             # layers and context map
python3 path/to/craftbook/tools/craftcheck check . --journal docs/standards-review.tsv
python3 path/to/craftbook/tools/craftcheck crates . --layer domain   # crate names of a layer, for CI
```

Every command takes `--config FILE` to use another `craftbook.toml`.

Python 3.11+ and no third-party packages. Exit status is 1 when it finds an
error that is not in the baseline.

| Code | Rule | Level |
|---|---|---|
| C101 | production lines in a file over 400 (warning over 250) | error |
| C102 | inline test module over 150 lines | error |
| C103 | logic, or more than 100 lines, in the `lib.rs`/`mod.rs` of a split module | error |
| C104 | more than 12 files in one module directory | error |
| C105 | module deeper than 4 below `src/` | error |
| C106 | module named by kind (`utils`, `helpers`, `common`, `types`, …) | error |
| C108 | public types other than the file's primary item and its companions (`<Primary>Error`, `<Primary>Builder`, …) | warning |
| C109 | module directory that holds only `mod.rs` | warning |
| N201 | module path stutters (`audio::audio_streaming`) | error |
| N202 | type name with a noise word (`Manager`, `Helper`, `Data`, `Info`, …) | error |
| N203 | `get_` prefix outside the standard keyed accessors | error |
| D301 | file without a `//!` doc | error |
| D302 | `//!` doc longer than 30 lines | warning |
| D303 | `TODO`/`FIXME`/`XXX`/`HACK` in a comment without `(#issue)` | error |
| D304 | commented-out code | error |
| D305 | banner/separator comments | warning |
| E403 | `let _ = call(…)` that may swallow an error | warning |
| E404 | `anyhow` in a public signature outside the interface layer | error |
| A501 | global mutable state (`static mut`, `static` lock/cell/once) | error |
| A502 | field-less struct used only as a namespace for functions | error |
| S701 | `format!` that builds an id or path (`"{a}:{b}"`, `"{a}/{b}"`) | warning |
| S702 | `match` on string literals outside a parser (`FromStr`, `parse*`) | warning |
| L601 | crate missing from the layer map | warning |
| L602 | dependency that points outwards (domain → infra, …) | error |
| L603 | dependency across contexts with no edge in the context map | error |
| L604 | shared-kernel/domain crate depending on an I/O crate (`tokio`, `wgpu`, …) | error |
| L605 | `std::fs`/`process`/`net`/`env`, `std::thread::sleep`, `tokio`, `async fn` in shared-kernel/domain source | error |
| L606 | crate that does not inherit the workspace lint set | warning |
| L607 | workspace with no `[workspace.lints]` at all | warning |

Generated files (line 1 is `// @generated …`) and `*_table.rs` data files
are skipped. Anything else is exempted only through `craftbook.toml`, with a
reason and, preferably, the codes it covers.

### craftbook.toml

```toml
[limits]            # optional; may only tighten a limit (a higher value is rejected)
# file_lines = 300

[[crate]]
name = "meridian-physics-domain"
layer = "domain"    # shared-kernel | domain | app | infra | interface | mixed (before a split)
context = "physics"
profile = "perf"    # optional: performance kernel, ch. 7 §7.8 (turns off C108 and A502)

[[context_edge]]
from = "physics"
to = "geometry"
relation = "customer-supplier"   # shared-kernel | customer-supplier | acl | published-language

[[exempt]]
path = "crates/meridian-foundation/src/logging.rs"
reason = "process logger: the one allowed global (ch. 4 §4.7)"
codes = ["A501"]
```

### Adopting on an existing codebase

1. Run `craftcheck check . --write-baseline .craftcheck-baseline` and the same
   for `deps` into `.craftcheck-deps-baseline`; commit both.
2. CI runs with `--baseline`: existing findings are tolerated, new ones fail.
3. Each migration PR regenerates the baseline after its fixes. The baseline
   stores a count per file and rule, without line numbers, so unrelated edits
   do not churn it; it only shrinks, and a PR that raises a count is rejected.
4. `--journal docs/standards-review.tsv` keeps a per-file review status
   (`todo`/`ok`) for the manual part of review: a file is `ok` once read
   against chapters 1–8 and fixed, and returns to `todo` the moment it has a
   finding.

## 6.4 Other tools

| Tool | Gate |
|---|---|
| `cargo llvm-cov` | per-layer coverage thresholds (chapter 8) |
| `cargo mutants` + `tools/mutation_score.py` | mutation score per crate on shared-kernel/domain/app crates, nightly |
| `cargo deny` with [`templates/deny.toml`](../templates/deny.toml) | licences, advisories, banned and duplicate crates |
| `cargo doc` with `RUSTDOCFLAGS="-D warnings"` | broken intra-doc links |
| `cargo +nightly miri test` | crates with `unsafe`, nightly |
| `cargo fuzz` | decoders and parsers of untrusted input, nightly |
| `criterion` | benchmarks of hot paths, nightly, > 5 % regression fails |

A ready CI workflow is in [`templates/ci.yml`](../templates/ci.yml).
