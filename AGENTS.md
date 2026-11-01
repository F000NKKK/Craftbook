# Craftbook contributor guidance

Craftbook is a Rust engineering handbook, not a DDD-only template.

- Correctness, soundness and documented project contracts outrank stylistic preferences.
- MUST / SHOULD / MAY / PROJECT are distinct. Do not turn a SHOULD into an unconditional CI failure.
- Rust standard library and ecosystem idioms outrank patterns borrowed mechanically from C#.
- Keep ten linked chapters, both README language versions, examples and migrations aligned.
- DDD, ports/adapters, CQRS and layers are optional profiles. Never force these on kernels, games or simple libraries.
- Changes to tooling must include tests and preserve an explainable migration path.
- The Python legacy checker is not authoritative for the v2 handbook.
