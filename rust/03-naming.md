# 3. Naming and API design

Follow [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/). Crates use kebab-case package names; source paths, functions and fields use snake_case; types and traits use UpperCamelCase; constants use SCREAMING_SNAKE_CASE.

Name things according to the local domain or library vocabulary. Prefer terms recognizable to users. `Manager`, `Context`, `Service`, `Model`, `Wrapper`, `types` and `utils` are not intrinsically invalid; rename only when the name obscures responsibility.

Avoid redundant type suffixes and C# interface prefixes (`IRepository`), unless required for interop. Use `new`, `try_new`, `from`/`try_from`, `as_`/`to_`/`into_` according to Rust conversion semantics. The `get_` prefix is valid for conventional lookup operations (for example `HashMap::get`); it is not inherently wrong.

Public APIs should document important invariants, errors, panics, safety and ownership. Prefer typed IDs and units when mixing primitives would be dangerous. Do not wrap every string or scalar mechanically: free text, dynamic keys and protocols can legitimately use strings.

Avoid exposing implementation dependencies unnecessarily. Use semver and capability-oriented APIs; plan compatibility of enums, traits and public struct fields. Name modules for concepts, not for forced file-count heuristics.
