---
kind: task
step: 24
delivers: [R20]
depends: [09-comparator-gcode-levels, 26-stage-dump-seams, 33-port-surface, 22-port-status-gates, 24-port-check-ci]
size: 5
---
# Task: Compare the benchy stage by stage

## What to build

This is **part 1 of plan step 24**, "Compare the benchy stage by stage": the command. Step 24 has
two halves and the plan orders them separately — the command is built *during* step 23's fan-out,
as soon as it can read a dump and drive the port, because the per-stage comparison is meant to be
the *first* comparison against the oracle and not a report that arrives once the port is finished.
The other half, the `just compare-stages` line in the macOS job of `.github/workflows/ci.yml`, is a
separate task (`40-compare-stages-ci`) and lands after all of step 23. **This task adds no CI
line.**

Build `just compare-stages`, wired to the `oracle-compare` binary, with the work in a new
`crates/oracle-compare/src/stages.rs` exported from `crates/oracle-compare/src/lib.rs` and reached
from `crates/oracle-compare/src/main.rs`. It slices `fixtures/3DBenchy.stl` with the port in the
base FFF configuration and compares, for every pipeline stage that is fully ported, the port's
stage output against that stage's oracle dump, at the stage's declared tier, and reports the first
stage that differs.

**Which stages exist, and which oracle.** The stages are the values of the manifest's
`pipeline stage` column, which task `06-manifest-scope` derives and which holds a stage name or
`none` and nothing else. **That column carries thirteen stage names**: the eleven hook seams the
spec's R5 enumerates, which this task does not restate, plus `psGCodeExport` and `psConflictCheck`,
the two steps with no seam that this command falls back to the exported G-code for. Read the list
from the manifest column, which is where it is recorded; do not restate it here and do not
re-derive it from `vendor/OrcaSlicer/src/libslic3r/Print.hpp`, which is the spec's own rule about
the seam list and which also matters because the enum holds thirteen values of its own and the
eleven are what they are only because of R5's two deliberate exclusions. The oracle for a stage dump
is always the locally built library, which is normative for
every per-stage proof, so this command has no oracle to choose and no
`port/reports/oracle-parity.tsv` to read.

**"Fully ported" is derived, never read.** Take every in-scope row of `port/manifest.tsv` whose
`pipeline stage` column names the stage; the stage is fully ported when every one of those rows has
a derived status of `ported`, computed the same way `port/port-status` computes it, which is why
this task depends on `22-port-status-gates`: a file at the
row's target path with a `PORT STATUS` trailer naming that row's source path, an uncommented `mod`
line reaching it, no `todo!()` or `unimplemented!()`, and the `// PROOF: <proof path>` line beside a
test naming the module that task fixes as the proof-presence convention. Proof *outcome* is
`cargo test`'s to report, here as there. The command must never read the manifest's status column
as a fact. A stage whose rows are not all `ported` prints `not ported` and is **not a failure**:
that is what makes the command useful from the day it lands and stronger with every Phase B batch
that merges.

**The dumps.** Read the committed stage dumps the step 8 dumper writes — one dump per fired seam,
line-oriented text with integer `coord_t` coordinates and a `;`-comment provenance header,
produced by `just oracle-dump 3DBenchy base-fff`. Parse exactly the format
`oracle/stage-dump/dump_seams.cpp` writes; do not define a second dump format, and do not change
the dumper. Keep `oracle-compare` free of any dependency on `crates/slic3r-ffi` or on a libslic3r
build, as it is today — which is why **`src/stages.rs` reaches the port as a subprocess and not as a
library**: it invokes the `grampus-cli` binary with `--dump-stages <dir>` and reads the files it
writes, locating it through `CARGO_BIN_EXE_grampus-cli` in tests and through a path the
`just compare-stages` recipe passes otherwise. `crates/oracle-compare/Cargo.toml` gains no
dependency on `grampus-cli` or on any port crate; a library dependency would pull the port crates
and, through them, `crates/slic3r-ffi` and its link against `vendor/OrcaSlicer/build/arm64`, and
`cargo test -p oracle-compare` would then need a libslic3r build. When a stage is fully ported and its dump is absent, exit non-zero naming
the missing dump and the command that produces it, rather than reporting the stage as differing.

**The tier.** Compare at the tier the stage's manifest rows declare — the `equality tier`,
`tolerance` and `comparison function` columns — which is exact equality of integer-coordinate
output for a fixed-point stage, and the row's declared tolerance and comparison function for a
float one. A tolerance with no reason text in its row is refused, the way the comparator already
refuses one. Where the rows of one stage declare different tiers, hold the stage to the strictest
of them and print which tier was used; write that rule down beside the code, because it is a rule
this task chooses and a later reader will look for it.

**The port's stage output.** Drive the port through the surface
`crates/grampus-cli/src/surface.rs` exposes — build a model, apply a config, validate, process,
export G-code — and add the one thing it is missing: a `--dump-stages <dir>` flag on
`crates/grampus-cli/src/main.rs` and the emission call sites in `surface.rs` at the same eleven
seams, writing the dumper's format. A seam whose modules are not ported yet writes nothing, which
is what lets this command land before any stage is complete; while `src/main.rs` still exits
non-zero with its "not yet ported" error, the command prints `not ported` for every stage and exits
0 rather than reporting a failure. The emission must not change the slicing sequence or the
exported G-code: the port's G-code with `--dump-stages` is byte-identical to its G-code without it.

**The fallback, for the two stages with no seam.** G-code emission and conflict check have no value
in the hook's step enum and therefore no seam at all, and that gap is accepted rather than filled
with a pseudo-stage dump. For those two the command falls back to the comparator's L0 to L4 levels
on the exported G-code — the port's against the built oracle's, which
`just oracle-dump 3DBenchy base-fff` exports — localised by the `; FEATURE:`, `; Z_HEIGHT:` and
per-object label comments, reusing the localisation already in
`crates/oracle-compare/src/gcode.rs` rather than writing a second parser. It prints
`fallback: exported G-code` for those two lines and **says it is in fallback rather than naming
`psSkirtBrim`, the last seam, as the first difference**.

Cover it in `crates/oracle-compare/tests/` against committed fragments under `tests/data/`: a stage
that matches, a stage that differs, a stage that is not ported, a missing dump, and the fallback
path on a pair of exported G-code files.

**What makes this a 5, and why it is not split further.** Three things that are each more than
wiring: the derivation of "fully ported" from the Rust tree and the manifest rather than from a
cell, which has to agree with `port/port-status`'s rule exactly or the command lies; the per-stage
dump comparison at a tier read per row, across two crates, which needs the port to emit stage state
it does not emit today; and the comment-localised G-code fallback for the two seamless stages, with
the output saying it is in fallback. They are not split because they are one run of one command
sharing one stage table and one output format — a command that printed the stage table without the
fallback would report the last seam as the first difference for the largest in-scope subsystem,
which is exactly the failure the step exists to avoid.

**What this task must not touch.** `.github/workflows/ci.yml` (task `40-compare-stages-ci` owns
that line), `port/manifest.tsv`, anything under `port/reports/`, the root `Cargo.toml`,
`Cargo.lock`, `oracle/stage-dump/`, and anything under `vendor/OrcaSlicer/`.

## Done when

- `just compare-stages` prints one line per stage, with the level or tier result for a stage whose
  every in-scope manifest row has a derived status of `ported`, and `not ported` for the rest, and
  names the first stage that differs.
- Run on a tree where no stage is fully ported, it exits 0 and prints `not ported` for every stage
  rather than failing.
- `cargo test -p oracle-compare`'s fallback test shows the command printing
  `fallback: exported G-code` for the `psGCodeExport` and `psConflictCheck` lines and never naming
  `psSkirtBrim` as their first difference. The check is a test rather than a run of the command,
  because on the tree this task lands on no stage is fully ported and every line reads
  `not ported`.
- It prints which tier it held each compared stage to, and refuses a tolerance whose manifest row
  carries no reason text.
- It exits non-zero, naming `just oracle-dump 3DBenchy base-fff`, when a fully-ported stage has no
  committed dump.
- The stage list and each stage's rows come from `port/manifest.tsv`'s `pipeline stage` column —
  thirteen names, `psGCodeExport` and `psConflictCheck` among them — and `src/stages.rs` restates no
  seam list of its own; the command reads no status cell, which a reviewer checks by grepping
  `crates/oracle-compare/src/stages.rs` for the status column and finding nothing.
- `crates/grampus-cli` takes `--dump-stages <dir>` and writes the dumper's format at the seams
  whose modules are ported, and the port's exported G-code is byte-identical with and without the
  flag.
- `src/stages.rs` invokes the `grampus-cli` binary as a subprocess and
  `crates/oracle-compare/Cargo.toml` names no dependency on `grampus-cli` or any `slic3r-*` crate,
  so `cargo test -p oracle-compare` passes with no libslic3r build present.
- `cargo test -p oracle-compare` passes the new stage tests, including the matching, differing,
  not-ported, missing-dump and fallback cases.
- `just check` and `just port-check` pass, and `git status` shows no change to
  `.github/workflows/ci.yml`, `port/manifest.tsv`, `port/reports/`, `Cargo.toml`, `Cargo.lock`,
  `oracle/stage-dump/` or `vendor/OrcaSlicer/`.

**What it waits on.** `09-comparator-gcode-levels` for the comparator's levels and its G-code
localisation; `26-stage-dump-seams`, and through it `25-stage-dumper`, for the committed dumps and
their format; `33-port-surface` for the surface this command drives; `22-port-status-gates` for the
status derivation this command has to agree with; and `24-port-check-ci` for the `just port-check`
recipe this branch has to pass. The plan's step 24 says the command waits on steps 5, 8 and 19 and
on nothing else; the last two are added because this command mirrors `port/port-status`'s derivation
and is held to `port-check`, neither of which exists under that narrower reading.
