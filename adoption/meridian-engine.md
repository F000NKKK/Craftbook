# Adoption: Meridian-Engine

Status: **not adopted**. Measured at `61119e3` (2026-10-27) against chapters 01–05,
07, 08. No exceptions are granted: every row is debt, and numbers only go down (§8.1).

## 1. Measured state

How it was measured (read-only; artefacts in `/tmp/claude-0/meridian-audit`):
a Python script counting physical lines per `.rs` file, excluding the
`#[cfg(test)] mod … { }` block by brace matching; `cargo clippy --workspace
--all-targets` with `CLIPPY_CONF_DIR` → `too-many-lines-threshold = 50`,
`too-many-arguments-threshold = 5`, `excessive-nesting-threshold = 4`,
`cognitive-complexity-threshold = 15` (12.6 s warm); a second clippy run on
`--lib --bins --examples` with `missing_docs`, `missing_debug_implementations`
and `clippy::{missing_errors_doc, missing_panics_doc, module_name_repetitions,
unwrap_used, expect_used, panic, undocumented_unsafe_blocks,
let_underscore_must_use}`, de-duplicated by file/line/column. The build needs
system ALSA headers (`alsa-sys`), installed in the audit container only.

### 1.1 Files and modules (ch. 1)

| Metric | Value | Limit |
|---|---|---|
| `.rs` files (24 crates + `examples`) | **80** (28 220 lines: 20 527 production, 7 693 inline test) | — |
| Files over 400 production lines | **15** | 0 |
| Files over 250 production lines | **29** | target 0 |
| Inline test modules over 150 lines | **17** | 0 (then `tests.rs`) |
| `lib.rs`/`mod.rs` with logic, crate has > 1 file | **7 `lib.rs` + 2 `mod.rs`** | 0 |
| Single-file crates whose `lib.rs` exceeds 250 lines | **6** of 12 single-file crates | 0 |
| Files with > 1 public `struct`/`enum`/`trait` (rough: companions not excluded) | **40**; 16 of them with ≥ 5 | ≈ 0 |
| Glob re-exports `pub use x::*` / files named by kind (`generic.rs` ×3, `generic_softbody.rs`) | **3** / **4** | 0 / 0 |
| `tests/`, `benches/`, `fuzz/` directories | **0** | per §1.6/§8.2 |

Files over 400 production lines (production/inline-test lines, public types):
`graphics-core/src/submission.rs` 1457/277 (8), `graphics-driver/src/lib.rs`
992/63 (12), `physics-core/src/generic.rs` 985/0 (8), `gac-core/src/float_ga.rs`
836/540 (7), `gpu-driver/src/lib.rs` 792/108 (10), `gac-compute/src/fixed_wgsl.rs`
752/270 (2), `engine-core/src/lib.rs` 733/524 (12), `gac-core/src/generic.rs`
708/0 (17), `gac-core/src/fixed_ga.rs` 693/203 (6), `examples/examples/magic_figures.rs`
568/0, `examples/examples/physic_figures.rs` 511/0, `physics-compute/src/float.rs`
503/160 (2), `audio-core/src/lib.rs` 501/436 (8), `graphics-core/src/bloom.rs`
473/37 (2), `ecs-core/src/lib.rs` 427/151 (6).

`lib.rs` files that hold logic although the crate has submodules:
`audio-core` (501 lines, 20 definitions), `graphics-core` (305, 21),
`dsl-core` (241, 9), `asset-core` (202, 16), `gac-compute` (179, 12),
`foundation` (25, 2), `numeric-core` (21, 1); plus `asset-core/src/image/mod.rs`
(74, 5) and `audio-core/src/effects/mod.rs` (23, 1). Oversized single-file
crates: `graphics-driver` 992, `gpu-driver` 792, `engine-core` 733,
`ecs-core` 427, `audio-driver` 347, `compute-runtime` 306.

### 1.2 Functions (ch. 2)

| Metric | Value | Limit |
|---|---|---|
| Functions over 50 code lines (all targets) | **29** (27 in non-test code) | 0 |
| Functions over 5 parameters, reported | **10** | 0 |
| …plus functions hidden by `#[allow(clippy::too_many_arguments)]` | **9** (≥ 8 params each: they were silenced at clippy's default of 7) | 0 |
| Blocks nested deeper than 4 (`excessive_nesting`) | **67** | 0 |
| Cognitive complexity > 15 | 0 | 0 |

Worst functions: `physics-compute/src/fixed.rs:253` (119 lines, 7 params),
`platform-core/src/window.rs:229` (118), `physics-compute/src/float.rs:195`
(107, 7 params), `graphics-core/src/submission.rs:950` (98),
`dsl-macros/src/lib.rs:49` (89), `graphics-core/src/bloom.rs:363` (87, 6 params),
`graphics-core/src/submission.rs:1125` (87), `examples/…/magic_figures.rs:404`
(85). Seven-parameter functions: `graphics-driver/src/lib.rs:355,380,404`,
`graphics-core/src/submission.rs:1338`.

### 1.3 Types, errors, safety (ch. 4)

| Metric | Value | Limit |
|---|---|---|
| `#[allow(` / `#[expect(` | **15** / 0: 9 `too_many_arguments`, 5 `dead_code`, 1 `type_complexity` (`submission.rs` alone 4) | 0 / reasoned only |
| `.unwrap()` in non-test code | **68** (physics-compute 26, engine-core 13 — all `Mutex::lock()` in the `RuntimeState` hub, gac-compute 12, ecs-core 8 — archetype migration, task-core 4, audio-core 3, examples 2) | 0 |
| `.expect(` in non-test code | **30** (examples 9, physics-compute 6, gac-compute 4, gpu-driver 3, …) | invariant-only |
| `panic!` in non-test code | **22** (examples 18, sdk 2, …) | 0 in production paths |
| `let _ =` on `#[must_use]` results (swallowed errors) | **13** (foundation/logging 10) | 0 |
| `unsafe` blocks/impls without `// SAFETY:` | **12** reported, all in `platform-core/src/dynamic_library.rs` | 0 |
| Crates with `#![forbid(unsafe_code)]` | **0** of 24 | all but FFI crates |
| Public types without `Debug` | **27** | 0 |
| `#[must_use]` / `#[non_exhaustive]` | 0 / 1 (clippy suggests `must_use` on 306 items) | — |
| `pub` struct fields | **246** | DTO/POD/kernel only |
| `thiserror` / workspace `[lints]` / `clippy.toml` / `rustfmt.toml` | none | required |
| Global statics holding state (`OnceLock`) | 3: `foundation/logging.rs` ×2, `asset-core/…/compressed_audio.rs` | 0 mutable |

### 1.4 Naming (ch. 3)

- `get_` prefixes: **1** (`ecs-core::get_or_create_archetype` → `archetype_for` or `find_or_insert_archetype`).
- Stutter (`clippy::module_name_repetitions`): **33**, e.g. `asset::audio::audio_streaming`,
  `audio::compressed_audio`, `soft_body::{float,fixed,generic}_softbody`,
  `image::png_decoder`, `bloom::BloomPass`/`BloomConfig`, `scene::Scene3D`,
  `capabilities::CpuCapabilities`, `dynamic_library::DynamicLibraryError`,
  `render::render_frame`, `dsl::DslError`. Crate-level stutter as well:
  `audio-driver::{AudioDevice, AudioStream, AudioStreamConfig}`,
  `compute-runtime::{ComputeContext, ComputeKernel, ComputeBuffer, …}`.
- Noise words in type names: **7** — `ImageData`, `MeshData`, `AudioData`
  (asset-core), `LightData` (graphics-core), `ArchetypeData` (ecs-core),
  `GraphicsBase` (sdk), `AppHandler` (platform-core). The crate description of
  `engine-core` calls it a "subsystem manager".
- Abbreviations in production identifiers (snake_case parts): `pos` 55,
  `buf` 52, `idx` 27, `ctx` 13, `val` 12, `msg` 9 (≥ 168; `cfg`/`num`/`info` not counted);
  acronym casing `Scene3D`, `Scene2D`, `Renderable3D` → `Scene3d`….
- Crate names: 0 of 24 follow `<product>-<context>-<layer>`.

### 1.5 Docs and comments (ch. 5)

- Every file has a `//!` doc (80/80), but **18** exceed 30 lines (`engine-core`
  80, `physics-compute` 74, `sdk` 61, examples 58/55, `constraint_solver.rs` 57 …),
  and they carry history narration ("Earlier revisions of this crate had…"; 14
  hits for such phrases in comments).
- `missing_docs`: **674** undocumented public items (gac-core 166, graphics-core 75,
  platform-core 71, physics-core 59, sdk 42 …); `missing_errors_doc` **30**, `missing_panics_doc` **40**.
- No crate doc names its layer and bounded context. `docs/glossary.md` and
  `docs/context-map.md` do not exist; `craftbook.toml` does not exist.

## 2. Coverage and tests (ch. 8)

- **Coverage: not measured yet.** `cargo-llvm-cov` is not installed in the audit
  environment; installing it was not permitted. First CI job of wave 0
  records the per-crate baseline here.
- Mutation testing, property tests (`proptest`), benchmarks (`criterion`),
  fuzzing, golden tests, contract tests: **none**.
- `cargo test --workspace --no-fail-fast`: **361 tests, 357 passed, 4 failed**
  (the 4 `audio-driver` tests need a real audio device; they are not behind a
  `hw-tests` feature). No CI configuration in the repository.

Tests per crate (`#[test]` count): gac-core 45, audio-core 37, physics-core 34,
graphics-core 31, asset-core 30, sdk 23 (+1 doctest), physics-compute 18,
engine-core 16, platform-core 14, ecs-core 12, gac-compute 12, dsl-core 11,
numeric-core 10, compute-runtime 9, resource-core 9, audio-driver 8,
compute-driver 8, memory-core 8, task-core 6, foundation 5, gpu-driver 5,
graphics-driver 4, physics-driver 3, examples 2, **dsl-macros 0**.

## 3. Bounded contexts and layer map

Twelve contexts: geometry, entity, identity (shared kernels), diagnostics, platform,
compute, assets, rendering, physics, audio, scene-markup (DSL), runtime.

| Current crate | Layer today (actual) | Context | Target crate(s) |
|---|---|---|---|
| foundation | kernel **+ infra** (global log sink, tokio file I/O, panic hook) | diagnostics | `meridian-diagnostics` (kernel: `EngineError`, `Log` port) + `meridian-diagnostics-fs` |
| numeric-core | kernel | geometry | `meridian-numeric` |
| gac-core | kernel (profile kernel) | geometry | `meridian-gac` |
| memory-core | kernel (profile kernel) | identity | `meridian-memory` |
| resource-core | kernel | identity | `meridian-resource` |
| ecs-core | kernel (profile kernel) | entity | `meridian-ecs` |
| task-core | infra (OS threads) | runtime | `meridian-runtime-threads` (implements `JobExecutor` port of runtime-app) |
| platform-core | infra (winit, dlopen FFI, time) **+ domain** (input state) | platform | `meridian-input` (kernel) + `meridian-platform-winit` + `meridian-platform-os` |
| gpu-driver | infra | compute/rendering | `meridian-gpu-wgpu` |
| compute-driver | infra | compute | `meridian-compute-wgpu` |
| compute-runtime | app (ports + dispatch policy) **depending on infra** | compute | `meridian-compute-app` (owns `ComputeDevice`, `ComputeKernel` ports) |
| gac-compute | infra (WGSL kernels) | geometry | `meridian-gac-wgpu` |
| asset-core | domain (CPU types, `Decoder`) **+ infra** (symphonia, png, `std::fs`, global codec registry) | assets | `meridian-asset-domain` + `meridian-asset-symphonia` + `meridian-asset-png` (BMP/WAV/OBJ: see wave 5) |
| graphics-core | domain (scene, materials, render graph) **+ infra** (`submission.rs`, `bloom.rs` record `graphics-driver` `CommandBuffer`/`RenderPass`) | rendering | `meridian-rendering-domain` + `meridian-rendering-app` (frame submission use case) |
| graphics-driver | infra | rendering | `meridian-rendering-wgpu` (implements rendering ports, absorbs submission/bloom recording) |
| physics-core | domain (profile kernel) **with a dead `physics-driver` edge** (in Cargo.toml, never `use`d) | physics | `meridian-physics-domain` |
| physics-driver | infra (only a capability report) | physics | delete; capability report moves to `meridian-platform-os` |
| physics-compute | infra (WGSL + batched kernels) | physics | `meridian-physics-wgpu` |
| audio-core | domain (mixer, DSP, listener) **+ infra** (`AudioOutput::open` is `async` and drives `audio-driver`) | audio | `meridian-audio-domain` (owns `AudioSink` port) |
| audio-driver | infra (cpal) | audio | `meridian-audio-cpal` |
| engine-core | app **+ hub** (`RuntimeState` holds World/Physics/Audio behind `Mutex`es) | runtime | `meridian-runtime-app` (one use case per frame step; no state hub) |
| dsl-core | infra (text parser) / published language | scene-markup | `meridian-markup` (kernel parser) |
| dsl-macros | interface (build-time tooling) | scene-markup | `meridian-markup-macros` |
| sdk | interface **+ domain/app logic** (`scene.rs` 325, `camera.rs`, `render.rs`, `dsl.rs` tags) | — (composition root) | `meridian-sdk` (facade + composition root only); `FlyCamera` → rendering-domain, DSL tags → `meridian-scene-app` |
| examples | interface | — | `meridian-examples` |

Draft `craftbook.toml` (target state, to be committed with wave 0 so
`craftcheck deps` can report the gap):

```toml
[[crate]]
name = "meridian-gac"              # today: meridian-gac-core
layer = "shared-kernel"
context = "geometry"
profile = "perf"
[[crate]]
name = "meridian-physics-domain"   # today: meridian-physics-core
layer = "domain"
context = "physics"
profile = "perf"
[[crate]]
name = "meridian-runtime-app"      # today: meridian-engine-core
layer = "app"
context = "runtime"
[[crate]]
name = "meridian-sdk"
layer = "interface"
context = "runtime"
# … one [[crate]] per row of the table above

[[context_edge]]
from = "physics"
to = "geometry"
relation = "shared-kernel"         # also rendering, audio -> geometry; physics, rendering -> entity
[[context_edge]]
from = "rendering"
to = "assets"
relation = "published-language"    # decoded image/mesh/shader CPU types; audio -> assets: PCM
[[context_edge]]
from = "physics"
to = "compute"
relation = "customer-supplier"     # also rendering -> compute
[[context_edge]]
from = "runtime"
to = "physics"
relation = "customer-supplier"     # also runtime -> audio, runtime -> rendering
[[context_edge]]
from = "scene-markup"
to = "runtime"
relation = "acl"                   # markup tags translated into runtime commands
```

## 4. Migration plan (waves, lowest risk first; each numbered item is one PR)

**Wave 0 — gates without code changes.** (1) Add `clippy.toml`, `rustfmt.toml`,
`[workspace.lints]` from Craftbook templates with all size lints at *warn*,
plus `craftbook.toml` above; (2) CI job running fmt, clippy, tests with
`--no-fail-fast`, `cargo llvm-cov` (record baseline in §2); (3) put the 4
`audio-driver` device tests behind a `hw-tests` feature; (4) add
`docs/glossary.md` and `docs/context-map.md` (one section per context above).

**Wave 1 — mechanical hygiene.** (1) Replace the 15 `#[allow]` with fixes or
`#[expect(…, reason)]`; delete the `dead_code` items; (2) `// SAFETY:` on the
12 blocks in `dynamic_library.rs`; `#![forbid(unsafe_code)]` in the 23 other
crates; (3) `Debug` on the 27 types; (4) the 68 `unwrap()` and 13 `let _ =`:
`engine-core` lock unwraps and `ecs-core` migration unwraps first; (5) trim
the 18 long `//!` docs to ≤ 30 lines, moving rationale into `docs/`, and drop
history narration.

**Wave 2 — `lib.rs` becomes wiring.** Move definitions out of the 7 `lib.rs`
and 2 `mod.rs` files into item-named files: `audio-core` (Listener, Emitter,
Speaker, SpeakerLayout, AttenuationModel, AudioOutput), `graphics-core`
(render graph types), `asset-core` (decoded types, `Decoder`, `DecodeError`),
`dsl-core`, `gac-compute`, `foundation`, `numeric-core`. Move the 17 oversized
inline test modules into `tests.rs`. Rename by the naming list in §1.4
(stutter, `*Data`, `get_or_create_archetype`, `3D`, abbreviations).

**Wave 3 — split the top offenders, inside the existing crates.**
`graphics-core/submission.rs` (1457 → `mesh_registry.rs`, `material_registry.rs`,
`texture_registry.rs`, `draw_list.rs`, `light.rs`, frame recording), then
`graphics-driver/lib.rs` (992 → device, surface, command_buffer, render_pass,
pipeline), `physics-core/generic.rs` (985 → `rigid_body`, `collider`,
`broad_phase`, `narrow_phase`, `contact`, `solver`, `integrator`),
`gpu-driver/lib.rs`, `engine-core/lib.rs`, `ecs-core/lib.rs` (world, archetype,
column, query), `gac-compute/fixed_wgsl.rs` (WGSL strings → `*_table.rs`).
Fix the 29 long and 19 wide functions on the way (introduce `…Options` and
value objects such as `DrawCall`, `BlurPass`). Add `proptest` for gac-core and
numeric-core, `criterion` benches for every `profile = "perf"` crate (§7.8 #4).

**Wave 4 — float/fixed layout (needs decision D5).** Turn
`gac-core/{generic,float_ga,fixed_ga}.rs` (2237 prod lines, 30 public types)
into one directory per concept (`rotor/`, `motor/`, `vector/`, `shape/`…),
each with `mod.rs` + `float.rs` + `fixed.rs`; the same for `physics-core`,
`physics-compute`, `soft_body`. Move `physics-core/float.rs`'s 493 lines of
tests of generic code next to the generic code.

**Wave 5 — invert driver dependencies (needs decision D1).** One context per
PR, smallest first: physics (drop the unused `physics-driver` edge, delete the
crate), audio (`AudioSink` port in audio-domain; `audio-cpal` implements it;
`AudioOutput::open` leaves the domain), compute (`ComputeDevice` port in
compute-app; `compute-wgpu`, `gac-wgpu`, `physics-wgpu` implement it),
rendering (ports `MeshStore`, `TextureStore`, `FrameRecorder` in
rendering-domain/app; `submission`/`bloom` recording moves into
`rendering-wgpu`), assets (split codecs out of asset-domain; decide whether
hand-written BMP/WAV/OBJ decoders are infra `meridian-asset-builtin` or domain
parsers). Each adapter gets a fake + contract suite (§8.4).

**Wave 6 — runtime and composition root (needs D2, D3).** Replace
`RuntimeState`'s `Mutex` hub with per-step use cases in `runtime-app`; the
only place naming concrete adapters becomes `meridian-sdk`/examples. Move
`sdk`'s scene/camera/render logic to its context crates. Split `foundation`.

**Wave 7 — crate renames (needs D4)** to the `-domain/-app/-<tech>` names,
then raise all size lints from *warn* to *deny*.

## 5. Conflicts with Meridian's own rules — decisions needed

Each item needs an ADR in `docs/adr/` (or a Craftbook PR) before its wave starts;
until then `check-deps` enforces the old Meridian rule.

| # | Meridian rule | Standard | Recommendation |
|---|---|---|---|
| D1 | ADR 005 + `dependency-rules.md` rules 1–2 + CLAUDE.md "keep driver crates below their domain cores": edges point `*-core → *-driver`; ADR 005 explicitly *rejects* "backend crates depending on domain crates". | §7.3: the domain owns its ports; adapters depend on the domain; "the domain never depends on a driver". | New ADR superseding 005: ports in domain/app crates, drivers become `-<tech>` adapters. Affected edges today: graphics-core→graphics-driver, audio-core→audio-driver, physics-core→physics-driver, compute-runtime→compute-driver, gac-compute→gpu-driver, physics-compute→gpu-driver, sdk→graphics-driver. Rewrite rules 1, 2, 9, 12 and `check_dependency_rules.py`. |
| D2 | Rule 7 + engine-core doc: engine-core is "the only crate allowed to depend on every `*-core`"; `RuntimeState` owns World/Physics/Audio behind locks. | §7.7 no hub types/crates; §7.3 cross-context orchestration as application use cases, composition once in the interface layer. | ADR: engine-core becomes `runtime-app` (use cases, no shared state owner); state ownership and wiring move to the composition root. |
| D3 | `dependency-rules.md`: `sdk` is a "second, narrower" hub and the place that combines engine-core with drivers. | §7.3: that is exactly the interface layer — allowed — but it may not hold business logic. | Keep sdk as composition root; ADR records that its `scene`, `camera`, `render`, `dsl` logic moves out. |
| D4 | Crate naming `*-core`, `*-driver`, `*-compute`, `*-runtime`; crates are versioned for publishing (`release.sh`). | §3.5 `<product>-<context>-<layer>`. | ADR on the rename (and crates.io deprecation of old names), done last (wave 7). |
| D5 | CLAUDE.md "Float/Fixed branching: always 3 files" — `lib.rs` may hold the shared/generic logic, flavours in `float.rs`/`fixed.rs`, one trio per *crate*. | §1.4 `lib.rs` is wiring only; §1.2 one primary item per file; §1.3 #4 one submodule per flavour *per concept*. `float_ga.rs` holds 7 public types, `generic.rs` 17. | Change CLAUDE.md: the trio becomes per concept directory (`rotor/{mod,float,fixed}.rs`), `mod.rs` wiring only, generic logic in item-named files; ban `generic.rs` as a file name. Keep the rule's "generic over `GaFlavor`, no duplication" half — it matches §2.5. |
| D6 | ADR 008 / CLAUDE.md: `Motor3`/`Vec3`/`Bivector3`/`Rotor` duplicated as concrete `f32` for GPU dispatch. | §2.5 allows exactly this with the constraint in the item doc comment. | No conflict in substance; add the constraint to each item's `///`, and still split the files (D5). |
| D7 | Rule 0 + ADR 003: `foundation` is open to all, and its process-wide log sink (`static SINK: OnceLock`, tokio file writer, panic hook) is declared "not a global manager". | §4.7 no global mutable state; §7.3 the shared kernel contains no I/O and no `async`. | ADR: a `Log` port in `meridian-diagnostics` (kernel), file sink and crash reporter as `meridian-diagnostics-fs`, sink passed in from the composition root. Same for `asset-core`'s global `CodecRegistry`. |
| D8 | ADR 004 data-oriented ECS/physics: 246 `pub` fields, SoA, index access. | §7.8 allows it only with `profile = "perf"`, a typed API at the edge and a justifying benchmark. | Declare gac/ecs/memory/physics kernels in `craftbook.toml` and `//!`; benchmarks are missing today, so the profile is provisional until wave 3 adds them. |
| D9 | Meridian is internally inconsistent: rule 12 says only graphics-driver and compute-driver may depend on gpu-driver, but the edge list allows gac-compute and physics-compute too; physics-core lists physics-driver but never uses it. | §5.4 docs are updated in the change that makes them wrong. | Fix in wave 0 independent of D1. |
| D10 | `scripts/check_dependency_rules.py` `ALLOWED` dict is the source of truth for edges. | §7.3/§8.5: `craftbook.toml` + `craftcheck deps`. | One source only: generate `ALLOWED` from `craftbook.toml` during the transition, then delete the script. |
| D11 | `AGENTS.md`, `CLAUDE.md`, `.claude/skills/*` and `.codex/skills/*` (8 skill sets each) restate architecture rules. | §5.4: agent guides link Craftbook and keep only project rules. | Rewrite in the same PRs as D1/D5, or agents keep enforcing the old direction. (CLAUDE.md "async only on genuine I/O" already matches §4.8; `audio-core::AudioOutput::open` is still async in a domain crate — fixed by D1.) |

`craftcheck` on today's tree, with the current-name layer map in
[`meridian-engine/craftbook.toml`](meridian-engine/craftbook.toml) (each crate mapped to
the layer it is meant to be in):

- `craftcheck check`: 63 errors, 67 warnings — C101 ×16 (files over 400), C102 ×16
  (test modules over 150), C103 ×28 (logic in wiring files), C108 ×22, D302 ×17,
  D305 ×13, A501 ×1 (`CodecRegistry`), N201 ×1, N202 ×1. Type-level stutter is left to
  `clippy::module_name_repetitions` (the 33 cases above).
- `craftcheck deps`: 36 errors — L602 ×13 (domain/app crates depending on drivers and
  compute adapters: conflict D1), L603 ×6 (platform used by drivers without a
  context-map edge), L604 ×1, L605 ×32 (I/O or `async` in kernel/domain source).

Wave 0 commits this file as `craftbook.toml` and both outputs as baselines.
