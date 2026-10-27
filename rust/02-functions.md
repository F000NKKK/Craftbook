# 2. Functions, methods and traits

## 2.1 Limits

| What | Target | Hard limit | Lint |
|---|---|---|---|
| Lines in a function body (code lines, without comments and blank lines) | ≤ 25 | **50** | `clippy::too_many_lines` |
| Parameters, `self` included | ≤ 4 | **5** | `clippy::too_many_arguments` |
| Nested blocks inside a method or test (free function: one more) | ≤ 2 | **3** | `clippy::excessive_nesting` (threshold 5: it counts the `impl`/`mod` and the body) |
| Cognitive complexity | ≤ 10 | **15** | `clippy::cognitive_complexity` |
| `bool` parameters | 0 | **0** for public functions, 1 for private | review, `clippy::fn_params_excessive_bools` |
| Public methods in one inherent `impl` | ≤ 12 | **20** | review |
| Fields in one struct | ≤ 7 | **10** | review |
| `bool` fields in one struct | ≤ 1 | **2** | `clippy::struct_excessive_bools` |

The limits apply to tests too. A test that needs more than 50 lines is
missing a fixture or a builder.

Never silence these lints with `#[allow]`. If a signature really is fixed by
something outside the code (an FFI ABI, a shader's binding layout, a trait
from another crate), use `#[expect(clippy::too_many_arguments, reason = "…")]`
with the external constraint named in the reason.

## 2.2 Shape of a function

- **One level of abstraction per function.** A function either orchestrates
  (calls named steps) or does a step (works with the data). Mixing both is
  the most common reason a function grows past 25 lines.
- **Early return over nesting.** Guard clauses with `?`, `let … else`, and
  `return` first; the main path stays at the left margin.
- **Command–query separation.** A function returns information (`&self`, no
  observable change) or changes state (`&mut self`, returns `()` or
  `Result<(), E>`), not both. Exceptions are the standard ones: `pop`,
  `take`, `insert` returning the old value, and a creation that returns the
  new id.
- **Too many parameters means a missing type.** Three parameters that travel
  together (`origin, direction, max_distance`) are a value object (`Ray`).
  Configuration goes in a `…Options`/`…Config` struct with `Default`.
- **No flag parameters.** `render(scene, true)` is two functions
  (`render_lit`, `render_unlit`) or an enum (`render(scene, Shading::Lit)`).
- **No out-parameters.** Return a value, a tuple of at most two, or a named
  struct. `&mut` parameters are for genuinely mutating a caller's buffer
  (`fn fill(&self, out: &mut [f32])` in hot paths), named `out`/`dst`.

## 2.3 Method, associated function, free function or trait?

This is the decision C# makes for you by having no free functions. Rust does
not, so we make it explicitly. Ask the questions in order; the first "yes"
decides.

### 1. Does it need the private fields or protect an invariant of one type? → **method**

If the operation reads or changes a type's internal state, or would break an
invariant if done from outside, it is a method on that type. This is the
aggregate rule from DDD ([chapter 7](07-architecture-ddd.md)): state changes
only through the owner.

```rust
impl Inventory {
    pub fn reserve(&mut self, sku: Sku, quantity: Quantity) -> Result<Reservation, ReserveError> { … }
}
```

### 2. Does it create a value of the type? → **associated function**

Constructors and factories live on the type: `new`, `with_capacity`,
`from_parts`, `try_from_bytes`, `default`. Conversions from other types are
`From`/`TryFrom` impls, not free functions.

### 3. Is there a clear subject, and the others are arguments to it? → **method**

`camera.project(point)`, `path.join(segment)`. The subject is the noun the
operation is about in the [glossary](07-architecture-ddd.md#71-ubiquitous-language).
If you can say "the *camera* projects", it is a method on `Camera`.

### 4. Are all inputs equal peers, with no subject? → **free function**

Symmetric operations do not belong to either operand:
`lerp(a, b, t)`, `distance(a, b)`, `intersect(ray, shape)`,
`merge(left, right)`. In DDD terms these are **domain services**: stateless
operations that span several values or aggregates. They are free functions in
the module whose name is the concept (`collision::intersect`), not methods on
a made-up struct.

### 5. Does it act on something outside the process, with no state of its own? → **free function in a named module**

Operations on the file system, a child process, an executable such as `git`,
where the only "state" is a path or a handle passed in:
`git::status::read(root)`, `fs::watch::start(dir, sink)`. The module is the
namespace. A struct with no fields (`struct GitService;`) that exists only to
hang functions on is forbidden — it is a manager in disguise.

### 6. Do two or more types need to be used interchangeably? → **trait**

A trait exists when there are, or are about to be, two implementations used
through the same code: a port with a real and a fake adapter, a numeric
flavour (`f32` and fixed-point), a back end. Rules:

- A trait with one implementation and no generic consumer is not a trait yet;
  write the type. "We might need it" and "for mocking" alone do not count —
  a test fake is a second implementation only when the trait is a port
  (chapter 7).
- Traits are small: ≤ 5 required methods. A big trait is several roles; split
  it and combine with supertraits.
- No default method bodies that hide missing behaviour (`fn flush(&mut self) {}`).
  A default is fine when it is a real derivation from required methods.
- Extension traits (`trait Vec3Ext`) on our own types are forbidden; add the
  method. On foreign types, a free function is preferred over an extension
  trait unless method syntax is used in more than a few places.

### 7. Otherwise → **private free function**

A helper used by one module's functions, with no subject, is a private
`fn` at the bottom of the file. If it needs more than three arguments taken
from the same struct, it is a method on that struct instead.

### Summary table

| Situation | Form | Example |
|---|---|---|
| Changes or reads private state, protects an invariant | method | `inventory.reserve(sku, qty)` |
| Creates the type | associated fn / `From` | `Handle::new(index, generation)` |
| Has an obvious subject | method | `camera.project(p)` |
| Symmetric over peers, stateless | free fn (domain service) | `collision::intersect(a, b)` |
| Talks to the outside world, no own state | free fn in a named module | `git::status::read(root)` |
| Several interchangeable implementations | trait | `trait Clock`, `trait GaFlavor` |
| Local helper | private free fn | `fn clamp_index(i, len)` |
| Grouping functions under an empty struct | **forbidden** | `struct MathUtils;` |
| Global mutable state behind functions | **forbidden** | `static INSTANCE: Mutex<Manager>` |

## 2.4 `self` receivers

- `&self` for queries, `&mut self` for commands, `self` for conversions and
  builders (`into_*`, `build`, `with_*` on a builder).
- Take `self` by value on `Copy` types ≤ 16 bytes (`Vec3`, `Handle`); clippy's
  `trivially_copy_pass_by_ref` enforces it.
- Builders take and return `self` (`Builder::new().width(800).build()`),
  never `&mut self` returning `&mut Self`.

## 2.5 Closures and generics

- A closure longer than 5 lines is a named function.
- Prefer `impl Trait` in argument position for one-off generic parameters;
  name the generic (`<F: Flavor>`) when it appears more than once or in the
  return type.
- `where` clauses when bounds exceed one trait per parameter.
- Generic code is written once over a trait describing what it needs; it is
  not duplicated per concrete type. Duplication is allowed only under a
  documented external constraint (GPU-dispatched types that must stay `f32`
  are the known case), written in the item's doc comment.
