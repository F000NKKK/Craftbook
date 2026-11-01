# 13. Unsafe, FFI and low-level engineering

Unsafe Rust does not switch off memory rules. It delegates the proof of specific operations to the programmer. Keep the trusted computing base small and expose safe interfaces only when their invariants hold for all safe callers.

## Unsafe invariants

For each unsafe operation, describe applicable preconditions: allocation validity, alignment, initialization, provenance, aliasing, lifetime, synchronization, layout and unwind behavior. A `// SAFETY:` comment should explain *why those preconditions hold here*, not merely repeat a function name.

Unsafe functions require a `# Safety` contract stating obligations of callers. Avoid creating safe wrappers around unsafe functionality when correctness depends on undocumented caller discipline.

## FFI

Use explicit ABI, ownership and lifetime contracts. Specify who allocates/frees data, how null pointers and lengths are represented, whether calls are thread-safe, and how callbacks are registered and revoked. Do not unwind through a foreign ABI that does not support unwinding. Use `#[repr(C)]` only for C-compatible layout requirements; it is not a portable persistent-file format.

For opaque foreign handles, wrap them in a unique owning struct with `Drop`, avoid duplicate frees, and make raw constructors unsafe when callers must uphold invariants. Handle foreign libraries that use thread-affine or non-reentrant APIs.

## Systems profiles

A kernel or embedded project may require `no_std`, custom allocators, interrupt masking, atomics with correct orderings, DMA memory barriers, volatile MMIO access and explicit panic handlers. Each has hardware-specific rules and cannot be made correct by generic formatting or line-count constraints.

## Verification

Use Miri for supported unsafe Rust semantics; fuzz untrusted parsers; run sanitizers where available; use hardware tests for MMIO, DMA and interrupt code. Review platform-specific memory models. No tool proves all unsafe code sound.
