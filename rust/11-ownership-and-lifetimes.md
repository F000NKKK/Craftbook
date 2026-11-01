# 11. Ownership, borrowing and lifetimes

This is a decision guide, not a list of mandatory wrappers. The appropriate design minimizes accidental sharing while keeping data flow clear.

## Choose a representation

| Requirement | Starting point | Considerations |
|---|---|---|
| Sole owner | `T` | simplest lifecycle and cleanup |
| Read without owning | `&T` | lifetime does not outlive source |
| Exclusive temporary mutation | `&mut T` | no overlapping aliases |
| Heap indirection | `Box<T>` | indirection, recursive types and trait objects |
| Single-thread shared ownership | `Rc<T>` | not `Send`, reference cycles possible |
| Cross-thread shared ownership | `Arc<T>` | shared ownership is not shared mutation |
| Interior mutation, single thread | `Cell<T>` / `RefCell<T>` | runtime borrowing checks for `RefCell` |
| Concurrent mutable access | `Mutex<T>` / `RwLock<T>` | lock contention, poison and deadlock risks |
| Borrowed or owned input | `Cow<'a, T>` | suitable for copy-on-write data |
| Stable identity in collections | indexes, generation handles, arenas | address stability and invalidation rules |

## API design

Default to `&str` for reading text, `&[T]` for slices, `&Path` for borrowed paths and owned `String` / `Vec<T>` / `PathBuf` for stored values. The callee should own data when it must survive its source; do not retain borrowed references that artificially couple subsystems.

Avoid `Arc<Mutex<T>>` as a default container. First ask whether state can have a single owner receiving messages. When sharing is required, name the synchronization invariant and the scope of each lock.

Avoid `clone()` merely to satisfy the borrow checker: reconsider storage, scope and lifecycle. Conversely, cloning a small immutable value is often simpler than introducing a complex lifetime.

## Lifetimes are constraints, not memory managers

```rust
fn find_prefix<'a>(input: &'a str, prefix: &str) -> Option<&'a str> {
    input.strip_prefix(prefix)
}
```

The returned view is tied to `input`, not `prefix`. Prefer elision when the relationship is obvious. A struct holding references must not outlive its inputs; use owned values when data crosses asynchronous tasks or independent component boundaries.

## Self-referential and pinned structures

Avoid moving references to fields of the same owning struct into that struct. Where genuine address stability is needed, understand `Pin` and pin projection; `Pin` does not automatically pin pointees or guarantee that every type is immovable. Prefer stable handles/arenas or dedicated safe abstractions.

## Resource life cycle

Use RAII: a value owns a file descriptor, allocation, connection or lock guard, and releases it in `Drop`. Do not rely on `Drop` for asynchronous cleanup; provide explicit `close` or `shutdown` if completion and error reporting matter. Cancellation can drop futures at any suspension point; ensure partial work is safe.

## Verification

Test scope transitions, resource cleanup, double-close prevention, invalidated handles and concurrent access. Run Miri for relevant aliasing/unsafe cases and specialized tests for synchronization. Compiler acceptance is necessary, not a proof that high-level resource semantics are correct.
