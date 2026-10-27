# Adoption: Substrate-Platform

Status: **not adopted**. This page records where Substrate-Platform
(`substrate-platform` 0.4.0 plus `tauri-plugin-substrate` 0.1.0, commit
`1eb9ad2`) stands against chapters 1–8 and how it gets there. The TypeScript
`ui/` is out of scope. Every number below comes from a command run on that
commit (Rust 1.99.1); how it was measured is given with each table.

## 1. Measured state

Tools: `find`/`grep`; a line-counting script that removes each inline
`#[cfg(test)] mod … { }` block and every `tests.rs`; `cargo clippy` on the
core crate with `too-many-lines-threshold = 50`, `too-many-arguments-threshold = 5`,
`excessive-nesting-threshold = 4`, `cognitive-complexity-threshold = 15`, and
the chapter lints turned on as warnings. **`tauri-plugin` could not be
built here** (`gdk-3.0` system library missing), so its function numbers
come from a script estimate.

### Size (chapter 1)

| Measure | Value |
|---|---|
| `.rs` files | 68 (55 `src/`, 12 `tauri-plugin/`, 1 `benches/`) |
| Physical lines / production lines | 7 496 / 6 017 |
| Files over the 400-line hard limit (production) | **0** |
| Files over the 250-line target | 2: `src/run/task/mod.rs` 363, `src/git/status.rs` 341 (+120 test lines = 461) |
| Inline test module over 150 lines | 1: `src/run/workflow.rs` (174) |
| `mod.rs`/`lib.rs` with submodules **and** logic (§1.4) | 7: `src/run/task/mod.rs` (363 lines, the whole `Task`), `src/search/mod.rs` (249: `Layout`, `SearchQuery`, `Matcher`, `FileMask`), `src/state/mod.rs`, `src/process/mod.rs`, `src/commands/mod.rs`, `tauri-plugin/src/lib.rs` (185), `tauri-plugin/src/git/mod.rs` |
| Module directories with a single file (§1.3, "fold it back") | 2: `tauri-plugin/src/search/`, `tauri-plugin/src/terminal/` |
| Files with more than one public type (rough; companions included) | 19. Worst: `project/model.rs` 10, `git/status.rs` 6, `search/mod.rs` 5, `search/run.rs` 5, `run/workflow.rs` 5, `search/replace.rs` 4 |
| Glob re-exports (§1.5) | 4, all in `src/search/mod.rs` (`pub use files::*` …) |
| Files without a `//!` doc | 0 (longest is 17 lines; none over 30) |

### Functions (chapter 2): clippy, core crate, `--all-targets --all-features`

| Lint | Hits |
|---|---|
| `too_many_lines` (> 50) | 5: `search::run::run_search` 61, `search::replace::run_replace` 58, `git::run::stream` 53, `run::task::finish_streamed` 53, test `project_search_skips_ignored_build_and_binary_files` 80 |
| `too_many_arguments` (> 5) | 2: `search::run::plan` 6, `terminal::PtySession::spawn` 6 |
| `excessive_nesting` (> 4) | 5 blocks: `fs/watch.rs:49`, `git/run.rs:148`, `search/mod.rs:228`, `search/replace.rs:89`, `terminal/session.rs:171` |
| `cognitive_complexity` (> 15) | 0 |
| `struct_excessive_bools` | 1: `SearchQuery` |
| `bool` parameters on public fns (grep) | 8 on 7 fns: `git::status::commit` (`amend`, `stage_all`), `git::branch::{switch, create, delete}`, `git::remote::operation`, `search::read_text`, `Matcher::replace_all` |
| `tauri-plugin` (script estimate) | `init_with` ≈ 122 code lines (one builder chain of `cfg`'d registrations); no fn over 5 params |

Clippy with default lints is clean: every warning of the run above comes
from the added thresholds.

### Types, errors, safety (chapter 4)

| Measure | Value |
|---|---|
| `#[allow(` | 0 |
| `.unwrap()` / `.expect(` outside tests | 0 / 0 in `src` and `tauri-plugin`; 5 `unwrap()` in `benches/core.rs` |
| `anyhow` in public signatures | 5: `run::WorkflowFile::parse`, `PtySession::{spawn, spawn_with, spawn_utf8, resize}` |
| Error type of the crate | `Result<_, Message>` on 64 lines (≈ 41 `pub`/`pub(crate)` fns): a user-facing i18n text used as the error, not a typed enum; `TextView::write` returns `Result<(), String>`; 16 `io::Result` |
| `thiserror` | not a dependency |
| `let _ = fallible…` discards in production code (§4.4) | 15: 7 core (`fs/watch.rs` 2, `git/config.rs`, `git/diff.rs`, `run/task/mod.rs` 3), 8 plugin (`emit`/`send`/`kill`) |
| Global mutable state (§4.7) | 3 statics: `text_view::VIEWS` (`RwLock<HashMap>` registry, written at run time), `git::run::WRITE_LOCK` (`static Mutex<()>`), `git::diff::VIEW_SEQ` (`AtomicU64`) |
| `unsafe` | 1 production block (`kill_tree`, has `// SAFETY:`), 1 in tests; no crate has `#![forbid(unsafe_code)]` or any crate-level lint attribute; no `[lints]` table |
| `missing_debug_implementations` | 27 public types |
| `pub` struct fields (grep `pub name:`) | 174 lines |
| `serde` derives on core types | 47 in 19 files: the model is shaped by the UI wire format (§7.5) |
| Domain logic doing I/O | e.g. `git::status::Operation::in_progress` probes files under `.git`; `git::status::status` runs git *and* parses its output in one module |

### Naming (chapter 3)

| Finding | Count / examples |
|---|---|
| `get_` prefixes | 0 |
| Noise word `Info` | 5: `DirEntryInfo`, `BranchInfo`, `CommitInfo`, `StashInfo`, `ShellInfo` |
| `clippy::module_name_repetitions` | 28 (22 prefix, 6 suffix): `fs::dir::{DirEntryInfo, list_dir, create_dir}`, `git::status::{RepoStatus, parse_status}`, `git::stash::{StashInfo, StashMode, StashOp}`, `git::diff::{DiffSide, DiffTexts}`, `run::workflow::Workflow{File, Def, Step, Result, Handle}`, `run::log::{LogLine, LogSink}`, `run::task::{TaskExit, TaskHandle}`, `search::SearchQuery`, `text_view::register_text_view` |
| Stutter clippy misses | `git::status::status(root)` (the standard's own example wants `git::status::read`), `search::run::run_search`, `search::replace::run_replace` |
| One name, two meanings in one crate | `search::ProjectEntry` (a walked file) vs `project::ProjectEntry` (a solution's project) |
| Abbreviations (bindings/params, grep) | `ws` 21, `cmd` 16, `m` 9 (`use messages as m`), `type Res<T>` alias in 4 plugin files |
| Abbreviated type names | `WorkflowDef`, `RemoteOp`, `BranchOp`, `StashOp` |

### Docs (chapter 5): clippy `missing_docs` & co., core lib

231 `missing_docs` warnings (126 struct fields, 33 methods, 25 variants,
19 structs, 7 enums, 7 fns, 7 assoc fns, 7 consts); 68 `missing_errors_doc`;
34 `must_use_candidate`. Worst files: `git/status.rs` 31, `project/model.rs` 30.
`docs/` holds only `images/`: no `glossary.md`, `context-map.md`, ADRs or
`craftbook.toml`. 0 `TODO`/`FIXME`.

## 2. Tests and coverage

`cargo-llvm-cov` 0.9.1 was installed for this audit and run as
`cargo llvm-cov -p substrate-platform --all-features --lib --ignore-run-fail --summary-only`.
`tauri-plugin` is not measured (does not build here; it has 0 tests).

| Area (core) | Tests | Line cov. | Region cov. |
|---|---|---|---|
| `run` | 17 | 92.7 % | 92.5 % |
| `git` | 13 | **45.1 %** (`remote` 11.4, `diff` 25.0, `config` 0) | 44.8 % |
| `search` | 12 | 89.6 % | 89.3 % |
| `project` | 8 | 85.6 % | 84.8 % |
| `fs` | 4 | 90.6 % | 87.8 % |
| `terminal` | 3 | **29.2 %** (`session`, `shell` 0) | 38.2 % |
| `commands`, `message`, `process` | 2 each | 100 / 46.0 / **27.4 %** | |
| `state`, `sync`, `workspace` | 1 each | 75.0 / 100 / 94.5 % | |
| `events`, `text_view` | 0 | 0 / 100 % | |
| **Total** | **66** | **68.57 %** (3 147 lines) | **68.00 %** |

- 65 pass, 1 fails in this container on every run (3 of 3):
  `run::task::tests::cancel_kills_grandchildren_too` ("grandchild survived the
  cancel"; probably an unreaped zombie under the container's init, not yet
  verified). It also polls with `thread::sleep` (8 `sleep`/`now` uses in 4
  files), against §8.3 "deterministic".
- No `tests/` integration directory, no property tests, no fakes or
  contract tests (there are no ports yet). One `criterion` bench file.
- `CONTRIBUTING.md` runs plain `cargo test`, i.e. **default features**, which
  skips every feature-gated module; CI must use `--all-features`.

Baseline to record (§8.1, only goes up): line 68.57 %, region 68.00 %.

## 3. Bounded contexts and layer map

Today one crate holds every context, gated by features (`fs`, `git`,
`project`, `run`, `search`, `state`, `terminal`; there is no `lsp`). The rule
"one bounded context = one domain crate + app + adapters" maps them as follows.

| Today | Context | Target crates (layer) |
|---|---|---|
| `message` (+ `messages!` macro, every `*/messages.rs`) | shared kernel | `substrate-message` (kernel) |
| `workspace`, `fs`, `text_view` | workspace | `substrate-workspace-domain` (`Workspace`, `DirectoryEntry`, `EntryKind`, port `TextView`), `substrate-workspace-fs` (std::fs + `notify` watcher) |
| `git` | source-control | `substrate-git-domain` (`RepoStatus`, `FileChange`, `ChangeKind`, `Operation`, `Branch`, `Commit`, `StashEntry`, `DiffTexts`; ports `WorkingTree`, `History`, `Branches`, `Remotes`, `Stashes`, each ≤ 5 methods), `substrate-git-app` (use cases; the write lock as an owned value), `substrate-git-gitcli` (infra: `cli/` runs `git`, maps stderr to `GitError`; `porcelain/` is the ACL that parses `status --porcelain=v2 -z`, `log`, `branch`, `stash list` into domain types, pure and fixture-tested) |
| `search` | search | `substrate-search-domain` (`SearchQuery`, `Matcher`, `FileMask`, `LineHit`, `Layout`, `Scope`), `substrate-search-app` (find/replace use cases, ports `FileWalker`, `DocumentSource`), `substrate-search-ignore` (`ignore`/`globset` walker) |
| `run` | run | `substrate-run-domain` (`Workflow`, `Step`, `Outcome`, `LogLine`, `Level`, `TaskSpec`, `TaskExit`, port `ProcessRunner`), `substrate-run-app` (run/cancel workflow, `LogSink`), `substrate-run-process` (spawn, process-group kill, `libc`; the only `unsafe`), `substrate-run-toml` (workflow file → domain) |
| `terminal` | terminal | `substrate-terminal-domain` (`TerminalSize`, `Shell`, `Utf8Chunker`, `SpawnOptions`), `substrate-terminal-pty` (`portable-pty` session, shell detection) |
| `project` | project-model | `substrate-project-domain` (`Solution`, `Project`, … ; ports `SolutionFormat`, `ProjectFormat`, already traits: products are the adapters), `substrate-project-app` (`Formats`, open/save) |
| `state` | ide-state | `substrate-state-app` (port `StateStore`), `substrate-state-json`; no domain crate (no rules: exception below) |
| `process` (`command`, host env, `open_link`/`open_path`/`reveal`) | host | `substrate-host-app` (port `Launcher`), `substrate-host-os` (clean-environment `Command`, launcher); also used by the three process-running adapters |
| `commands`, `events`, `bin/substrate-messages` | host-ui-contract | `substrate-host-contract` (interface: command and event catalog, TS codegen bin; it is the published language with `ui/`) |
| `tauri-plugin/` | interface | `tauri-plugin-substrate` (composition root; DTOs with `serde` move here from the 19 core files) |
| `sync` (`Locked`, `ReadWrite`) | none | dissolved: `lock().unwrap_or_else(PoisonError::into_inner)` where used |

Exceptions to record when the split lands: `tauri-plugin-substrate` keeps
Tauri's mandatory `tauri-plugin-<name>` name instead of `-tauri`;
`substrate-search-domain` depends on `regex` (pure computation, no I/O);
`ide-state` has no domain crate; `substrate-host-os` is an infrastructure
crate other adapters depend on (needs an ADR).

Draft `craftbook.toml`, today's file first (lets the checker run before the split):

```toml
[[crate]]
name = "substrate-platform"
layer = "infra"      # really mixed: kernel + domain + app + infra + interface (commands, events)
context = "substrate"
[[crate]]
name = "tauri-plugin-substrate"
layer = "interface"
context = "substrate"
```

Target file, replacing it wave by wave (inline tables, same structure as `[[crate]]`/`[[context_edge]]`):

```toml
crate = [
  { name = "substrate-message",          layer = "shared-kernel", context = "message" },
  { name = "substrate-workspace-domain", layer = "domain",    context = "workspace" },
  { name = "substrate-workspace-fs",     layer = "infra",     context = "workspace" },
  { name = "substrate-git-domain",       layer = "domain",    context = "source-control" },
  { name = "substrate-git-app",          layer = "app",       context = "source-control" },
  { name = "substrate-git-gitcli",       layer = "infra",     context = "source-control" },
  { name = "substrate-search-domain",    layer = "domain",    context = "search" },
  { name = "substrate-search-app",       layer = "app",       context = "search" },
  { name = "substrate-search-ignore",    layer = "infra",     context = "search" },
  { name = "substrate-run-domain",       layer = "domain",    context = "run" },
  { name = "substrate-run-app",          layer = "app",       context = "run" },
  { name = "substrate-run-process",      layer = "infra",     context = "run" },
  { name = "substrate-run-toml",         layer = "infra",     context = "run" },
  { name = "substrate-terminal-domain",  layer = "domain",    context = "terminal" },
  { name = "substrate-terminal-pty",     layer = "infra",     context = "terminal" },
  { name = "substrate-project-domain",   layer = "domain",    context = "project-model" },
  { name = "substrate-project-app",      layer = "app",       context = "project-model" },
  { name = "substrate-state-app",        layer = "app",       context = "ide-state" },
  { name = "substrate-state-json",       layer = "infra",     context = "ide-state" },
  { name = "substrate-host-app",         layer = "app",       context = "host" },
  { name = "substrate-host-os",          layer = "infra",     context = "host" },
  { name = "substrate-host-contract",    layer = "interface", context = "host-ui-contract" },
  { name = "tauri-plugin-substrate",     layer = "interface", context = "host-ui-contract" },
]
context_edge = [
  { from = "search",           to = "workspace", relation = "customer-supplier" },  # TextView
  { from = "source-control",   to = "workspace", relation = "customer-supplier" },  # TextView for diffs
  { from = "source-control",   to = "host",      relation = "customer-supplier" },  # clean-env Command
  { from = "run",              to = "host",      relation = "customer-supplier" },
  { from = "terminal",         to = "host",      relation = "customer-supplier" },
  { from = "host-ui-contract", to = "message",   relation = "published-language" }, # names the UI is generated from
  { from = "workspace",        to = "message",   relation = "shared-kernel" },
  { from = "source-control",   to = "message",   relation = "shared-kernel" },
  { from = "search",           to = "message",   relation = "shared-kernel" },
  { from = "run",              to = "message",   relation = "shared-kernel" },
  { from = "terminal",         to = "message",   relation = "shared-kernel" },
  { from = "project-model",    to = "message",   relation = "shared-kernel" },
  { from = "ide-state",        to = "message",   relation = "shared-kernel" },
  { from = "host",             to = "message",   relation = "shared-kernel" },
]
```

`tauri-plugin-substrate` is the composition root and depends on every
context's app and adapter crates; the checker must treat the interface layer
as exempt from context edges (§7.3), or the edges must be listed.

Inside source-control the gitcli adapter is the anticorruption layer for
git's porcelain (§7.6); git's text never reaches `substrate-git-domain`.
Free functions taking `root` stay legal in `-gitcli` (§2.3 rule 5,
`git::status::read(root)`), but the parsing they do today moves into
`porcelain/` and returns domain types.

## 4. Migration plan

Each wave is one PR (or one per named area) and leaves `cargo test --all-features` green.

| Wave | Content | Risk |
|---|---|---|
| **0. Gates, no code change** | Add `craftbook.toml` (today block), `clippy.toml` thresholds, `[workspace.lints]` as **warn**; CI runs `--all-features` and records coverage 68.57 / 68.00 %; fix or `#[ignore]` with an issue the grandchild test; add `docs/glossary.md`, `docs/context-map.md` (from §3); fix `CONTRIBUTING.md` (§5) | none |
| **1. Renames** | `*Info` → `DirectoryEntry`, `Branch`, `Commit`, `StashEntry`, `Shell`; `git::status::status` → `read`; `register_text_view`/`text_view` → `register`/`find`; `list_dir`/`create_dir` → `dir::{list, create}`; `run_search`/`run_replace` → `search::find`/`replace`; `workflow::Workflow*` → `workflow::{File, Definition, Step, Outcome, Handle}`; `search::ProjectEntry` → `WalkEntry`; `ws`/`cmd`/`m`/`Res` spelled out; 7 `bool`-parameter fns → enums (`CommitMode`, `BranchSource`, `DeleteMode`, …). UI copy regenerated in the same PR. | low, mechanical |
| **2. Files and functions** | `run/task/mod.rs` → `task.rs`, `spawn.rs`, `kill_tree.rs`; `git/status.rs` → `status/{repo_status.rs, change_kind.rs, operation.rs, porcelain.rs, commands.rs, tests.rs}`; `search/mod.rs` logic → `layout.rs`, `search_query.rs`, `matcher.rs`, `file_mask.rs`, named re-exports; `project/model.rs` → one type per file; `workflow.rs` tests → `tests.rs`; fold `tauri-plugin` single-file dirs; split the 4 long production fns and the 2 six-parameter fns (`SearchPlan`, `SpawnOptions`), the 5 deep nests; `tauri-plugin` `init_with` into one registration fn per feature | low |
| **3. Errors** | `thiserror`; `WorkflowParseError`, `PtyError` replace the 5 `anyhow` signatures; `GitError` (`GitMissing`, `NotARepository`, `AuthenticationRequired`, `IdentityMissing`, `CommandFailed`) from the existing `AUTH_SIGNS`/`IDENTITY_SIGNS`/`NO_REPO_SIGNS`, with `From<GitError> for Message` at the interface; same per area for the other 64 `Message` results; decide the 15 discards; `# Errors` docs (68) | medium: touches every signature |
| **4. Globals** | `text_view::VIEWS` → a `TextViews` value built by the host and passed to search/diff; `WRITE_LOCK` → a lock owned by the plugin's state; `VIEW_SEQ` → `tempfile` or an owned counter | medium |
| **5. Kernel + contract out** | Extract `substrate-message` and `substrate-host-contract` (`commands`, `events`, codegen bin; `build.rs` reads it); no compatibility re-exports (CONTRIBUTING: old API replaced) | medium |
| **6. Source control** (top offender: 971 lines, 45 % covered) | `substrate-git-{domain,app,gitcli}`; ports + `InMemory*` fakes + contract suite; porcelain parsers fixture-tested; `Operation::in_progress` I/O moves to gitcli | high |
| **7. Search** | `substrate-search-{domain,app,ignore}`; `SearchQuery` bools → options/enums | medium |
| **8. Run** | `substrate-run-{domain,app,process,toml}`; `ProcessRunner` fake; `#![forbid(unsafe_code)]` everywhere but `-process` | medium |
| **9. Terminal** (29 % covered) | `substrate-terminal-{domain,pty}`; PTY contract test on Linux CI | medium |
| **10. Workspace, project, state, host** | the remaining crates; delete `substrate-platform` | medium |
| **11. Boundary + gates** | `serde` derives (47) leave domain crates for DTOs in the plugin; `missing_docs` to zero; lints **deny**; per-layer coverage thresholds | medium |

Waves 1–4 change the code inside the existing crate, so they need no
dependency work and each can be reviewed on its own. The split starts only
once names and errors are right, so no crate is born with a wrong public API.

## 5. `CONTRIBUTING.md` against the standard

| CONTRIBUTING says | Standard | Recommendation |
|---|---|---|
| "Code reads like the code around it: same naming, idioms and comment density." | Chapters 1–5 decide; the code around is what gets migrated | Replace with a link to Craftbook and this page; "match the surroundings" only where Craftbook is silent |
| "Make the change, with tests where it makes sense." | §8.3, §8.6: every public fn and error variant tested, regression test per bug fix, ≥ 90 % on changed lines | Replace with the definition of done (§8.6) |
| Check: `cargo test` (default features) and `npm test` | §8.5: fmt, clippy `-D warnings`, craftcheck, `test --workspace --all-features`, llvm-cov, `cargo deny`, `cargo doc` | List the gates, introduced by wave (warn first, then deny) |
| "Rust is formatted with `cargo fmt`" (no `rustfmt.toml`) | Template `rustfmt.toml`, `group_imports = "StdExternalCrate"` | Adopt the template when it exists in Craftbook (see below) |
| "An outdated API is replaced, not kept alongside the new one." | Consistent: rules are not kept in old versions; renames migrate per this page | Keep: it lets waves 1–5 rename without shims |
| Conventional Commits `feat(scope)` | Silent | Keep; use context names as scopes (`source-control`, `search`, `run`, …) |
| "Substrate stays generic: no Yog-specific formats or logic." | §7.2–7.3: product formats are adapters of ports | Keep, and state it in `docs/context-map.md` as "project formats are product adapters of `SolutionFormat`/`ProjectFormat`" |
| Bug report asks for "Minecraft version and loader (Fabric, NeoForge or Forge)" | §5.4: docs current | Stale text from another project: remove |

## 6. Notes on this audit

- `craftcheck` on today's tree, with the single-crate map in
  [`substrate-platform/craftbook.toml`](substrate-platform/craftbook.toml):
  `check` 30 errors, 41 warnings (C103 ×20 items in wiring files, C108 ×19,
  E403 ×19, N202 ×5, E404 ×2 single-line signatures — the multi-line ones above
  are found by review, A501 ×2, C102 ×1, C101 warnings ×3); `deps` 0, as
  expected before the split. After wave 5 the target map above replaces it.
- Pending exceptions (none granted yet): Tauri plugin crate name, `regex` in
  `substrate-search-domain`, no `ide-state` domain crate, `substrate-host-os`
  as a shared adapter. `#[coverage(off)]`: none used.
