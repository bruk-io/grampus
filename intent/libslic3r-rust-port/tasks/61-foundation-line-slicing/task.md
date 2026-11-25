---
kind: task
step: 20
delivers: []
depends: [55-foundation-libslic3r-execution, 56-foundation-point, 57-foundation-utils, 58-foundation-bounding-box, 59-foundation-config, 60-foundation-trees-grids, 34-foundation-spike, 29-ffi-bridge-core, 30-ffi-bridge-libraries, 31-proof-harness, 32-proof-dumps-fuzz, 33-port-surface]
size: 5
---
# Task: Design Line, Slicing and the AABB tree over lines

## What to build

This is one of the 33 tasks cut from step 20 of `intent/libslic3r-rust-port/plan.md`, "Create the crates and hand-design the foundation types", and a piece of a split step: its `delivers` is empty because the whole step is groundwork that delivers no requirement by itself — steps 21, 22 and 23 need it. The step's first task was the spike `34-foundation-spike`, cut in an earlier pass: it added `port/crate-skeletons` and `just crate-skeletons`, committed a `Cargo.toml` and a doc-comment-only `src/lib.rs` for every crate `port/cut.tsv` names, corrected the manifest's `template-heavy` column, and committed `port/foundation.tsv` — the in-scope `template-heavy` rows in crate-level order then path, batched to at most 4 units or 2,500 source lines. The plan cuts **one task per row of that file**, and this is one of them.

**This task is the `foundation-10` row of `port/foundation.tsv`**: 3 units, 2,133 source lines, unit paths `Line.cpp, Line.hpp, Slicing.cpp, Slicing.hpp, AABBTreeLines.hpp`.

The template-heavy and header-only files are not file-units: they are type-and-trait design, every other unit depends on their shape, and so they are designed once, by hand, before the Phase A fan-out, as `docs/WORKFLOW.md` "Unit of work" states. That is what this task does for its own rows and nothing else.

### The rules every foundation task follows

This block is identical in all 33 tasks cut from step 20. It is the part a builder must not get
wrong; everything after it is this task's own rows.

**Read in full before starting**: step 20 of `intent/libslic3r-rust-port/plan.md`, "Create the
crates and hand-design the foundation types"; `docs/WORKFLOW.md`'s sections "Unit of work", "The
two phases", "The loop", "The rejection rule" and "Conflict discipline"; and `docs/PORTING.md`'s
"Crate map", "Type map", "Idiom map", "Don't-translate list", "Banned-substitution list" and its
rules "Scaling conversions", "Float widths", "Rounding modes", "Error sets", "Number formatting"
and "Linked libraries". This block names the rules from them a builder must not miss; it does not
replace them.

- **These modules are ported, not drafted.** Step 20's words: each compiles, carries its
  `PORT STATUS` trailer, has its `mod` line, has its R19 proof, and has its differential fuzz
  target where its manifest row says `required`. A Phase A draft with no `mod` line is not what
  this step delivers.
- **One module per unit, at the path the manifest already names.** Write
  `crates/<crate>/src/<module>.rs` at exactly the `target crate` and `target module` cells of
  `port/manifest.tsv`, listed per unit below. Create no crate and invent no path: task
  `34-foundation-spike` committed every `crates/<crate>/Cargo.toml` and a `src/lib.rs` holding
  the crate's doc comment and no `mod` line.
- **Add each `mod` line at its alphabetical position** in the crate's one block of `mod` lines,
  per `docs/WORKFLOW.md` "Conflict discipline", so two branches conflict textually rather than
  reordering silently. The crate's `lib.rs` is the one shared file a port task writes.
- **The six markers are a closed list**: `// TODO(port):`, `// PERF(port):`, `// PORT NOTE:`,
  `// SAFETY:`, `// PORT SRC:`, `// FIDELITY(port):`. Every transcribed numeric constant carries
  a `// PORT SRC: <vendored path>:<line>` whose line really holds that literal; every `unsafe`
  block carries a `// SAFETY:`; and every order choice — a fixed summation order, a total-order
  comparator, an insertion-ordered map whose iteration reaches output — carries a
  `// FIDELITY(port):`, which is what puts the site in `port/reports/divergences.md`.
- **Every module ends with a `PORT STATUS` trailer** naming its source path,
  `confidence: high|medium|low`, its equality tier and the oracle that proves it.
- **Equality tier.** Every row of this step is `equality tier: exact` with the `tolerance` and
  `comparison function` cells empty, so every proof here is exact equality with
  no tolerance chosen in any of these 33 tasks — of integer-coordinate output where the unit
  produces coordinates, and of the unit's own output where it produces none. A unit that
  cannot be held to exact equality is a finding for the pull request, not a tolerance invented on
  the branch.
- **The proof paths.** `port/manifest.tsv`'s `proof path` column is 1 = FFI to the library built
  from the pin at `vendor/OrcaSlicer/build/arm64`, which is built with `-ffp-contract=off` and is
  therefore the oracle C2's decision makes normative; 2 = the committed stage dumps, read through
  `crates/slic3r-proof/src/dumps.rs`; 3 = a test translation unit inside the FFI crate,
  `crates/slic3r-ffi/cxx/include_vendored.cpp`, that `#include`s the vendored `.cpp` without
  modifying it, for code with no external linkage to call.
- **A proof of path 1 or path 3 needs its own shim, and the files it goes in are shared.** The
  test calls the oracle through the `cxx` bridge, so for each unit it proves this task appends
  **one shim** to `crates/slic3r-ffi/cxx/proof_shims.cpp` and **one bridge declaration** to
  `crates/slic3r-ffi/src/proof.rs`. `31-proof-harness` created both files, with two of each as
  the harness's own proof, and no other task adds any: they are files a port task writes, not
  files it must leave alone. A proof-path-3 unit additionally appends its `#include` of the
  vendored `.cpp` to `crates/slic3r-ffi/cxx/include_vendored.cpp`, and only where that satisfies
  the duplicate-symbol rule that file's header comment states — a vendored `.cpp` whose
  internal-linkage helpers are the thing being proven, with the shim exposing the helper rather
  than re-exporting the file's own surface. Append at the end of each of the three files, so two
  branches conflict textually rather than reordering silently, and change no other line of any
  of them: up to 31 sibling foundation tasks append to the same three. `crates/slic3r-ffi/src/lib.rs`
  already carries its `mod proof;` line, so no foundation task edits it, and
  `crates/slic3r-ffi/Cargo.toml` stays on the must-not list.
- **How a proof is recognised, and why a unit can need two.** `port/port-status`'s
  `proof_present` looks for a `// PROOF: <the proof path cell, verbatim>` line beside a test
  function whose name contains the module's snake_case file stem, either in a file under
  `crates/<crate>/tests/` or in a `#[cfg(test)]` module inside the module file itself. It matches
  **that row's own cell**, never a sibling row's, and `derive_row_status` runs per manifest row —
  so a unit whose `.cpp` row and `.hpp` row carry different cells needs **one `// PROOF:` marker
  for each cell** before both of its rows derive `ported`. `port/inventory` seeds the cell per row
  (path 3 from the `.cpp`'s internal linkage, path 2 from a row's own pipeline stage, path 1
  otherwise), which is why a unit's two rows can and do disagree — roughly 56 of the manifest's
  232 in-scope target modules are in that position. Each unit's cells, and the markers it needs,
  are listed below. `// PROOF:` is **not** one of the six port markers: it is exempt only in
  those two positions, and a stray `// PROOF:` anywhere else in a `src/` file is an ordinary
  marker-whitelist failure.
- **A module is `ported` only when `port/port-status` says so.** `derive_row_status` reads
  `ported` for a row whose target file a `mod` line reaches, which holds no live `todo!()` or
  `unimplemented!()`, and whose `// PROOF:` marker is present; `drafted` when no `mod` line
  reaches the file but it carries a `PORT STATUS` trailer naming the row, or when the proof is
  still missing; and `not-started` for anything else — **including a `mod`-reached file that
  still holds a live `todo!()`**, which is indistinguishable in the burn-down from a module
  nobody has touched. So never land a `mod` line over a `todo!()`: leave the module unreached by
  any `mod` line instead, which is `drafted`.
- **No stand-in for the code under test or for the oracle.** `just port-check` fails a file
  under `crates/*/tests/` or a `#[cfg(test)]` module in a crate holding a ported module when it
  declares or names an identifier matching `mock`, `stub`, `fake` or `dummy` in any case, and
  there is no allowlist.
- **A raw-pointer field's Rust type comes from `port/ownership.tsv` verbatim**, matched by
  header path and line. A module never invents one. The rows that apply to this task are listed
  below; where a unit has none, it has no raw-pointer field at the pin.
- **Faithfulness, not idiom.** This intent leaves Rust idioms and optimisation to the following
  one, so take the simplest translation that keeps the C++ meaning, not the nicest. A workaround
  that needs a paragraph of justification is wrong code: re-read the vendored source and port what
  it does (`docs/WORKFLOW.md` "The rejection rule").
- **The review loop.** One implementer, then two adversarial reviewers in separate contexts, then
  one fixer. Each reviewer reads the diff **and** the vendored source, is told to assume the
  module is wrong, cites every divergence as `source:line` against `module:line`, and may not
  implement. Both verdicts go in the pull request body with each unit's manifest row. The
  recipe that checks such a body, `just pr-body-check <file>`, is added by `36-pr-body-check` of
  plan step 21, which this step does not wait on and which may land after this task — plan step
  20 waits on steps 5, 7, 11, 15, 17, 18 and 19, none of them step 21. So write the body in the
  form that recipe checks, and run `just pr-body-check <file>` on it only once
  `port/port-status --pr-body` is on `main`; until then the body recording both verdicts is what
  is asked. That is the same condition `35-trial-run` states from the other side.
- **A `required` fuzz target** goes at `crates/slic3r-ffi/fuzz/fuzz_targets/<crate>_<module>.rs`,
  generates arbitrary `Point`, `Polygon` and `ExPolygon` inputs, and asserts exact equality of
  integer-coordinate output between the Rust module and the same call through FFI. Registering it
  needs one `[[bin]]` block appended to the end of `crates/slic3r-ffi/fuzz/Cargo.toml` — that
  file is the only crate manifest a foundation task may touch, and appending keeps two branches
  conflicting textually. `just fuzz <target>` runs it, and `just port-status` fails a `required`
  unit with no target naming its module.
- **Files this task must not write.** Each is produced by the repository, and writing it by hand
  is what `docs/WORKFLOW.md` "Conflict discipline" forbids: `port/manifest.tsv`, `port/cut.tsv`,
  `port/foundation.tsv`, `port/ownership.tsv`, `port/ownership-batches.tsv`, `port/inventory`,
  `port/cut-tasks`, `port/crate-skeletons`, `port/ownership`, `port/port-status`,
  `port/audit-order`, `port/regenerate`, everything under `port/reports/`, the root `Cargo.toml`
  (a `crates/*` glob, so a new crate needs no edit), `Cargo.lock`, the `justfile`,
  `.asdlc/checks`, `.github/workflows/ci.yml`, everything under `docs/`, and every
  `crates/<crate>/Cargo.toml` other than the fuzz crate's — no foundation task adds a dependency,
  because `34-foundation-spike` generated each crate's path dependencies from `port/cut.tsv`'s
  `dep` rows and a dev-dependency on `slic3r-proof`.
- **An included in-scope file whose manifest row is `template-heavy: no` is a fan-out module**,
  whatever crate it targets — including a crate another foundation task owns. Its Rust module is
  written by Phase A and Phase B, not here and not by a sibling foundation task, so it appears in
  neither dependency list below. Where a translation needs an item from one, leave a
  `// TODO(port): <vendored path>:<line>` marker rather than a guess and name it in the pull
  request.
- **Write only your own modules.** This task writes the module files listed below, its crates'
  `lib.rs` `mod` lines, its own tests under `crates/<crate>/tests/`, its own fuzz targets, its
  own appended lines in the four shared files of `crates/slic3r-ffi` named above —
  `cxx/proof_shims.cpp`, `src/proof.rs`, `cxx/include_vendored.cpp` and `fuzz/Cargo.toml` — and
  its own plan where it has one. Thirty-two sibling foundation tasks are built at the same time
  in worktrees of their own, so a file that is not yours is how they collide.
- **Nothing under `vendor/OrcaSlicer/` changes** and the submodule pointer stays at
  `f92a7c065c5718b8ef1baac1cd704df2911238f4`.
- **The branch adds no file under `intent/`** other than this task's own plan where it has one,
  and changes none, so `just guard` passes.
- **Rebase on `main` before opening the pull request, and again whenever `main` moves**, since
  the tasks this one depends on merge while it is open.
- **The oracle has to exist before any proof runs.** A builder's worktree can arrive with the
  submodule uninitialised, so start with `git submodule update --init vendor/OrcaSlicer`, which
  checks out the pin and leaves `git submodule status` showing
  `f92a7c065c5718b8ef1baac1cd704df2911238f4`, and then `just oracle-build`, which produces the
  library at `vendor/OrcaSlicer/build/arm64` that a proof-path-1 test links and whose link flags
  a proof-path-3 translation unit is compiled with. Neither step is a commit and neither changes
  a tracked file.
- **How to check it**: `cargo build --workspace`, `cargo test --workspace`, `cargo fmt --check`,
  `just port-check` (which includes `just port-status --per-commit` and the per-marker
  assertions), `just inventory-check`, `just ownership-check`, `just check` and `just guard`. Do
  **not** run `just audit-check` or the perf-backlog whole-file diff: those belong to the
  regeneration pass on `main`, which concurrent branches could not each pass.

### The units this task ports

One `.cpp` plus its same-stem `.hpp` is one unit and one Rust module, at the `target crate` and `target module` path of `port/manifest.tsv`. A header with no same-stem `.cpp` in scope is a unit on its own. Line counts are the manifest's `lines` cells.

**`crates/slic3r-line/src/line.rs`** — crate `slic3r-line`, crate-level level 5.

- `vendor/OrcaSlicer/src/libslic3r/Line.cpp`, 150 lines; `template-heavy: no`, `proof path: 1`, `fuzz target: required`, pipeline stage `none`.
- `vendor/OrcaSlicer/src/libslic3r/Line.hpp`, 341 lines; `template-heavy: yes`, `proof path: 2`, `fuzz target: required`, pipeline stage `posSlice`.
- 491 source lines in the unit.
- Proof path **1** (the `Line.cpp` row): FFI to the library built from the pin at `vendor/OrcaSlicer/build/arm64`, through `crates/slic3r-ffi`.
- Proof path **2** (the `Line.hpp` row): the committed stage dumps, read through `crates/slic3r-proof/src/dumps.rs`.
- The unit's two rows carry different `proof path` cells, so it needs **two `// PROOF:` markers** — `// PROOF: 1` and `// PROOF: 2` — beside tests naming `line`, or one of its two rows never derives `ported`.
- `fuzz target: required`, so this unit needs `crates/slic3r-ffi/fuzz/fuzz_targets/slic3r-line_line.rs` and a `[[bin]]` block for it; `just fuzz slic3r-line_line` runs it.

**`crates/slic3r-slicing/src/slicing.rs`** — crate `slic3r-slicing`, crate-level level 5.

- `vendor/OrcaSlicer/src/libslic3r/Slicing.cpp`, 1,028 lines; `template-heavy: no`, `proof path: 1`, `fuzz target: n/a`, pipeline stage `none`.
- `vendor/OrcaSlicer/src/libslic3r/Slicing.hpp`, 235 lines; `template-heavy: yes`, `proof path: 2`, `fuzz target: n/a`, pipeline stage `posSlice`.
- 1,263 source lines in the unit.
- Proof path **1** (the `Slicing.cpp` row): FFI to the library built from the pin at `vendor/OrcaSlicer/build/arm64`, through `crates/slic3r-ffi`.
- Proof path **2** (the `Slicing.hpp` row): the committed stage dumps, read through `crates/slic3r-proof/src/dumps.rs`.
- The unit's two rows carry different `proof path` cells, so it needs **two `// PROOF:` markers** — `// PROOF: 1` and `// PROOF: 2` — beside tests naming `slicing`, or one of its two rows never derives `ported`.

**`crates/slic3r-aabbtreelines/src/aabb_tree_lines.rs`** — crate `slic3r-aabbtreelines`, crate-level level 6.

- `vendor/OrcaSlicer/src/libslic3r/AABBTreeLines.hpp`, 379 lines; `template-heavy: yes`, `proof path: 2`, `fuzz target: n/a`, pipeline stage `posSlice`.
- 379 source lines in the unit.
- Proof path **2** (the `AABBTreeLines.hpp` row): the committed stage dumps, read through `crates/slic3r-proof/src/dumps.rs`.
- One cell across the unit's row, so one `// PROOF: 2` marker beside a test naming `aabb_tree_lines`.

### Raw-pointer fields

None of this task's units declares a raw-pointer field or a container of raw pointers at the pin, so no row of `port/ownership.tsv` applies and no module here invents a Rust type for one.

### What it waits for

`depends` names the foundation tasks whose modules this one needs, plus the five tasks plan step 20 itself waits on for steps 17, 18 and 19, which step 20 takes whole rather than per task — so a task that reaches neither the stage-dump reader nor a fuzz target still names `32-proof-dumps-fuzz`. It is derived from two committed files, by the rule all 33 foundation tasks use:

- **Across crates**: for each crate this task builds, the crates `port/cut.tsv`'s `dep` rows name for it, kept only where one of this task's own vendored files `#include`s a file whose manifest row targets that crate. The `dep` rows are the feedback-arc-cut graph plan step 11 computes — the module-granularity include graph is one cycle and the cut is what makes it an order — so an include the cut cut is not a dependency.
- **Inside a crate shared with a sibling foundation task**: the sibling whose module one of this task's headers includes directly.
- **Uniformly**: `34-foundation-spike`, which committed the crates and `port/foundation.tsv`; `29-ffi-bridge-core` and `30-ffi-bridge-libraries`, the FFI bridge every proof path reaches the oracle through; `31-proof-harness` and `32-proof-dumps-fuzz`, the proof harness, its tiers, the stage-dump reader and the fuzz crate; and `33-port-surface`, the port's slicing surface. Those five are plan step 20's "waits on Steps ... 17, 18 and 19".

The foundation tasks this task's own files reach across crates, with the module each one owns that is needed here:

- `55-foundation-libslic3r-execution` — `crates/slic3r-libslic3r/src/libslic3r.rs`.
- `56-foundation-point` — `crates/slic3r-point/src/point.rs`.
- `57-foundation-utils` — `crates/slic3r-utils/src/utils.rs`.
- `58-foundation-bounding-box` — `crates/slic3r-boundingbox/src/bounding_box.rs`.
- `59-foundation-config` — `crates/slic3r-config/src/config.rs`.
- `60-foundation-trees-grids` — `crates/slic3r-aabbtreeindirect/src/aabb_tree_indirect.rs`.

This task's files also reach `slic3r-slicingadaptive`, a crate whose modules the Phase A and Phase B fan-out writes. The generated `Cargo.toml` already carries the path dependency, so the crate resolves; where a translation needs an item from it, leave a `// TODO(port): <vendored path>:<line>` marker rather than a guess and name it in the pull request.

**`35-trial-run` is not a dependency.** Plan step 21 waits on step 20, not the other way round: the three-unit trial is run after the foundation types are in, and it is the *fan-out* steps 22 and 23 that wait on it. Do not wait for it.

### Built beside the other foundation tasks

No other foundation task writes a module of `slic3r-line`, `slic3r-slicing`, `slic3r-aabbtreelines`, so the files this task shares with a sibling are `crates/slic3r-ffi/fuzz/Cargo.toml`, where it appends its `[[bin]]` block, and the `crates/slic3r-ffi` files named below — append at the end of each so two branches conflict textually. Do not touch a sibling's module file.

This task also appends to two files of `crates/slic3r-ffi` that every foundation task proving a
path-1 or path-3 unit appends to: one shim per proven unit at the end of
`crates/slic3r-ffi/cxx/proof_shims.cpp`, and one `cxx` bridge declaration per proven unit at the
end of `crates/slic3r-ffi/src/proof.rs`. Append at the end of each and change no other line, so
two branches conflict textually rather than reordering silently.

### Why this is a 5, and why it is not split

Three units, 2,133 lines, across three crates at two cut levels. `Line` carries a `required` fuzz target, `Slicing.cpp` is 1,028 lines of layer-height arithmetic where `docs/PORTING.md`'s "Float widths" and "Rounding modes" rules decide the answer, and `AABBTreeLines.hpp` is a template over the tree the previous row designed. Not split: one row is one task.

## Done when

- `crates/slic3r-line/src/line.rs` is committed, ports `Line.cpp`, `Line.hpp`, ends with a `PORT STATUS` trailer naming its source path, its `confidence`, `exact` as its equality tier and the oracle that proves it, and a `mod line;` line in `crates/slic3r-line/src/lib.rs` reaches it at its alphabetical position.
- `crates/slic3r-slicing/src/slicing.rs` is committed, ports `Slicing.cpp`, `Slicing.hpp`, ends with a `PORT STATUS` trailer naming its source path, its `confidence`, `exact` as its equality tier and the oracle that proves it, and a `mod slicing;` line in `crates/slic3r-slicing/src/lib.rs` reaches it at its alphabetical position.
- `crates/slic3r-aabbtreelines/src/aabb_tree_lines.rs` is committed, ports `AABBTreeLines.hpp`, ends with a `PORT STATUS` trailer naming its source path, its `confidence`, `exact` as its equality tier and the oracle that proves it, and a `mod aabb_tree_lines;` line in `crates/slic3r-aabbtreelines/src/lib.rs` reaches it at its alphabetical position.
- `crates/slic3r-line/src/line.rs` carries a `// PROOF:` marker for each of its manifest rows' `proof path` cells — `// PROOF: 1` and `// PROOF: 2` — beside a test naming `line`, each proof held to exact equality with no tolerance — of integer-coordinate output where the unit produces coordinates, and of the unit's own output where it produces none — and `cargo test --workspace` passes with them.
- `crates/slic3r-slicing/src/slicing.rs` carries a `// PROOF:` marker for each of its manifest rows' `proof path` cells — `// PROOF: 1` and `// PROOF: 2` — beside a test naming `slicing`, each proof held to exact equality with no tolerance — of integer-coordinate output where the unit produces coordinates, and of the unit's own output where it produces none — and `cargo test --workspace` passes with them.
- `crates/slic3r-aabbtreelines/src/aabb_tree_lines.rs` carries a `// PROOF: 2` marker — its manifest row's one `proof path` cell — beside a test naming `aabb_tree_lines`, the proof held to exact equality with no tolerance — of integer-coordinate output where the unit produces coordinates, and of the unit's own output where it produces none — and `cargo test --workspace` passes with it.
- `crates/slic3r-ffi/fuzz/fuzz_targets/slic3r-line_line.rs` is committed, is registered by one `[[bin]]` block appended to `crates/slic3r-ffi/fuzz/Cargo.toml`, asserts exact equality of integer-coordinate output between `crates/slic3r-line/src/line.rs` and the same call through FFI, and `just fuzz slic3r-line_line` runs it.
- `just port-status` reports all three units of this task as `ported` — a `mod` line reaches each module, none holds `todo!()` or `unimplemented!()`, and each `// PROOF:` marker's proof passes.
- Every numeric constant transcribed from the vendored source carries a `// PORT SRC: <vendored path>:<line>` marker whose line really holds that literal; every `unsafe` block carries a `// SAFETY:`; every order choice carries a `// FIDELITY(port):`; and no marker outside the closed list of six appears.
- `cargo build --workspace`, `cargo test --workspace`, `cargo fmt --check` and `just port-check` all pass on the rebased branch.
- `just check` and `just guard` pass, and `just inventory-check` and `just ownership-check` still pass.
- `git diff main` touches only: this task's module files, the `lib.rs` of `slic3r-line`, `slic3r-slicing`, `slic3r-aabbtreelines`, this task's own test files, and its fuzz target and the `[[bin]]` block registering it. It also appends to `crates/slic3r-ffi/cxx/proof_shims.cpp` and `crates/slic3r-ffi/src/proof.rs`, and changes no other line of either.
- `port/manifest.tsv`, `port/cut.tsv`, `port/foundation.tsv`, `port/ownership.tsv`, every script under `port/`, everything under `port/reports/`, the root `Cargo.toml`, `Cargo.lock`, the `justfile`, `.asdlc/checks`, `.github/workflows/ci.yml` and everything under `docs/` are unchanged.
- Nothing under `vendor/OrcaSlicer/` changed and `git submodule status` still shows `f92a7c065c5718b8ef1baac1cd704df2911238f4`.
- The pull request body records both adversarial review verdicts, each citing `source:line` against `module:line`, and names each of this task's manifest rows; `just pr-body-check` passes on it.
