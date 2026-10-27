# 8. Testing, coverage and quality gates

## 8.1 Coverage thresholds

Coverage is measured with [`cargo-llvm-cov`](https://github.com/taiki-e/cargo-llvm-cov)
per crate, and the threshold depends on the crate's layer (from
`craftbook.toml`):

| Layer | Line coverage | Region coverage | Mutation score (`cargo-mutants`) |
|---|---|---|---|
| Shared kernel | ≥ 95 % | ≥ 90 % | ≥ 85 % |
| Domain | ≥ 90 % | ≥ 85 % | ≥ 80 % |
| Application | ≥ 85 % | ≥ 80 % | ≥ 70 % |
| Infrastructure (adapters) | ≥ 70 %, plus contract tests (§8.4) | — | — |
| Interface (CLI, Tauri, examples) | smoke tests per entry point | — | — |
| **New or changed lines in a PR** | **≥ 90 %** in kernel/domain/app crates | — | — |

- Coverage is a floor, not the goal. A test that executes code without
  asserting its result does not count in review, and `cargo-mutants` exists
  to catch exactly that: a surviving mutant in domain code is a missing
  assertion.
- Coverage is never raised by excluding code. `#[coverage(off)]` is allowed
  only on code that cannot run in CI (hardware-only paths) and each use is
  listed in the project's adoption page.
- Thresholds only go up. A project that is below them records its current
  numbers in its adoption page and must not lower them in any PR.

## 8.2 Test pyramid

| Level | What | Where | Speed |
|---|---|---|---|
| Unit | one value object, entity, function; pure, no I/O | next to the code (chapter 1 §1.6) | < 10 ms each |
| Property | invariants over generated inputs (`proptest`): value objects, math, parsers, round trips | unit test modules | < 1 s per property in CI |
| Use case | an application use case with fake ports | `tests/` of the app crate | < 100 ms each |
| Contract | one shared test suite that every adapter of a port must pass (real and fake) | `testing` module of the port's crate, run from each adapter crate | seconds |
| Integration | several crates wired as in production, real adapters where CI can run them | workspace `tests/` or the interface crate | seconds |
| Golden / snapshot | output formats, generated code, rendered images within tolerance | `tests/golden/` with `insta` or committed files | seconds |
| Fuzz | decoders and parsers of untrusted bytes (`cargo-fuzz`) | `fuzz/` | nightly job |
| Benchmark | hot paths and every performance kernel (`criterion`) | `benches/` | nightly job, regressions > 5 % fail |

Most tests are unit and property tests. If a domain behaviour can only be
tested through an integration test, the domain is leaking into an adapter.

## 8.3 How a test is written

- **Name says the behaviour:** `<subject>_<condition>_<outcome>`:
  `reserve_more_than_available_returns_insufficient_stock`.
- **Arrange, act, assert,** separated by a blank line; one behaviour per test.
  Several `assert!`s are fine when they check one outcome.
- **No logic in tests:** no loops or branches computing the expected value.
  Table-driven tests iterate over literal cases.
- **Deterministic:** no wall clock, no real randomness (seeded RNG, `Clock`
  port), no dependence on test order, no sleeps.
- **Floating point** compared with an explicit tolerance helper
  (`assert_approx_eq!(a, b, 1e-6)`), the tolerance justified once in the
  helper's doc.
- **Builders for fixtures:** `RigidBodyBuilder::default().mass(2.0).build()`;
  no 30-line struct literals repeated across tests.
- **Every bug fix adds a regression test** that fails before the fix.
- **Every public function has at least one test** that calls it, and every
  error variant is produced by at least one test.
- Errors are asserted by variant (`assert!(matches!(err, DecodeError::Truncated { .. }))`),
  not by message text.
- Tests never `#[ignore]` without an issue reference; a flaky test is fixed or
  deleted within the week, never retried in CI.

## 8.4 Ports, fakes and contract tests

For each port trait:

1. The port's crate ships a `testing` module (feature `testing`) with an
   in-memory fake and a **contract test suite**: a generic function
   `fn run_contract<S: MeshStore>(make: impl Fn() -> S)` that asserts the
   behaviour every implementation must have.
2. The fake runs the contract in its own tests.
3. Each real adapter runs the same contract (hardware adapters behind a
   `hw-tests` feature in CI jobs that have the hardware).

This is what makes fakes trustworthy: they are checked against the same
specification as the real thing.

Mocking libraries that verify call sequences (`mockall` expectations) are
used only to test that an adapter talks to an external API correctly; domain
and use-case tests use fakes and assert on state, not on calls.

## 8.5 Quality gates in CI

Every PR must pass, in this order (fast first):

1. `cargo fmt --all --check`
2. `cargo clippy --workspace --all-targets --all-features -- -D warnings`
   with the workspace lint set ([chapter 6](06-tooling.md))
3. `python3 craftcheck.py check .` — file, module and comment rules
4. `python3 craftcheck.py deps .` — layer and context-map dependency rules
5. `cargo test --workspace --all-features` (and `cargo test --doc`)
6. `cargo llvm-cov` with the per-layer thresholds
7. `cargo deny check` — licences, advisories, duplicate versions
8. `cargo doc --no-deps` with `RUSTDOCFLAGS="-D warnings"`

Nightly jobs: `cargo mutants` on kernel/domain/application crates, fuzzing,
benchmarks, Miri on crates with `unsafe`.

A PR is merged only green. No gate is disabled to merge a PR; a gate that is
wrong is fixed in Craftbook first.

## 8.6 Definition of done

A change is done when:

- [ ] it follows chapters 1–7, and `craftcheck` reports nothing new;
- [ ] tests for the new behaviour exist at the right level of the pyramid;
- [ ] coverage of changed lines is ≥ 90 % (kernel/domain/app);
- [ ] public items are documented, the glossary and context map are current;
- [ ] an ADR is written if the change makes an architectural decision;
- [ ] all CI gates are green.
