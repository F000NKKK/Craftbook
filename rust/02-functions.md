# 2. Functions, ownership and abstractions

## Choosing a function

Prefer a method when behavior uses a value's state or invariant; an associated function for construction; a free function for symmetric or module-level operations; a trait for a useful behavioral contract or generic/dynamic polymorphism. No minimum implementation count is required for a sound abstraction.

Prefer `&T` for borrowing, `&mut T` for exclusive mutation, `T` for transfer or cheap `Copy`, and `Arc<T>` only for real shared ownership. Avoid cloning merely to silence borrow errors. Prefer returning owned data when a reference would force awkward lifetime coupling.

Use `impl Trait` / generics for static dispatch where useful, `dyn Trait` for runtime heterogeneity, `enum` for closed variants and closures for behavior passed locally. Measure monomorphization and dynamic dispatch in performance-sensitive code.

Functions should have clear responsibilities. Early returns, `?`, `match`, iterators and named helper functions are tools, not mandates. An arbitrary 25- or 50-line function ceiling can incentivize harmful fragmentation.

## State transitions

`enum` with variant-specific data prevents invalid combinations. Typestate (`Connection<Disconnected>` -> `Connection<Connected>`) is worthwhile for stable state machines, protocol APIs and resource lifecycle; runtime enums are preferable when states are naturally dynamic or stored in collections.

Do not force command/query separation on standard mutating collection operations that reasonably return a previous value. `bool` arguments are acceptable when meaning is obvious, while a named enum improves ambiguous public APIs.

## Async

Async is a concurrency model, not an architecture layer. It may express I/O, streaming and async coordination. Avoid blocking an executor; consider cancellation, timeouts, backpressure, bounded queues, `Send` requirements and structured task lifetimes. Lock scopes must not cross await unintentionally.

## Traits

Design object-safe traits only when dynamic dispatch is required. Prefer associated types when they clarify relationships. Public trait evolution can be breaking; seal when extension by downstream crates is not part of the contract.
