---
kind: task
step: 11
delivers: [R8, R9]
depends: [03-manifest-file-set]
size: 5
---
# Task: Compute scope in four passes with each deduction's reason in its row

## What to build

This is **part 2 of the plan's step 11, "The scope manifest and the crate cut": the scope
algorithm and the deductions.** Step 11 is cut into five tasks and this one owns two columns of
`port/manifest.tsv` — *in scope or out with a reason* and *pipeline stage or none* — and the
computation behind both. `03-manifest-file-set` has already added `port/inventory` and the row set;
this task extends that same script. The two columns belong together because both are read off the
entry-point include closure this task builds, and the plan's step 11 lists both among the columns
"derived from the vendored tree on every run".

**The four passes.** Scope is computed in these four passes, in this order, each named in the
script:

1. The include closure over the four entry points `Model::read_from_file`, `Print::apply`,
   `Print::process` and `Print::export_gcode`. These are the four the application's headless path
   and the GUI's background slicing process both call, and they are the whole basis of scope.
2. The same-stem `.cpp` sibling of every header in the closure.
3. An orphan pass: every `.cpp` that defines methods of a class declared in an in-closure header.
4. The named deductions below, each written into its row as the reason.

**Passes 1 to 3 are re-run to a fixed point, and that changes the answer.** Pass 3 admits `.cpp`
files pass 1 never saw — `LayerRegion.cpp`, `ContourZ.cpp` and `BeltPurge.cpp` are the three the
assertions below name — and each of those carries its own `#include` directives, which are new
edges into the closure. `ContourZ.cpp:8` is `#include "SLA/IndexedMesh.hpp"`, and that edge is
exactly how the SLA decision below is reached. So the script repeats passes 1, 2 and 3 over the
files the previous round added until the in-scope set stops growing, and **prints the round number
at which it stopped**. A single pass would produce a different, smaller set; neither the plan's step
11 nor the spec's R8 says which, so it is fixed here. Pass 4 runs once, after the fixed point.

**Include resolution is case-sensitive**, and that is not a detail: `Support/SupportMaterial.cpp`
has `#include "utils.hpp"` inside a debug-only branch (at line 68, under `#define _DEBUG`), while
the tracked file is `Utils.hpp`. On this filesystem a case-insensitive resolution silently
double-counts the two. If the include graph `03-manifest-file-set` built for the file-level level
resolves case-insensitively, that is corrected here.

**The three assertions.** Passes 1 and 2 provably cannot reach three core slicing files, because
each has no header and no `#include` anywhere in the tree names it: `LayerRegion.cpp` (1,129
lines, defining `LayerRegion::make_perimeters` and `LayerRegion::prepare_fill_surfaces`, which are
declared in `Layer.hpp`), `ContourZ.cpp` (252 lines) and `BeltPurge.cpp` (433 lines). The orphan
pass is what reaches them, and the script asserts all three are present in the result and exits
non-zero otherwise. Those assertions are the only thing that proves the orphan pass ran at all, so
they are not optional and they are not a comment.

**The deductions.** Each is written as the reason in every row it takes out, so a reader of the
manifest never has to come back to a document to learn why a file is out:

| Group | Reason in the row |
|---|---|
| GUI-only mesh, measure and gizmo helpers | No caller in the four entry points' closure. |
| Parametric CAD | No caller in the four entry points' closure. |
| SLA, by file | Each `SLA/` row carries its own reason, written out below; there is no group reason for `SLA/`. |
| Arrange and orient (`Arrange.cpp`, `ModelArrange.cpp`, `IMEXArrange.cpp`, `Orient.cpp`) | Placement is fixed in the fixture and `just oracle-slice` passes `--arrange 0`, so no arranger runs in any comparison; the CLI reaches `ModelArrange::arrange_objects` only from `--copy` (`src/OrcaSlicer.cpp:5026-5040`), which no configuration uses, and its default path (`:5637`) is driven by `Slic3r::GUI::PartPlate`, which is out of scope. C14 carries the deduction, because the application does call this code. |
| Texture baking | No caller in the four entry points' closure. |
| Everything under `Format/` other than `Format/STL.{cpp,hpp}` | The port's input is a committed STL plus a resolved config, so `Format/STL.cpp` and `Format/STL.hpp` stay in and **every other file under `Format/` is out**, apart from `Format/ModelIO.{hpp,mm}`, which carry the Objective-C++ reason below. `Model::read_from_file` dispatches `.drc`, `.amf` and `.3mf` into these loaders (`Model.cpp:462-470`), including `Format/bbs_3mf.cpp` at 10,211 lines. The group is stated as the whole directory rather than as "project-format readers", because `Format/` holds 30 files at the pin and several are not project readers at all — `OBJ.cpp`, `objparser.cpp`, `svg.cpp`, `STEP.cpp`, `AssimpImport.cpp`, `SL1.cpp`, `AssembleList.cpp`, `ZipperArchiveImport.cpp`, `ProjectStrategy.hpp`, `ResourcePathUtils.hpp` — and a category a builder has to classify would leave several thousand lines of scope undecided. C6 carries it. |
| The preset layer (`Preset.cpp` 5,266 lines, `PresetBundle.cpp` 9,351 lines, `PresetCacheFormat.cpp` 666 lines) | The port's configuration input is the oracle's resolved config, not preset JSON. C6 carries it. |
| Files no caller in the vendored tree reaches | Dead at the pin. |

Three files are deducted individually so that no porting rule has to cover Objective-C++, which is
what keeps the later steps' unit rule — one `.cpp` plus its same-stem `.hpp` — true of every unit
that gets a task. `MacUtils.mm` is 23 lines holding `is_macos_support_boost_add_file_log` and
`is_mac_version_15`, two OS-version predicates no in-scope caller reaches.
`Format/ModelIO.mm` is 26 lines whose whole content is `make_temp_stl_with_modelio`, an Apple-only
importer that converts an asset to a temporary STL before the tree's own readers run, and every
committed fixture is already an STL. `Format/ModelIO.hpp` is that importer's header: it is not
itself Objective-C++, so its reason is its own — out, as the header of an Apple-only importer no
in-scope caller reaches. All three are rows — `03-manifest-file-set` puts them there — and this task
writes why they are out.

**SLA is marked in or out per file, with its own reason in each row, never by directory** (R9).
`SLA/IndexedMesh.cpp` and `SLA/IndexedMesh.hpp` are **in** scope, because Z-contouring calls them
from `ContourZ.cpp`, with `AABBMesh.cpp`, `PrintObject.cpp` and `Layer.hpp` in the same closure;
the hollowing, pad and concave-hull implementations (`SLA/Hollowing.cpp`, `SLA/Pad.cpp`,
`SLA/ConcaveHull.cpp`) are **out**. The SLA fields carried on the model type are ported as data
only, which is a translation rule for the file that carries them and not a row of its own.

**`GCode/GCodeStatistics.cpp` and `GCode/GCodeStatistics.hpp` are in scope**, because the
application calls them at `src/OrcaSlicer.cpp:7231`; the reason goes in the row the same way a
deduction's does.

Both columns are **derived from the vendored tree on every run**: a run recomputes them and
overwrites whatever is committed. Neither is a judgement a later task edits by hand.

**The `pipeline stage or none` column, and how a stage name is derived.** The cell holds one of
thirteen stage names, or `none`, and nothing else. The thirteen are the eleven hook seams the spec's
R5 enumerates, plus `psGCodeExport` and `psConflictCheck`, which are real pipeline steps with no
value in `SlicingPipelineStepPlugin` and therefore no seam — they are in the column because
`39-compare-stages` derives its whole stage list and its "fully ported" test from this column and
has to print a line for them too, falling back to the exported G-code. The names are read from the
vendored enums, `PrintObjectStep` and `PrintStep` in
`vendor/OrcaSlicer/src/libslic3r/Print.hpp`, and not restated anywhere else, which is the spec's
rule about the seam list.

The derivation, per stage, is:

1. Find the step's own block in the vendored source by its `set_started(<step>)` call — the
   per-object steps in `PrintObject.cpp` (for example `set_started(posPerimeters)` at
   `PrintObject.cpp:525`, `posPrepareInfill` at `:629`, `posInfill` at `:818`, `posIroning` at
   `:842`, `posContouring` at `:873`, `posDetectOverhangsForLift` at `:932`, `posSupportMaterial` at
   `:964`, `posSimplifyPath` at `:1060`) and the print-wide steps in `Print.cpp`
   (`set_started(psWipeTower)` at `:3645`, `set_started(psSkirtBrim)` at `:3692`,
   `set_started(psGCodeExport)` at `:5766`).
2. Take the functions that block calls, and the include closure of the files that define them,
   restricted to rows this task marks in scope.
3. A row reached by exactly one stage's closure carries that stage. A row reached by more than one
   carries the **earliest** stage in pipeline order, so the attribution is a function of the tree
   and not of iteration order. A row no stage's closure reaches, and every out-of-scope row,
   carries `none`.

The reason this is a derivation rather than a table written here: a table would be a second place
the stage list can drift from the enum, and `just inventory-check` diffs this column on every run,
so a hand-written cell would be overwritten anyway.

**What this task must not touch.** `port/cut.tsv`, the crate-level level and the `target crate and
module` column belong to `07-crate-cut`. The judgement columns — template-heavy, equality tier, tolerance,
comparison function, proof path, fuzz target — and the status column and `just inventory-check`
belong to `14-manifest-columns`. The file-set enumeration and the two traps in
`src/libslic3r/CMakeLists.txt` that go with it — the source variable misspelled upstream as
`lisbslic3r_sources`, and the `libslic3r_cgal` list that puts two files on one line — belong to
`03-manifest-file-set`. Nothing under `vendor/OrcaSlicer/` changes and the submodule pointer stays
at `f92a7c065c5718b8ef1baac1cd704df2911238f4`.

**Why this is a 5.** Four passes have to be built, not one: an include closure over four named
entry points, a same-stem sibling pass, an orphan pass that matches a `.cpp`'s method definitions
against the classes declared in in-closure headers, and the deduction pass. The orphan pass is the
hard one and it is load-bearing — without it three core slicing files are missing and the manifest
would scope the port wrongly in the direction nobody notices. Eight deduction groups each have to
be recognised from the tree and have their reason written into every row they cover, SLA has to be
decided file by file rather than by its directory, and the three assertions are what prove the
whole thing ran. The risk is in the orphan pass and in the deduction groups' edges: a group that
over-reaches takes a needed file out of scope, and a scope error here propagates into every later
step, which is why this task's own pull request is where the groups are reviewed.

**What it waits on.** `03-manifest-file-set`, for `port/inventory`, the row set and the include
graph it builds.

## Done when

- `port/inventory` computes scope in the four passes above, each pass named in the script, and
  writes every row's *in scope or out with a reason* cell.
- The script asserts that `LayerRegion.cpp`, `ContourZ.cpp` and `BeltPurge.cpp` are in the result,
  prints their presence, and exits non-zero when any of them is missing.
- Include resolution is case-sensitive: the closure resolves `Support/SupportMaterial.cpp`'s
  `#include "utils.hpp"` to nothing and reports it as unresolved, and `Utils.hpp` enters the
  closure only through an include that names it with that exact spelling.
- The script prints the round number at which the fixed point of passes 1 to 3 was reached, and
  that number is greater than one.
- Every row marked out of scope has a non-empty reason, and every reason is either one of the seven
  group reasons in the table above or one of the stated per-file reasons — the three `SLA/` rows
  below, `MacUtils.mm`, `Format/ModelIO.mm` and `Format/ModelIO.hpp`. No row's reason is the
  `SLA/` table cell, which states that `SLA/` has no group reason.
- `awk -F'\t' '$1 ~ /SLA\//' port/manifest.tsv` prints every `SLA/` file with its own in-or-out
  decision and its own reason; `SLA/IndexedMesh.cpp` and `SLA/IndexedMesh.hpp` are in, and
  `SLA/Hollowing.cpp`, `SLA/Pad.cpp` and `SLA/ConcaveHull.cpp` are out.
- `GCode/GCodeStatistics.cpp` and `GCode/GCodeStatistics.hpp` are in scope, each with a reason
  citing `src/OrcaSlicer.cpp:7231`.
- `Format/STL.cpp` and `Format/STL.hpp` are in scope, and **every** other file under `Format/` is
  out — `bbs_3mf`, `3mf`, `AMF`, `DRC`, `OBJ`, `objparser`, `svg`, `STEP`, `AssimpImport`, `SL1`,
  `AssembleList`, `ZipperArchiveImport`, `ProjectStrategy.hpp` and `ResourcePathUtils.hpp` with the
  group's reason, and `Format/ModelIO.hpp` and `Format/ModelIO.mm` with the Objective-C++ reasons —
  so that `awk -F'\t' '$1 ~ /^Format\//' port/manifest.tsv` shows exactly two in-scope rows.
- `Preset.cpp`, `PresetBundle.cpp` and `PresetCacheFormat.cpp` are out with the preset-layer
  reason; `Arrange.cpp`, `ModelArrange.cpp`, `IMEXArrange.cpp` and `Orient.cpp` are out with the
  arrange reason, which cites `src/OrcaSlicer.cpp:5026-5040` and `:5637`.
- `MacUtils.mm`, `Format/ModelIO.mm` and `Format/ModelIO.hpp` are rows, all three out, each with
  the reason stated above.
- Every in-scope row's `pipeline stage or none` cell holds one of the thirteen stage names read from
  `PrintObjectStep` and `PrintStep`, or `none`, and every out-of-scope row holds `none`; at least
  one in-scope row carries `psGCodeExport` and at least one carries `psConflictCheck`, which is what
  `39-compare-stages` needs in order to print a fallback line for those two stages.
- Editing a `pipeline stage` cell by hand and re-running `port/inventory` restores it, so the column
  is derived and not a record.
- Running `port/inventory` twice leaves the in-scope column byte-identical, and editing an
  in-scope cell by hand and re-running restores it.
- The branch adds no `port/cut.tsv`, writes no crate-level level, target crate or module, and
  changes none of the judgement columns or the status column.
- `just check` passes, `git -C vendor/OrcaSlicer rev-parse HEAD` prints
  `f92a7c065c5718b8ef1baac1cd704df2911238f4`, and `git status` reports no change under
  `vendor/OrcaSlicer/`.
