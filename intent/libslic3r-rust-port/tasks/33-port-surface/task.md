---
kind: task
step: 19
delivers: [R10]
depends: [08-comparator-workspace, 12-config-goldens, 14-manifest-columns, 19-porting-contract, 24-port-check-ci]
size: 5
---
# Task: Expose the port's GUI-free slicing surface and CLI skeleton

## What to build

This is the **whole of plan step 19, "The port's slicing surface and CLI skeleton"**, which
delivers R10. Read plan step 19 and spec R10 and R11 before starting, together with C14's recorded
decision (arrange and orient are left to the following intent) and C6's (the port slices a mesh
plus a resolved configuration, not a project).

Add `crates/grampus-cli` with `Cargo.toml`, `src/surface.rs`, `src/resolved_config.rs`,
`src/main.rs` and `tests/surface.rs`. The root `Cargo.toml` is **not** edited: it is
`members = ["crates/*"]` (plan step 5), so the directory appearing is enough. `Cargo.lock` may
change — plan step 5 names step 19 among the steps that may change it.

**`src/surface.rs` — the five-call sequence, and nothing else.** It mirrors what
`vendor/OrcaSlicer/tests/fff_print/test_helpers.cpp` does, which is the GUI-free sequence R10
names: build a model (`init_print` at `:253-282` adds an object, adds its volume and adds an
instance), apply a config (`print.apply(model, config)`, `:283`), validate (`print.validate()`,
`:284`), process (`print.process()`, `:352`) and export G-code
(`print.export_gcode(temp.string(), result, nullptr)`, `:353`). The two per-object calls
`init_print` makes between building the model and applying the config —
`ModelObject::ensure_on_bed()` (`:279`) and `Print::auto_assign_extruders()` (`:280`) — belong to
the build-a-model call and are exposed rather than dropped, so the port's sequence matches the
oracle's; for a committed fixture `ensure_on_bed` is the identity, which is the single resolved
instance transform R24 records.

**The five calls' shape, fixed here because tasks 39 to 43 are written against it.** Nothing under
`crates/slic3r-*/` is ported yet, `src/main.rs` must return a named "not yet ported" error, and
`todo!()` and `unimplemented!()` are banned, so the surface is a shape with no implementation behind
it — and its shape has to be stated rather than invented. It is one struct, `Surface`, with five
inherent methods, in this order:

```rust
pub struct Surface { /* private state */ }

impl Surface {
    pub fn new() -> Self;
    pub fn build_model(&mut self, mesh_path: &Path) -> Result<(), SurfaceError>;
    pub fn apply_config(&mut self, config: &ResolvedConfig) -> Result<(), SurfaceError>;
    pub fn validate(&self) -> Result<(), SurfaceError>;
    pub fn process(&mut self) -> Result<(), SurfaceError>;
    pub fn export_gcode(&self, out: &Path) -> Result<(), SurfaceError>;
}
```

`build_model` is the call that also carries `ensure_on_bed` and `auto_assign_extruders`.
`SurfaceError` has two variants that matter now: `NotYetPorted`, which every one of the five returns
until Phase B lands and which is the named error `src/main.rs` exits non-zero with, and
`OutOfOrder`, returned when a call is made before the one before it has succeeded. The
out-of-order state is tracked in `Surface`'s private state, which is what makes the order
mechanically checkable rather than a matter of reading the test.

Three exclusions are the point of this module, and each is checkable:

- **No arranger.** `init_print`'s own `arrange` parameter
  (`vendor/OrcaSlicer/tests/fff_print/test_helpers.hpp:88`, default `true`) is not part of the
  surface, and the `arrange_objects(model, InfiniteBed{}, ArrangeParams{...})` call it guards
  (`test_helpers.cpp:277`) is not reachable from it. Placement is fixed in the fixture (R24) and
  every oracle invocation passes `--arrange 0`, so no arranger runs on either side of any
  comparison.
- **Nothing from the plate layer.** No type, function or constant from
  `vendor/OrcaSlicer/src/slic3r/GUI/PartPlate` is referenced or reimplemented, which R10's proof
  checks with `grep -r 'PartPlate' crates/`.
- **No preset resolution.** The surface takes a resolved configuration, so nothing here reads
  `vendor/OrcaSlicer/resources/profiles/`, a preset file or an `inherits` chain (R11).

**The Bambu-printer flag is set explicitly by the caller.** The oracle does not derive it from the
resolved configuration: the application assigns it from the printer's vendor preset at
`vendor/OrcaSlicer/src/OrcaSlicer.cpp:7026`
(`(dynamic_cast<Print*>(print))->is_BBL_printer() = is_bbl_vendor_preset;`), and
`Print.hpp:1351-1353` keeps it as a plain field that also decides the wipe-tower type, while
G-code emission reads it at `src/libslic3r/GCode.cpp:3437`. So the surface takes it as an explicit
value from its caller, because validation and G-code depend on it and no golden carries it.

**`src/resolved_config.rs` — the port's configuration input.** It reads the goldens task
`12-config-goldens` commits under `port/goldens/configs/`: one resolved configuration per
`docs/ORACLE.md` row, each produced by `just oracle-slice ... --export-settings` on an unarmed run
with a throwaway `--datadir`, each carrying plan step 3's provenance header as a `provenance`
object. Bed shape, exclusion areas, wipe-tower placement and filament maps reach the port as
resolved configuration values through this reader and in no other way.

- It treats the golden's `provenance` object as the header it is, not as a configuration key, and
  fails with a named error on a golden that carries none.
- **The two keys that exist only in the G-code config block are derived, not read.**
  `first_layer_temperature` and `first_layer_bed_temperature` are absent from the exported JSON;
  the oracle writes them into the G-code from values it computes at emission time
  (`src/libslic3r/GCode.cpp:3720-3724`, written at `:4333-4334`). So the reader must not fail on
  their absence, and must mark them as derived at emission rather than read, so the Phase B
  emission module takes them from the surface and not from the golden. Writing that emission is
  not this task's work.

**`src/main.rs` — the binary tasks `39-compare-stages`, `41-compare-benchy`,
`42-compare-fixtures` and `43-compare-configurations` call.** Until Phase B lands (plan step 23)
it exits non-zero with a named "not yet ported" error and **not** a panic: return a named error
variant that prints what is not ported, and do not reach that failure with `todo!()` or
`unimplemented!()` — `just port-status` counts both in the burn-down R16 requires to reach zero.

Fix the command line here, because four later tasks call it and there has to be one spelling: a
positional model path, `--config <golden>`, `--out <dir>` and `--export-statistics`, matching
`oracle/slice`'s `--out` and `--export-statistics` (plan step 2). It writes its G-code as
`plate_1.gcode` in `--out`, the name the oracle writes (`plate_<N>.gcode`, plan step 2), and its
statistics JSON under the oracle's own `plate_1.statistics.json` basename, so the comparator
compares like-named files. Until Phase B lands each export path reaches the same named error.

**`tests/surface.rs` — the shape and the round trip in one test file.** It asserts that the
surface exposes the five calls in that order and exposes no arrange or orient entry point, and it
round-trips a committed golden: the golden is read through `src/resolved_config.rs`, the values the
surface needs are read back out unchanged, the absence of the two G-code-only keys does not fail
the read, and both are reported as derived. That round trip is the test R10's and R11's proofs
name.

**This crate holds no ported module**, so no file in it carries a `PORT STATUS` trailer and the
per-module checks that apply to ported modules do not apply to it. `just port-check` must still
pass, `cargo fmt --check` included, since the crate compiles and a `mod` line reaches every file
in it.

**What makes this a 5.** It is the surface every later comparison calls, so its shape is load
bearing for tasks `39` to `43`: the five-call sequence with no arranger and nothing from
`PartPlate`, the resolved-config reader over the goldens including the two keys the oracle derives
at emission time rather than exporting, and a binary that has to fail with a named error rather
than a panic while nothing underneath it is ported yet. It is **not split**, because the surface
and the config input are tested together by one round-trip test: splitting them would leave each
half with no proof.

**What it waits on.** `08-comparator-workspace` for the Rust workspace and the committed
`Cargo.lock`; `12-config-goldens` for the goldens under `port/goldens/configs/` that
`src/resolved_config.rs` reads and the round-trip test uses; `14-manifest-columns` for the crate
map and the in-scope row set; `24-port-check-ci` for the `just port-check` recipe this branch has to
pass, which is the task that creates it and which brings `20-workflow-document`,
`21-port-status-markers` and `22-port-status-gates` with it, since `port-check` runs
`just port-status --per-commit` and that fails unless `docs/WORKFLOW.md` holds its sections; and
`19-porting-contract` for `docs/PORTING.md`, whose crate map, type map, idiom map and per-API
scaling rule the types in this crate follow rather than invent. It does not wait on any ported
module: nothing under `crates/slic3r-*/` exists yet.

**What it must not touch.** Nothing under `vendor/OrcaSlicer/`, and the submodule pointer stays at
`f92a7c065c5718b8ef1baac1cd704df2911238f4`. The root `Cargo.toml`. `port/manifest.tsv`. The four
generated reports — `port/reports/perf-backlog.md`, `divergences.md`, `order-containers.md`,
`order-comparators.md`. `docs/PORTING.md` and `docs/WORKFLOW.md`, which tasks `19-porting-contract`
and `20-workflow-document` own. `port/goldens/configs/`, which this task reads and never rewrites.
`.github/workflows/ci.yml`: plan step 19 adds no CI line, and `cargo test --workspace` in the
macOS job task `24-port-check-ci` creates already covers this crate. No file is added under
`intent/`.

## Done when

- `crates/grampus-cli` exists with `Cargo.toml`, `src/surface.rs`, `src/resolved_config.rs`,
  `src/main.rs` and `tests/surface.rs`, and `cargo test -p grampus-cli` passes.
- `tests/surface.rs` calls the five entry points in order — `build_model`, `apply_config`,
  `validate`, `process`, `export_gcode` — and asserts that calling `process` before `apply_config`
  returns `SurfaceError::OutOfOrder`, which is what makes the order a mechanical check rather than a
  reading of the test.
- `Surface`'s public API is the five methods and `new` and nothing else, so it exposes no arrange
  and no orient entry point, and
  `grep -rn 'arrange_objects\|ArrangeParams\|orient(' crates/grampus-cli/src` finds nothing. The
  grep is on the call forms rather than on the words, because this module deliberately carries
  comments saying why there is no arranger.
- `grep -r 'PartPlate' crates/` finds nothing.
- `grep -rn 'resources/profiles\|inherits' crates/grampus-cli/src` finds no read of a preset file
  or an `inherits` chain.
- The Bambu-printer flag is a value the caller passes to the surface, and the module comment cites
  `src/OrcaSlicer.cpp:7026` for why it is not read from the resolved configuration.
- `src/resolved_config.rs` reads a committed golden from `port/goldens/configs/`, treats its
  `provenance` object as the header and not as a configuration key, and fails with a named error
  on a golden with no header.
- The round trip in `tests/surface.rs` passes on a committed golden, does not fail on the absence
  of `first_layer_temperature` and `first_layer_bed_temperature`, and reports both as derived at
  emission time.
- `src/main.rs` exits non-zero with a named "not yet ported" error rather than a panic, printing
  what is not ported, and `grep -rn 'todo!\|unimplemented!' crates/grampus-cli/src` finds neither
  of the two the burn-down counts.
- The binary takes a positional model path, `--config <golden>`, `--out <dir>` and
  `--export-statistics`, and its `--help` names those four.
- No file in `crates/grampus-cli` carries a `PORT STATUS` trailer, and `just port-check` passes,
  `cargo fmt --check` included — the recipe exists on this branch because `24-port-check-ci` is in
  this task's `depends`.
- `git -C vendor/OrcaSlicer status --porcelain` prints nothing and
  `git -C vendor/OrcaSlicer rev-parse HEAD` prints
  `f92a7c065c5718b8ef1baac1cd704df2911238f4`.
- The branch changes no file under `intent/`, not the root `Cargo.toml`, not `port/manifest.tsv`,
  none of the four generated reports, neither contract document, nothing under
  `port/goldens/configs/` and not `.github/workflows/ci.yml`.
