# 8. Testing and assurance

Test behavior, contracts and failure modes rather than chasing a universal coverage threshold. Use deterministic unit tests; integration tests for real boundaries; contract tests when implementations share semantics; end-to-end tests for critical flows; and property tests for stable invariants.

Use fuzzing for untrusted parsers, Miri for relevant unsafe patterns, model checking or Loom for concurrency algorithms and supported sanitizers for memory/thread bugs. No one technique proves correctness or soundness.

Regressions SHOULD receive reproducing tests where feasible. Verify cancellation, race conditions, timeouts, persistence failures and resource cleanup when applicable. Fakes should reproduce required semantics rather than merely verify implementation call order.

Coverage and mutation testing are diagnostic evidence. Project policy MAY set justified numerical targets, with maintained baselines and exemptions for unavailable hardware or targets. Never enforce 95% coverage on every type of Rust software.

Performance regression gates require noise-aware, representative, comparable benchmark environments. CI should cover supported targets and valid feature combinations, not indiscriminately `--all-features`.
