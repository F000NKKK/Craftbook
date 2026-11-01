# 5. Documentation and comments

Document observable behavior, invariants, safety requirements, error classes, ownership, cancellation, concurrency expectations, supported targets and feature constraints. Rustdoc examples should compile where feasible; use doctests for core public workflows.

Write comments for reasons, non-obvious tradeoffs and proofs; do not narrate syntax. Module-level `//!` docs are valuable on public modules and important subsystems but need not appear in every implementation file.

Write `# Safety` for public unsafe APIs, `# Panics` where surprising or relevant, and `# Errors` when errors matter to the caller. Track TODOs in issues where suitable; a small internal TODO is not automatically a build failure.

Keep architecture decision records for irreversible boundaries: public API guarantees, concurrency models, persistence formats, dependency directions and security-sensitive decisions. Documentation may be English, Russian or both according to project policy. Do not require emojis or a prescribed README layout.
