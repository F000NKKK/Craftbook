# crate-name

One sentence: what the crate is for (the same as `description` in `Cargo.toml`).

**Layer and context:** domain · physics

- **For:** what it owns, in two or three bullets.
- **Not for:** what people would look for here but is owned by `neighbour-crate`.

## Usage

```rust
//! The example is the crate's doc test: `#![doc = include_str!("../README.md")]`.
use crate_name::Thing;

let thing = Thing::new();
```

## Features

| Feature | Enables |
|---|---|
| `fixed` | The deterministic fixed-point flavor |

## Depends on / used by

- Depends on: [`meridian-gac`](../meridian-gac)
- Used by: [`meridian-physics-compute`](../meridian-physics-compute)

## License

MPL-2.0, see the repository [LICENSE](../../LICENSE).
