# 1. Modules, crates and workspaces

**MUST:** expose coherent APIs, avoid accidental visibility, and keep build targets and features correct. There is no universal one-type-per-file rule.

## Module boundaries

A Rust module is a privacy and namespace boundary. Group things that change together and share invariants: several enums, structs, helpers and implementations may belong in one file. Use `mod foo;` with `foo.rs` or `foo/mod.rs` consistently where useful; both are valid.

Prefer private implementation modules with a deliberate public surface. `pub(crate)`, `pub(super)` and `pub` communicate progressively broader contracts. Re-export when it improves discovery; do not flatten useful namespaces by default.

Split files when separate concepts, ownership responsibilities or review audiences emerge. Size can be a **signal**, never a universal correctness gate. A large cohesive parser is often clearer than a maze of tiny files.

## Crates

Create another crate for an independently reusable API, compile-time dependency isolation, feature isolation, deployment target or meaningful stability boundary. Do not create a crate for each DDD layer or type by default. Cyclic crate dependencies are invalid; use explicit dependency inversion or move a genuinely shared contract.

Use features for optional capabilities, not mutually exclusive configuration unless enforced. Treat feature unification and additive features as a compatibility concern. `no_std`, `alloc` and target-specific dependencies are project choices.

## Organization examples

- Library: `src/lib.rs`, grouped public modules, private implementation modules, `tests/`.
- CLI: `src/main.rs` plus modules or a library when logic needs reuse.
- Server: transport/use cases/domain/adapters *when warranted*; not required for CRUD.
- Engine: asset, scheduling, renderer, ECS, platform; prefer data-layout boundaries in hot paths.
- Embedded: hardware abstraction, control logic, protocols, interrupt boundary; minimize allocation where needed.

Tests can sit beside implementation (`#[cfg(test)]`) or in integration tests. Generated files, bindings and macro-heavy code need not obey arbitrary line budgets.
