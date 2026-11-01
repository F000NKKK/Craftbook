# 4. Types, errors, resources and safety

## Correctness

Represent invalid states with `enum`, `Option`, `Result` or validated newtypes when this reduces failure modes. Keep invariants localized, but distinguish validated domain values from untrusted wire representations. Avoid encoding open-ended dynamic vocabularies as fixed enums.

`Default`, `Copy`, `Clone`, `Eq`, `Hash`, `Send` and `Sync` are semantic capabilities, **not universal requirements**. Derive only when meaningful. `f32` normally does not implement `Eq`; many handles cannot be cloned safely.

## Errors and panics

Use `Result<T,E>` for recoverable failure; use error enums or opaque errors according to caller needs. `thiserror` helps library error types; `anyhow` fits application boundaries and can be valid inside libraries with intentionally opaque errors. Preserve source information and avoid logging the same failure at every layer.

`unwrap`/`expect` are acceptable when the invariant is demonstrably maintained and failure means a programmer error; avoid panics on untrusted data. Never silently swallow an error without a documented reason. Explicitly describe panic and cancellation semantics for public operations.

## Ownership and concurrency

Prefer single ownership, borrowing and scoped mutation. `Box` for indirection, `Rc` for single-thread sharing, `Arc` for cross-thread sharing, locks and atomics for synchronization, arenas and generational IDs when stable identity is needed. Interior mutability can be appropriate for caches, graphs and adapters; document invariants.

Global state is not categorically prohibited: immutable constants, process registries, initialization locks and thread-local state have valid uses. Minimize hidden coupling, initialization races and test interference.

## Unsafe, FFI, low-level code

Every unsafe operation MUST have a soundness argument: pointer validity, aliasing, initialization, lifetimes, alignment, synchronization and provenance as relevant. Document `# Safety` contracts for public unsafe functions and `SAFETY:` reasoning for unsafe blocks. `unsafe` does not relax compiler checks elsewhere.

Use RAII to manage handles, buffers, locks and FFI resources. Ensure cleanup works with early return, unwind and cancellation. `Pin`, `MaybeUninit`, atomics and lock-free code require precise invariants. Miri and sanitizers cover subsets of bugs; neither proves full soundness.

## Target constraints

For `no_std`, embedded, kernel and real-time profiles, explicitly choose allocation, panic strategy, blocking behavior, interrupt safety and memory layout. `#[repr(C)]` guarantees a C-compatible field layout but not arbitrary binary serialization.
