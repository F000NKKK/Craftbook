# 4. Types, errors and safety

## 4.1 Types carry meaning

- **No primitive obsession.** An id is a newtype (`struct MeshId(u32)`), not
  a bare `u32`. A unit is a newtype (`Seconds`, `Radians`, `Pixels`). A closed
  set is an `enum`, not a `&str` or an integer. A rectangle is `Rect`, not
  `[f32; 4]`; a pair of meaning is a struct, not a tuple.
- **Make invalid states unrepresentable.** If two fields are only valid
  together, they are one `Option<Pair>`, not two `Option`s. A state machine
  is an `enum` with data per state, not a `state: u8` plus nullable fields.
- **Validate at construction.** A value object's constructor checks its
  invariant and returns `Result`; after that, the value is valid for its
  whole life, and no other code re-checks it. Fields are private.
- **Parse, don't validate.** Text from outside (files, CLI, network, UI) is
  parsed into a typed value once, at the boundary. Domain code never takes
  `&str` where it means a typed value.
- **One parser per format, one type per concept, workspace-wide.** Two
  crates with their own `Color` or `Vec3` is a defect: use the owner crate's
  type ([chapter 7](07-architecture-ddd.md)).

## 4.1a Strings are data, never structure

Data may be *stored* as text (a file, a column, a message on the wire). Code
never *works* with text that has meaning. All infrastructure — ids, keys,
paths, commands, events, messages, protocols, configuration, registries — is
typed end to end.

- **A string crosses exactly two points:** it is parsed into a typed value
  where it enters the process (`FromStr`/`TryFrom<&str>`/`serde` into a DTO,
  then into the domain type), and rendered back where it leaves
  (`Display`/`serde`). Between those points no function takes or returns a
  `String`/`&str` that stands for something else.
- **No string keys.** Maps, registries, event buses, command dispatch and
  lookups are keyed by enums, newtype ids or typed keys, not by
  `HashMap<String, _>` and `match name { "open" => … }`.
- **No string building of structure.** Ids, paths, prefixes and qualified
  names are composed by their type (`AssetId::new(namespace, path)`,
  `PathBuf::join`), never by `format!("{ns}:{path}")` or string
  concatenation; and they are taken apart by the type, never by `split(':')`
  at the call site.
- **No string literals naming a kind, a key, an extension, a prefix or a
  route.** Each such fact is one enum variant or one `const` of a typed value,
  defined once (the single source of truth), and every use goes through it.
- **Errors, events and messages are enums with typed fields,** not
  formatted text; text is produced at the edge for a person or a log
  (`Display`). Tests assert on variants, not on messages.
- **Text that really is free text** (a user's document, a commit message, a
  log line, a localized label) is still wrapped when it has a role:
  `CommitMessage(String)`, `LocalizedText`, so it cannot be passed where a
  path or an id is expected.
- **Dynamic, open-ended vocabularies** (plugins registering their own kinds)
  use a typed key minted by a registry (`KindId`), validated once at
  registration, not a string compared on every use.

Review rejects a `String`/`&str` parameter or field whose doc or name implies a
structure (`id`, `key`, `kind`, `path`, `name` of something registered,
`type`, `command`, `event`). The checker flags `format!` that builds a
`:`/`/`-joined id, and `match` on string literals outside parsers
(`craftcheck` S701, S702).

## 4.2 Derives and trait impls

Every public type implements, where it makes sense:

| Trait | When |
|---|---|
| `Debug` | always (`missing_debug_implementations` is denied) |
| `Clone`, `Copy` | value objects ≤ 16 bytes are `Copy`; others `Clone` unless they own a unique resource |
| `PartialEq`, `Eq`, `Hash` | value objects and ids: always; entities: by id only, written by hand |
| `PartialOrd`, `Ord` | when an order exists in the domain |
| `Default` | when an obvious empty/neutral value exists; never a "half-built" value |
| `Display` | errors, ids, anything shown to a user or a log |
| `From` / `TryFrom` | conversions; never `fn convert_x_to_y` |
| `Send`, `Sync` | public types are `Send + Sync` unless documented why not; a test asserts it |

`#[must_use]` on pure functions, builders, and types whose value matters
(`Result`-like, guards). `#[non_exhaustive]` on public enums and structs that
are expected to grow.

## 4.3 Visibility

- Default is private. Then `pub(super)`, `pub(crate)`, and `pub` only for
  the crate's API (`clippy::redundant_pub_crate` keeps it tidy).
- Struct fields are private, except for plain data carriers with no
  invariant (DTOs at the interface layer, POD types sent to the GPU marked
  `#[repr(C)]`).
- Everything `pub` is documented ([chapter 5](05-docs-and-comments.md)).

## 4.4 Errors

| Layer | Error type |
|---|---|
| Domain, application, infrastructure libraries | a typed `enum` per operation or per module: `DecodeError`, `ReserveError` |
| Binaries, examples, top-level hosts (`main`, Tauri command glue, CLI) | `anyhow::Result` allowed |

- Error enums are written with `thiserror` (or by hand with the same shape):
  `#[derive(Debug, thiserror::Error)]`, a `#[error("…")]` message per
  variant that says **what failed and with what input**, and `#[source]` for
  the cause. Messages are lower-case, no trailing period.
- `anyhow` never appears in a library's public signature.
- One error enum per operation family, not one god-error per crate.
  Variants a caller cannot act on differently are merged.
- Propagate with `?`. Convert with `From` impls or `map_err` at the layer
  boundary — infrastructure errors are translated into the port's error type,
  domain never sees `std::io::Error` or `wgpu::Error`.
- An error is **logged once**, at the boundary that decides it will not be
  propagated further, with context. Not at every level.
- **Swallowing is forbidden:** `let _ = fallible();`, `.ok();` dropping an
  error, `if let Ok(x)` with no `else` on a fallible operation. Handle,
  propagate, or log with a comment saying why it is safe to continue.

## 4.5 Panics

- `unwrap()` is forbidden outside tests (`clippy::unwrap_used`).
- `expect("…")` only on true invariants, with the invariant stated in the
  message: `.expect("pool index is in range: handle validated above")`.
- `panic!`, `unreachable!`, `todo!`, `unimplemented!` are not used in
  production code paths; `todo!` never reaches the default branch.
- Indexing (`v[i]`) is allowed where the index is proven in range by the
  surrounding code; otherwise `get(i)` and handle the `None`.
- No panic crosses an FFI or thread-callback boundary: catch with
  `std::panic::catch_unwind` there.

## 4.6 `unsafe`

- `#![forbid(unsafe_code)]` in every crate except those whose job is
  unsafe (drivers, FFI bindings, SIMD/GPU kernels), which say so in the crate
  doc.
- Each `unsafe` block is as small as possible and preceded by a
  `// SAFETY:` comment naming the invariant that makes it sound
  (`clippy::undocumented_unsafe_blocks`).
- Each `unsafe fn` documents its contract under a `# Safety` heading.
- Unsafe code has tests under Miri (`cargo +nightly miri test`) where the
  crate allows it.

## 4.7 State, globals and concurrency

- **No global mutable state.** No `static` `Mutex`/`RwLock`/`OnceLock`
  holding application state, no singleton "managers". State is owned by a
  value that is passed in. Process-wide read-only data (`LazyLock` of a
  constant table) is fine. The one allowed global is the process logger
  (`tracing`/`log` dispatcher), installed once by the composition root;
  the project lists it in `craftbook.toml` `[[exempt]]`.
- Shared ownership (`Arc`) is a decision, not a default. Prefer one owner and
  borrowing; reach for `Arc<Mutex<…>>` only at a real concurrency boundary,
  and document who else holds it.
- Locks are held for the shortest scope, never across `.await` or I/O.
- Interior mutability (`RefCell`, `Cell`) in domain types is a review
  finding unless the type documents why.

## 4.8 Async

- A function is `async` only when it performs genuine I/O: its completion is
  unbounded and decided outside the process (OS, device queue, file system,
  network). Allocation, validation, recording and math stay synchronous.
- Domain code is never `async`. Ports that do I/O may be `async` in the
  application layer.
- Blocking calls inside `async fn` go through
  `tokio::task::spawn_blocking`.
- Only crates with genuine I/O depend on `tokio`.

## 4.9 Dependencies

- A dependency is added to the crate that uses it, not to a shared "common"
  crate for convenience. Versions are declared once in
  `[workspace.dependencies]`.
- A new third-party dependency is justified in the PR (what it replaces,
  maintenance status, licence). `cargo deny check` passes.
- Optional subsystems are cargo features with names of their own, not
  `cfg(not(…))` forests.
