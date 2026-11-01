# Craftbook contributor guidance

Craftbook is a Rust engineering handbook, not a DDD-only template.

- Correctness, soundness and documented project contracts outrank stylistic preferences.
- MUST / SHOULD / MAY / PROJECT are distinct. Do not turn a SHOULD into an unconditional CI failure.
- Rust standard library and ecosystem idioms outrank patterns borrowed mechanically from C#.
- Keep fourteen linked chapters, both README language versions, examples and migrations aligned.
- DDD, ports/adapters, CQRS and layers are optional profiles. Never force these on kernels, games or simple libraries.
- Changes to tooling must include tests and preserve an explainable migration path.
- The only supported checker is native Rust `cargo-craftcheck`. Maintain the root and nested `CraftBook.toml` files and require self-check CI.
