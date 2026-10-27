<div align="center">

<a id="top"></a>

# 🧱 Product Name

### One line: what it is, for whom

[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE)
[![Rust 1.99](https://img.shields.io/badge/Rust-1.99-orange.svg)](rust-toolchain.toml)

🇺🇸 **English** | 🇷🇺 [Русский](README.ru.md)

[Overview](#-overview) • [Getting Started](#-getting-started) • [Architecture](#-architecture) • [Development](#-development) • [License](#-license)

</div>

---

## 📖 Overview

What it is, in a short paragraph. What it is not, if that prevents a wrong guess.

| Part | Where | What it gives you |
|---|---|---|
| `crate-a` | [`crates/crate-a`](crates/crate-a) | One sentence |

<p align="center"><img src="docs/images/overview.png" alt="What the picture shows" width="900"></p>

## 🚀 Getting Started

```toml
# Cargo.toml
[dependencies]
crate-a = "0.1"
```

```rust
// A minimal example that compiles; it is built in CI.
```

## 🏛️ Architecture

The layers and contexts in one picture ([Craftbook ch. 7](https://github.com/F000NKKK/Craftbook/blob/main/rust/07-architecture-ddd.md)),
then the crate table; the rest is in [`docs/`](docs).

## 🛠️ Development

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
```

Engineering standard: [Craftbook](https://github.com/F000NKKK/Craftbook).
Contributing: [CONTRIBUTING.md](CONTRIBUTING.md).

## 📄 License

One paragraph and a link to [LICENSE](LICENSE).

---

<div align="center">

**[⬆ Back to Top](#top)**

</div>
