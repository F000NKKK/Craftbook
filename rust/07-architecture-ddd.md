# 7. Architecture: domain-driven design

We use DDD in both of its halves. **Strategic** design decides how the system
is cut into crates and who may depend on whom; it applies to everything.
**Tactical** design (entities, value objects, aggregates, ports) decides how
code inside a domain is shaped; it applies to domain and application code,
with one documented exception for performance kernels (§7.8).

## 7.1 Ubiquitous language

Every bounded context has a glossary section in `docs/glossary.md`:

```markdown
## Physics

| Term | Meaning | In code |
|---|---|---|
| Rigid body | A body whose shape does not deform; has mass, pose and velocity. | `RigidBody` |
| Contact | A pair of bodies whose shapes overlap this step, with normal and depth. | `Contact` |
| Step | One fixed-length advance of the simulation. | `World::step` |
```

- The code uses the glossary's words, the glossary uses the code's words.
  Renaming either renames both, in one PR.
- One term, one meaning, inside a context. The same word may mean different
  things in two contexts (a *mesh* in graphics vs. physics); then each context
  has its own type, and a translation at the boundary (§7.6).
- A term that appears in more than one context with the same meaning belongs
  to a **shared kernel** crate.

## 7.2 Bounded contexts and the context map

- A **bounded context** is a part of the domain with its own model and
  language: *physics*, *rendering*, *audio*, *assets*, *source control*,
  *search*, *terminal*.
- Each bounded context is **one domain crate** (plus its application and
  adapter crates). A crate never holds two contexts; a context never spreads
  over two domain crates.
- `docs/context-map.md` lists the contexts and the relation between each pair
  that talks: **shared kernel** (both depend on a small common crate),
  **customer–supplier** (downstream depends on upstream's published API),
  **anticorruption layer** (downstream translates upstream's model into its
  own), **published language** (a documented data format both sides speak).
  Every context may use shared-kernel crates; any other dependency across
  contexts needs a listed relation, or it is not allowed.

## 7.3 Layers

Inside each context the dependency rule points inwards: outer layers depend on
inner ones, never the reverse.

```text
 interface   ──►  application  ──►  domain  ◄──  infrastructure
 (CLI, Tauri,      (use cases,       (model,       (adapters: wgpu,
  examples,         ports for I/O,    rules,        git CLI, fs, OS,
  host apps)        orchestration)    ports)        audio device)
                         ▲                                │
                         └──── implements ports of ───────┘
```

| Layer | Contains | May depend on | Must not contain | Crate name suffix |
|---|---|---|---|---|
| **Shared kernel** | value types used by several contexts: math, ids, units, color | `std`, `core`, small pure crates (`serde` behind a feature) | I/O, `async`, domain rules of one context | none: `<product>-<concept>` |
| **Domain** | entities, value objects, aggregates, domain services, domain events, port traits the domain needs | shared kernel | I/O, `async`, `tokio`, GPU/OS/file APIs, logging back ends, serialization formats | `-domain` |
| **Application** | use cases (one per command), port traits for I/O, transactions, scheduling, cross-context orchestration | domain crates, shared kernel | business rules (they go into the domain), concrete technology | `-app` |
| **Infrastructure** | adapters implementing ports: GPU, audio device, file system, process, `git` executable, persistence | the ports it implements (domain/app), the technology crate | business rules, decisions the domain should make | `-<technology>`: `-wgpu`, `-cpal`, `-gitcli`, `-fs` |
| **Interface** | composition root: wires adapters into use cases; CLI, Tauri commands, examples, binaries | everything above | business rules | `-cli`, `-tauri`, product name for the app |

Rules:

- **The domain owns its ports.** When a domain or use case needs something
  from the outside world (a clock, a mesh upload, a process run), it declares
  a trait in its own crate. The adapter crate depends on the domain and
  implements the trait. The domain never depends on a driver.
- **Composition happens once,** in the interface layer (the composition
  root): the only place that names concrete adapters.
- **No layer skipping** in the other direction: interface code calls use cases,
  not adapters directly, except for wiring them up.
- **Cross-context calls go through the application layer** or through a
  published API of the upstream domain listed in the context map. A domain
  crate depends on another domain crate only as customer–supplier, listed.
- Layers and allowed edges are declared in `craftbook.toml` and checked by
  `craftcheck deps` on every PR.

## 7.4 Tactical building blocks in Rust

### Value object

Immutable, compared by value, valid by construction.

```rust
/// Mass of a body, in kilograms. Always finite and positive.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Mass(f32);

impl Mass {
    pub fn new(kilograms: f32) -> Result<Self, MassError> {
        if kilograms.is_finite() && kilograms > 0.0 {
            Ok(Self(kilograms))
        } else {
            Err(MassError::NotPositive { kilograms })
        }
    }

    #[must_use]
    pub fn kilograms(self) -> f32 {
        self.0
    }
}
```

- Private fields, `new`/`try_*` validates, no setters: "changing" returns a
  new value (`with_*`).
- Most of the domain should be value objects. They are cheap to test and
  cannot be in an invalid state.

### Entity

Has identity that survives changes of its attributes. Identity is a typed id.

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct BodyId(u32);

#[derive(Debug)]
pub struct RigidBody {
    id: BodyId,
    mass: Mass,
    pose: Motor,
    velocity: Velocity,
}
```

- Equality of entities is by id only; do not derive `PartialEq` over all
  fields.
- Fields private; behaviour as methods that keep invariants.

### Aggregate

A cluster of entities and value objects changed as one unit, behind one
**root**. Outside code holds the root (or its id) and calls its methods; it
never reaches inside to mutate a member.

- One aggregate per transaction/step: a command changes one aggregate.
  Coordination between aggregates is a domain service or the application
  layer, possibly through domain events.
- Aggregates refer to other aggregates **by id**, never by reference or
  `Arc`.
- Keep aggregates small. If the root's `impl` exceeds the limits of chapter 2,
  the aggregate boundary is wrong.

### Domain service

A stateless operation that does not belong to one entity or value object:
**a free function** in a module named after the concept
(`collision::intersect(a, b)`, `pricing::quote(order, tariff)`). See
[chapter 2 §2.3](02-functions.md#23-method-associated-function-free-function-or-trait).

### Domain event

Something that happened, in the past tense, as data:

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum PhysicsEvent {
    BodiesCollided { a: BodyId, b: BodyId, impulse: Impulse },
    BodyFellAsleep { body: BodyId },
}
```

- Aggregates return or push events; they do not call other aggregates.
- Events are values (no handles to live objects) so they can be logged,
  replayed and sent across threads.

### Repository and other ports

A trait in the domain or application crate describing what the model needs,
in the model's words:

```rust
pub trait MeshStore {
    fn upload(&mut self, mesh: &MeshData) -> Result<MeshId, MeshStoreError>;
    fn release(&mut self, id: MeshId);
}
```

- Ports are small (≤ 5 methods) and named by role (`Clock`, `MeshStore`,
  `ProcessRunner`), not by technology.
- Every port has at least one real adapter and one fake (`InMemoryMeshStore`)
  used by domain and application tests.

### Factory

Complex creation is an associated function on the root
(`World::with_gravity(g)`) or a builder (`WorldBuilder`). A separate
`…Factory` type exists only when creation needs injected ports.

### Use case (application service)

One function or one small struct per command, named after the command:

```rust
pub fn open_project(fs: &impl FileSystem, path: &ProjectPath) -> Result<Project, OpenProjectError> { … }
```

The use case loads what it needs through ports, calls the domain, saves
through ports, and emits events. It contains no rule that the domain could
own.

## 7.5 What may cross a layer boundary

- Into the domain: domain types, shared-kernel types, primitives. Never a
  driver object, a file path string meant for the OS, a JSON value.
- Out of the domain: domain types and events. The interface layer maps them to
  DTOs (`serde` structs) for the UI or the wire.
- `serde` derives live on DTOs in the interface/infrastructure layer, or
  behind a `serde` feature on shared-kernel value types. Domain types are not
  shaped by a file format.

## 7.6 Anticorruption layers

When a context consumes a foreign model (a third-party crate, another
context's data, a file format, `git`'s porcelain output), the translation
lives in one adapter module that turns the foreign shape into our types. No
foreign type appears in a domain signature.

## 7.7 No managers, no hubs

- No type whose job is "holds everything" (`Engine`, `Context`, `App` with
  forty fields reached from everywhere). The composition root builds the
  graph of values once and passes each one what it needs.
- No crate that every other crate depends on except the shared kernel, and
  the kernel stays small (types, not services).

## 7.8 Performance kernels

Hot paths of an engine (ECS storage, SIMD math, physics solvers, GPU kernels
and the buffers they read) are allowed a data-oriented shape that tactical DDD
would not produce: structure-of-arrays storage, plain-old-data `#[repr(C)]`
structs with public fields, index-based access, free functions over slices.

The conditions:

1. The crate or module declares itself a kernel in its `//!` doc and in
   `craftbook.toml` (`profile = "kernel"`).
2. It is still placed in the correct layer, follows naming, file and
   function limits, and has its own tests and benchmarks.
3. It exposes a typed, validated API to the rest of the domain (value objects
   and ids), so the data-oriented layout stays an internal detail.
4. The decision has a benchmark that justifies it.
