---
kind: task
step: 18
delivers: [R19]
depends: [31-proof-harness, 26-stage-dump-seams, 29-ffi-bridge-core, 11-fixture-set]
size: 5
---
# Task: Prove units against the stage dumps and add the differential fuzz crate

## What to build

This is **part 2 of plan step 18, "The per-unit proof harness"**, which delivers R19; `delivers`
names the whole step because the step is split. Part 1 is task `31-proof-harness`: the
`crates/slic3r-proof` crate, `src/tiers.rs`, proof paths 1 and 3, and the mock check. Part 2 is
proof path 2 and the `cargo-fuzz` crate. Read plan step 18, plan step 8 and spec R19 before
starting.

**Proof path 2 — the committed stage dumps.** Add `crates/slic3r-proof/src/dumps.rs`, reached by a
`mod dumps;` line in `crates/slic3r-proof/src/lib.rs`, which is the proof path a unit uses for a
whole pipeline step. It reads the dumps `just oracle-dump <fixture> <configuration>` writes
(tasks `25-stage-dumper` and `26-stage-dump-seams`): one dump per fired seam, line-oriented text
carrying step 3's provenance header as a `;` comment block, holding **the fields the dump format
section of `oracle/stage-dump/README.md` names**, which task `26-stage-dump-seams` writes. Read that
section for the field list rather than a list here: at the pin it covers per-layer and per-region
state, per-support-layer `support_islands` and `support_fills`, and, for the two print-wide seams,
`Print::skirt()`, `Print::skirt_brim_groups()`, every object's `object_skirt()` and
`Print::wipe_tower_data()` — and a list copied into this task would be a second place that can
drift from the writer this parser is held to.

Three things the reader has to get right, each of which silently weakens a proof otherwise:

- **It parses every coordinate as an integer, never through a float.** `coord_t` is `int64_t` at
  the pin (`vendor/OrcaSlicer/src/libslic3r/libslic3r.h:45-48`, where the `int32_t` alternative is
  inside `#if 0`), so a coordinate above 2^53 is not representable in `f64` and a float round-trip
  would lose exactly the exactness an `exact` tier asserts. `dump_seams.cpp` writes integers for
  the same reason.
- **It refuses a dump whose provenance header is missing or disagrees with the pin**, with a named
  error naming the file, rather than comparing against a stale dump. That is the same judgement
  `oracle/provenance --check` makes (plan step 3), which covers every stage dump.
- **It takes the dump directory as an argument** and does not hardcode a path. The committed
  location is `port/dumps/<fixture-stem>-<configuration>/<seam>.dump`, the path task
  `05-oracle-provenance` fixes and task `25-stage-dumper` writes to, and the argument defaults to
  it. Read a committed dump rather than inventing a format here — the writer is `dump_seams.cpp`
  and this parser is held to it.

**This task commits one dump, and one hand-written fixture, and says which is which.** The dumps
path needs a committed dump to read at all, and `25-stage-dumper` commits the dumps of its own proof
run, so the test reads
`port/dumps/20mm_cube-base-fff/<seam>.dump` from the tree. The 2^53 case cannot come from a real
dump — a 256 by 256 mm bed at `SCALING_FACTOR` gives coordinates around 2.6e8, nowhere near 2^53 —
so it gets an **explicitly hand-written** fixture at
`crates/slic3r-proof/tests/data/oversized-coordinate.dump`, committed with a comment saying it is
hand-written for exactly that case and is not an oracle output. That fixture is the one exception to
"read a committed dump rather than inventing a format", it carries no provenance header, and it is
not under `port/dumps/`, so `oracle/provenance --check` does not cover it.

It exposes the parsed per-seam state keyed by seam, object, layer and region, so that a unit whose
manifest row names `proof path` 2 compares its stage output against the dump at the tier
`crates/slic3r-proof/src/tiers.rs` applies (task `31-proof-harness`). The oracle for a stage dump
is always the locally built library, which C2's decision makes normative for every per-stage
proof, so this path has no oracle to choose. This task commits the harness's own proof of the
path: a test that reads a committed dump and compares one seam's state through `src/tiers.rs`.

**The fuzz crate.** Add `crates/slic3r-ffi/fuzz/Cargo.toml` and
`crates/slic3r-ffi/fuzz/fuzz_targets/`, a `cargo-fuzz` crate, and the `just fuzz <target>` recipe
that runs one of its targets.

- `fuzz/Cargo.toml` carries its own `[workspace]` table, so the crate is its own workspace: the
  root `Cargo.toml` is `members = ["crates/*"]` (plan step 5) and is **not** edited, and
  `cargo test --workspace` — the line task `24-port-check-ci` puts in the macOS job — does not try
  to build it.
- `Arbitrary` implementations for the inputs R19's targets take: a point, a polygon and a polygon
  with holes, over 64-bit integer coordinates, with a generated polygon holding at least three
  points. **They are newtypes defined inside `crates/slic3r-ffi/fuzz`** — `ArbPoint`, `ArbPolygon`
  and `ArbExPolygon`, deriving `Arbitrary` over `i64` coordinates — which convert into the bridge's
  integer path representation at the target's entry. They cannot be `impl Arbitrary for` a
  `slic3r-ffi` type: both the trait and the type would be foreign to the fuzz crate, which is an
  orphan-rule error, and task `29-ffi-bridge-core` promises one boolean operation and one offset on
  integer paths rather than a named point or polygon type, so there may be no such type to implement
  it for. No ported module exists at this step either; task `34-foundation-spike`'s fan-out is where
  the ported `Point` lands.
- **The committed example target**, which drives the FFI side alone: it round-trips arbitrary
  polygons through the bridge and asserts the coordinates come back unchanged. That is what makes
  the crate, its `Arbitrary` implementations and the recipe real rather than described. Name it so
  it cannot be read as a unit's target — Step 14's `port/port-status` looks for
  `fuzz/fuzz_targets/<crate>_<module>.rs` naming a module, so the example's name must match no
  `<crate>_<module>` pair in `port/manifest.tsv` — and put the shape a qualifying unit's
  target follows in its doc comment: one target per qualifying unit at
  `fuzz/fuzz_targets/<crate>_<module>.rs`, generating arbitrary points and polygons and asserting
  exact equality of integer-coordinate output between the Rust module and the same call through
  FFI. Plan step 20 commits the first truly differential target, against its first foundation
  module.
- `just fuzz <target>` runs one target. `cargo-fuzz` needs its sanitizer flags, so if the
  toolchain `rust-toolchain.toml` pins cannot run it the recipe names the toolchain it uses
  explicitly in the recipe — `rust-toolchain.toml` is not changed, because plan step 16 installs
  that version in CI and plan step 5 owns the file.
- **No line is added to `.github/workflows/ci.yml`.** No step of this plan gates on a fuzz run;
  which units need a target is held instead by the manifest's `fuzz target` column (task
  `14-manifest-columns`) and by `port/port-status` failing a `required` row with no target naming
  its module (task `21-port-status-markers`). This task provides the crate, the implementations,
  the example target and the recipe those rows are held to, and writes neither check.

**What makes this a 5.** Two pieces that prove against committed artifacts rather than a live
link, each with its own uncertainty. The dump reader is a parser against a format another task
writes, and it has to carry 64-bit coordinates and a provenance check without ever touching a
float. The fuzz crate is a second build system inside the repository — its own workspace, its own
toolchain requirement and an `Arbitrary` implementation over the bridge's types rather than over a
ported type that does not exist yet. It is not split further because both halves are proven by the
same `cargo` invocations in one crate pair and neither is a task's worth of work alone.

**What it waits on.** `31-proof-harness` for the `crates/slic3r-proof` crate, `src/tiers.rs` and
the bridge's `build.rs` link; `26-stage-dump-seams` for the serialised seam state this parser
reads, and through it `25-stage-dumper` for `just oracle-dump`, the dump path and the provenance
header; `29-ffi-bridge-core` for the bridge the example target exercises and its
`-ffp-contract=off` build. A committed dump has to exist to read, so the proof of the dumps path
runs on a machine where `just oracle-build` and `just oracle-dump` have run.

**What it must not touch.** Nothing under `vendor/OrcaSlicer/`, and the submodule pointer stays at
`f92a7c065c5718b8ef1baac1cd704df2911238f4`. The root `Cargo.toml`. `rust-toolchain.toml`.
`port/manifest.tsv`. The four generated reports — `port/reports/perf-backlog.md`,
`divergences.md`, `order-containers.md`, `order-comparators.md`. `crates/slic3r-proof/src/tiers.rs`,
`crates/slic3r-ffi/cxx/` and the mock check, which are task `31-proof-harness`'s; this task's edits
outside its own files are the `mod dumps;` line in `crates/slic3r-proof/src/lib.rs` and the
`just fuzz` recipe in the `justfile`, and nothing else.
`crates/slic3r-proof/tests/trial_units.rs`, which is task `35-trial-run`'s. No file is added under
`intent/`.

## Done when

- `crates/slic3r-proof/src/dumps.rs` exists, a `mod dumps;` line in
  `crates/slic3r-proof/src/lib.rs` reaches it, and `cargo test -p slic3r-proof` passes on a machine
  where `just oracle-build` has run — the same condition task `31-proof-harness` states for its own
  tests, since the crate links the library that build produces.
- `src/dumps.rs` reads the dumps task `25-stage-dumper` commits under
  `port/dumps/20mm_cube-base-fff/`, takes the dump directory as an argument defaulting to
  `port/dumps/`, and exposes the per-seam state the dump format section of
  `oracle/stage-dump/README.md` names, keyed by seam, object, layer and region. This task depends on
  `11-fixture-set` so that `20mm_cube` is a frozen fixture with a `vendor/README.md` record — plan
  step 4 records that the un-frozen `tests/data/20mm_cube.obj` spans 0 to 20 in X and Y, inside the
  X1C's `bed_exclude_area`, so with `--arrange 0` it cannot be sliced before that task lands.
- Every coordinate is parsed as a 64-bit integer, and a test over the hand-written fixture
  `crates/slic3r-proof/tests/data/oversized-coordinate.dump` asserts that a coordinate too large to
  be represented exactly in `f64` comes back unchanged; that fixture carries a comment saying it is
  hand-written for this case, and it is the only dump in the branch that is not an oracle output.
- A dump with a missing provenance header, and a dump whose header disagrees with the pin, are
  each refused with a named error that names the file, rather than compared.
- A committed test proves proof path 2: it reads a committed dump and compares one seam's state
  through `crates/slic3r-proof/src/tiers.rs` at the tier the unit's manifest row declares.
- `crates/slic3r-ffi/fuzz/` is a `cargo-fuzz` crate whose `Cargo.toml` carries its own
  `[workspace]` table; the root `Cargo.toml` is unchanged and `cargo test --workspace` passes
  without building it.
- `Arbitrary` is implemented for a point, a polygon and a polygon with holes over 64-bit integer
  coordinates, and a generated polygon holds at least three points.
- `crates/slic3r-ffi/fuzz/fuzz_targets/` holds the example target, which round-trips arbitrary
  polygons through the bridge and asserts the coordinates come back unchanged; its name matches no
  `<crate>_<module>` pair in `port/manifest.tsv`, so `port/port-status` cannot read it as a unit's
  target, and its doc comment states the shape a qualifying unit's target follows.
- `just fuzz <target>` is a recipe in the `justfile`, runs the example target and exits 0 on a short
  run; it names the toolchain it uses explicitly, `rust-toolchain.toml` is unchanged, and where
  `cargo-fuzz` is not installed the recipe exits non-zero with a named error telling the reader to
  install it rather than installing it itself.
- `.github/workflows/ci.yml` gains no fuzz line, and this branch does not change that file.
- `git -C vendor/OrcaSlicer status --porcelain` prints nothing and
  `git -C vendor/OrcaSlicer rev-parse HEAD` prints
  `f92a7c065c5718b8ef1baac1cd704df2911238f4`.
- `just port-check` passes, and the branch changes no file under `intent/`, not the root
  `Cargo.toml`, not `rust-toolchain.toml`, not `port/manifest.tsv` and none of the four generated
  reports.
