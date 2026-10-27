# 3. Naming

Names come from the [ubiquitous language](07-architecture-ddd.md#71-ubiquitous-language)
of the bounded context: if the glossary says *Renderable*, the code says
`Renderable`, not `DrawItem`, `Drawable` or `RenderObj`. A name that is not in
the glossary and is not a generic programming term is added to the glossary
in the same PR, or renamed.

Casing follows [Rust API Guidelines C-CASE](https://rust-lang.github.io/api-guidelines/naming.html);
everything below is our stricter layer on top of it.

## 3.1 Casing

| Item | Case | Example |
|---|---|---|
| Crate (package) | `kebab-case` | `meridian-gac-core` |
| Crate as written in code | `snake_case` | `meridian_gac_core` |
| Module, file, directory | `snake_case`, singular noun | `render_graph`, `scene` |
| Type, trait, enum variant | `UpperCamelCase` | `RenderGraph`, `Light::Point` |
| Function, method, local, field | `snake_case` | `bake_vertices` |
| `const`, `static` | `SCREAMING_SNAKE_CASE` | `MAX_LIGHTS` |
| Generic type parameter | one capital or `UpperCamelCase` word | `T`, `F: Flavor`, `Storage` |
| Lifetime | short lower-case word | `'a`, `'frame`, `'src` |
| Feature flag | `kebab-case` | `fixed-point`, `hw-tests` |

**Acronyms are words** (`clippy::upper_case_acronyms`, aggressive mode). `GpuDevice`, `HttpClient`, `Ecs`, `Wgsl`,
`parse_json` — never `GPUDevice` or `parseJSON`. Two-letter acronyms too:
`Io`, `Ui`, `Id`.

## 3.2 Choosing words

- **Full words.** `position`, `velocity`, `index`, `buffer`, `message`. Not
  `pos`, `vel`, `idx`, `buf`, `msg`, `mgr`, `ctx`, `cfg`, `tmp`. Accepted
  short forms are limited to: `id`, `len`, `min`/`max`, `src`/`dst`, `io`,
  `ui`, `rgb`/`rgba`, and math symbols inside math code (`a`, `b`, `t`, `n`,
  `x`/`y`/`z`, `dt`) where they match the formula in the doc comment.
- **One word per concept, workspace-wide.** Pick `remove` or `delete`,
  `create` or `make`, `color` (American spelling is our standard) — and use
  it everywhere. The glossary records the choice.
- **No noise words.** `Manager`, `Handler`, `Helper`, `Util(s)`, `Data`,
  `Info`, `Object`, `Thing`, `Base`, `Impl`, `Processor`, `Wrapper` are
  rejected as type-name parts. They say "I did not decide what this is".
  `Handler` is allowed only for an actual callback registered with a
  dispatcher.
- **No type-names in names.** `name_string`, `items_vec`, `IRenderer`,
  `TFlavor` (Hungarian, C#-interface `I` prefix) are rejected.
- **No stutter.** The path already says the module: `scene::Camera`, not
  `scene::SceneCamera`; `audio::streaming`, not `audio::audio_streaming`;
  `git::status::read`, not `git::status::read_git_status`. `craftcheck`
  N201 catches file names that repeat their directory; type and function
  names that repeat their module are a review finding (clippy's
  `module_name_repetitions` only sees items under a `pub mod`).
- **Units in names when the type does not carry them.** `timeout_ms`,
  `delay_secs`, `width_px`, `angle_rad`. Better: a newtype (`Seconds(f32)`,
  `Radians(f32)`) and no suffix ([chapter 4](04-types-and-errors.md)).

## 3.3 Types

| Kind | Pattern | Examples |
|---|---|---|
| Entity / aggregate root | domain noun | `RigidBody`, `Solution`, `Workflow` |
| Value object | domain noun, often a measure | `Quantity`, `Rotor`, `Aabb`, `CommitHash` |
| Typed id | `<Entity>Id` | `EntityId`, `MeshId`, `SessionId` |
| Generational handle | `<Thing>Handle` | `TextureHandle` |
| Error | `<Operation or Thing>Error` | `DecodeError`, `ReserveError` |
| Error kind enum inside an error | `<Thing>ErrorKind` | `DecodeErrorKind` |
| Domain event | past-tense phrase | `BodyCollided`, `ProjectOpened` |
| Command (application layer input) | imperative phrase | `OpenProject`, `SpawnBody` |
| Port (trait in domain/application) | role noun | `Clock`, `MeshStore`, `ProcessRunner` |
| Adapter (implementation of a port) | technology + role | `WgpuMeshStore`, `SystemClock`, `GitCliRunner` |
| Fake for tests | `Fake<Role>` / `InMemory<Role>` | `FakeClock`, `InMemoryMeshStore` |
| Builder | `<Type>Builder` | `RenderGraphBuilder` |
| Options struct | `<Operation>Options` | `SearchOptions` |
| Marker / flavour type | adjective or noun | `Float`, `Fixed` |
| Trait for a capability | verb-able adjective or role noun | `Integrable`, `Decode` |

## 3.4 Functions and methods

| Kind | Pattern | Examples |
|---|---|---|
| Constructor | `new`, `with_<what>`, `from_<source>` | `Pool::with_capacity(n)` |
| Fallible constructor | `try_new`, `try_from_<source>`, or `TryFrom` | `Rotor::try_from_matrix(m)` |
| Getter | the field's noun, **no `get_`** | `body.mass()`, `graph.passes()` |
| Mutable getter | `<noun>_mut` | `body.velocity_mut()` |
| Setter | `set_<noun>` | `body.set_mass(m)` |
| Boolean query | `is_`, `has_`, `can_`, `should_` | `is_empty()`, `has_children()` |
| Cheap borrowed conversion | `as_<type>` | `as_slice()` |
| Expensive or owned conversion | `to_<type>` | `to_matrix()` |
| Consuming conversion | `into_<type>` | `into_inner()` |
| Lookup that may fail | `find_<thing>` → `Option` | `find_entity(id)` |
| Collection access by key | `get` / `get_mut` (std convention) | `pool.get(handle)` |
| Iterator | `iter`, `iter_mut`, `<plural noun>` | `bodies()` |
| Command | verb + object | `apply_impulse`, `submit`, `reserve` |
| Domain service | verb + object(s), in a concept module | `collision::intersect(a, b)` |
| Test | `<subject>_<condition>_<outcome>` sentence | `stale_handle_after_free_resolves_to_none` |

`get_` is reserved for the standard keyed access (`get`, `get_mut`,
`get_or_insert_with`). Anything else starting with `get_` is renamed.

## 3.5 Modules and crates

- Crate names: `<product>-<context>-<layer>` where layer is one of the
  [layer suffixes](07-architecture-ddd.md#73-layers): `meridian-physics-domain`,
  `meridian-physics-app`, `meridian-physics-wgpu`, `substrate-git-domain`,
  `substrate-git-gitcli`. Existing names are migrated per
  the project's adoption page.
- A module name is a concept, singular: `scene`, `collision`, `status`. Not
  a layer word (`models`, `services`, `managers`), not a kind
  (`types`, `traits`, `structs`), not a plural (`events` is allowed only when
  the module is a set of event types).
- Files are named after their primary item (§1.2 of chapter 1).

## 3.6 Variables

- Booleans read as a statement: `is_visible`, `needs_rebuild`, not
  `visible_flag` or `check`.
- Collections are plural: `bodies`, `lights`; a map is `<value>_by_<key>`:
  `mesh_by_id`.
- Indices and counts are explicit: `body_index`, `light_count`.
- Shadowing is fine for the same value in a new form
  (`let path = path.canonicalize()?;`), never for a different meaning.
- `_` prefixes only for intentionally unused bindings; a leading `_` on a used
  binding is rejected.
