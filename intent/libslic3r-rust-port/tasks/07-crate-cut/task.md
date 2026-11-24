---
kind: task
step: 11
delivers: [R8, R9]
depends: [03-manifest-file-set, 06-manifest-scope]
size: 5
---
# Task: Compute and commit the feedback-arc cut and the crate levels

## What to build

This is part 3 of plan step 11, "The scope manifest and the crate cut". It computes and commits
`port/cut.tsv` — the named feedback-arc cut of the module graph — and fills the manifest columns
only the cut can produce: `crate-level level`, `target crate` and `target module`. It extends the
`port/inventory` script and the `just inventory` recipe that task `03-manifest-file-set` adds and
task `06-manifest-scope` extends.

It waits on `03-manifest-file-set` for the file set, the `subsystem` and `lines` columns and the
resolved per-file `#include` edges, and on `06-manifest-scope` for each row's in-scope-or-out
decision, which selects the rows the crate map covers. Nothing else: plan step 11 reads only the
vendored tree and starts on day one.

**Why this is a 5.** The module graph is cyclic — the spec measured one strongly connected component
holding 39 modules and 321,722 lines — so a cut is mandatory before any crate boundary exists, and
C10's decision accepts
the computed cut as the crate map — which means this task decides the whole order of work for
plan steps 20, 22 and 23. The two mutual clusters must come out as single crates. The risk is the
algorithm and its determinism, not the volume of code: the script is small, but a cut that is not
reproducible, or that splits a mutual cluster, makes every later step's ordering wrong. It is not
split further because the cut, the crate map and the crate-level level all fall out of one graph
computation, and splitting them would give two tasks the same graph.

**The module graph, and what a module is.** Crate boundaries are drawn over modules, not files.
**A row's module is not its `subsystem` cell**, and this task derives the module itself rather than
reusing that column. The rule, written into `port/inventory`:

- A file under a subdirectory of `src/libslic3r` belongs to the module named by that subdirectory at
  its **full nested path** — so `Fill/Lightning/Layer.cpp` is in module `Fill/Lightning` and not in
  `Fill`, and `Arachne/utils`, `Arachne/BeadingStrategy`, `Feature/FuzzySkin` and
  `Feature/Interlocking` are modules of their own. There are 20 such directories at the pin.
- A file directly under `src/libslic3r` belongs to a module of its own same-stem unit: the `.cpp`
  and its same-stem `.hpp` or `.h` together, or a lone header. There are 164 such stems at the pin.

**Why the module cannot be the `subsystem` column.** Task `03-manifest-file-set` defines
`subsystem` as the file's top-level subdirectory, with `root` for a file directly under
`src/libslic3r`, and pins it by its own "Done when" to one of fifteen names or `root`. That is 16
vertices, 287 of the 607 files in one of them, and under it three requirements of this very task
fail by construction rather than contingently: `Print.cpp` and `GCode.cpp` would be the same module
and so could never have different crates; `Model` would not be a module at all and `GCode.cpp` would
not be one either, so neither named merge group could be expressed and the script's own
group-not-in-graph guard would fire; and a 16-vertex graph cannot hold a 39-module strongly
connected component. The derivation above is the coarsest one under which all three are expressible,
which is why it is the one written here.

**What this derivation does not reproduce, and what happens about it.** The spec records 43 modules,
5 components, and one component holding 39 modules and 321,722 of 337,335 lines, and the plan's
step 20 speaks of "roughly 43 crates"; neither document states the derivation those figures were
measured with, and the derivation above yields far more vertices than 43 — 184 before the two
merges at the pin, from 164 root stems and 20 directories, fewer once out-of-scope modules drop out.
**Say in the pull request what that costs downstream**, because it is the reviewer's decision and
not this script's: most of those 164 root-stem crates hold a single unit, so for them the batching
rules of `15-task-cuts` — Phase A batching "within one target crate until either 6 units or 2,500
source lines", Phase B "one task per crate, except a crate whose line sum exceeds 2,500 lines is
split" — collapse to one task per unit, and the per-crate batching that the plan's steps 22 and 23
rest on only does real work inside the 20 directory crates and the two merged clusters. The
alternative granularity, the `subsystem` column, keeps the batching intact but puts 287 files in one
crate and fails the three requirements named above outright. **The script therefore prints its
own numbers and asserts none of them against a figure from a document.** C10's decision is that the
computed cut is the crate map and that this pull request is where it is reviewed, so the crate count
this derivation produces, beside the spec's figures, is part of what the body puts in front of the
reviewer — and if the reviewer decides the map must be coarser, the named-merge-group mechanism
below is where that coarsening is written, in this task's own pull request, rather than by
redefining the module afterwards. Task `34-foundation-spike` reads the crate count from
`port/cut.tsv` for the same reason and does not carry the plan's estimate as a number.

Build a directed module graph from the per-file
include edges task 03 already resolves: map each file-to-header edge onto
module-to-module, drop self-edges, and weight each module edge with the number of include
references it carries. Resolve includes case-sensitively, as task 03 does, because
`Support/SupportMaterial.cpp:68` includes `"utils.hpp"` in a debug-only branch where the tracked
file is `Utils.hpp`, and this filesystem would otherwise double-count two files. If the include
edges are not already a reusable function in `port/inventory`, factor them into one without
changing what task 03 or task 06 computes from them.

**Measure before cutting anything.** Over every row, run Tarjan and print: the module count, the
strongly connected component count, the largest component's module count and line sum, and the
total number of include references between modules. These are reported, not asserted, for the
reason stated above, and they go in the pull request body beside the spec's figures.

Assert, and print, that the graph at *file* granularity has no non-trivial strongly connected
component. That is the claim R8 rests on, it is what makes task 03's file-level include-DAG level
meaningful, and plan step 23's batch ordering depends on it. Add the assertion here if task 03 does
not already make it.

**Merge the two mutual clusters, then cut.** The clusters the cut cannot break cheaply are
`GCode.cpp` with `GCode/*`, and `Model` with `Format`, and C10's decision is that each is a single
crate. In the module vocabulary above those are the group `{GCode, GCode/}` — the root same-stem
unit `GCode` together with the `GCode` directory module — and the group `{Model, Format/}` — the
root same-stem unit `Model` together with the `Format` directory module. Write the two groups into
the script by those module names and merge each into one vertex *before* ordering or cutting, which
is what guarantees no cut edge can fall inside one. The script exits non-zero naming the group when
a module it names is not in the graph; under the derivation above all four names are modules, so
that guard fires only if the derivation is changed.

**The cut.** Compute it over the in-scope rows only, with the two merges applied: the crate map
covers what is ported, and a module whose every row is out of scope has nothing to port and names
no crate. (The plan's "roughly 43 crates" comes from the whole-library module measurement above;
the script prints the crate count it actually produced.) Order the merged in-scope vertices with a
deterministic greedy feedback-arc-set heuristic — Eades–Lin–Smyth sequencing over the
reference-weighted graph, with ties broken by module name ascending — and cut every edge that runs
backwards in that order. Then assert with Tarjan that the remaining graph has no non-trivial
strongly connected component, and that each merged group lies wholly inside one crate.

Determinism is a requirement, not a nicety, because `just inventory-check` (task
`14-manifest-columns`) diffs the derived columns on every pull request: no step may iterate a set
or a mapping whose order is not fixed, and every sort key must be total. The spec's 62 edges
carrying 156 of 2,696 references is the whole-library figure C10 records, so the in-scope cut is
expected to be smaller; print the edge count and the references each edge carries rather than
asserting that figure.

**`port/cut.tsv`.** It has five consumers besides this task, and none of them is served by the cut
edges alone: plan step 20's `port/crate-skeletons` (task `34-foundation-spike`) writes a
`Cargo.toml` and a `lib.rs` for every crate the cut names; plan step 15's `docs/PORTING.md` (task
`19-porting-contract`) generates its crate map from this file; and `port/cut-tasks` (task
`15-task-cuts`, run by the spikes `37-phase-a-spike` and `38-phase-b-spike`) reads **the crates a
crate depends on** out of this file for every `phase-b` row it writes. So write it as a
tab-separated file with a header row and a first column `kind` holding three record kinds:

- `crate` rows: the crate name, its crate-level level, the modules it holds, its in-scope file
  count and its in-scope line sum;
- `dep` rows: one per **retained** crate-to-crate edge — the source crate, the target crate it
  depends on, and the include references the edge carries. These are the edges the feedback-arc cut
  did *not* remove, and they are the crate dependency graph. Without them neither
  `15-task-cuts`'s "the crates it depends on" nor this task's own level check below can be answered
  from any committed artifact, and a reader would have to approximate the dependencies from the
  `crate-level level`, which is not the same thing.
- `edge` rows: one per **cut** edge — the cutting file's path, the header it includes, the source
  module, the target module, and the include references the edge carries.

Sort it so re-running is byte-identical: `crate` rows by level then name, `dep` rows by source
crate, then target crate, `edge` rows by source module, target module, then path. Document the
schema in the script and in the pull request body, since later steps read the file.
`port/cut.tsv` is not an oracle output and carries no provenance header.

**The manifest columns this part fills**, in the positions plan step 11 lists them — path,
subsystem, lines, file-level include-DAG level, crate-level level, in scope or out with a reason,
target crate, target module, pipeline stage, then the judgement columns:

- `crate-level level`: the level of the row's crate in the cut graph, the longest path to a crate
  with no in-scope dependency, which is level 0.
- `target crate`: `slic3r-<module, lowercased with every run of non-alphanumeric characters
  replaced by a hyphen>`, with the two merged groups named `slic3r-gcode` and `slic3r-model`.
- `target module`: the Rust module path a port task writes,
  `crates/<crate>/src/<stem in snake_case>.rs`, from the unit's `.cpp` stem, so a `.cpp` and its
  same-stem `.hpp` share one target module — that pairing is plan step 15's unit of work and plan
  steps 22 and 23 write files at exactly this path. **An in-scope header with no same-stem `.cpp`
  takes its own stem in snake_case**, by the same rule: 77 of the tree's headers have no `.cpp`
  sibling at the pin, and plan step 20's seed rule names one of them, `AABBTreeIndirect.hpp`, as an
  in-scope row, so the cell cannot be left undefined for them.
- An out-of-scope row has all three cells empty.

Two in-scope files whose stems collide inside one crate would share one Rust module path, so **the
script always applies a stated deterministic disambiguation rule rather than failing**: the parent
directory prefixed in snake_case. It prints every module it disambiguated, and the printed list is
how a reviewer sees that the rule fired. The script does not fail on a collision, because failing
would make this task's first "Done when" — `just inventory` exits 0 — unanswerable the moment one
exists. At the pin only two `.cpp` stems repeat in the whole tree, `Layer.cpp` against
`Fill/Lightning/Layer.cpp` and `Circle.cpp` against `Geometry/Circle.cpp`, and under the module
derivation above each pair falls in a different crate, so the list is expected to be empty — which
is itself worth printing.

Add the columns to the header if task 03 has not created them, and change no other cell of any row.

**The recipe.** `just inventory` now regenerates `port/manifest.tsv` and `port/cut.tsv` together,
and prints the manifest row count, the module and component counts, the cut's edge count and the
references it carries, which is R8's proof line. Running it twice leaves both files unchanged;
that repeat run is this task's own determinism check, because `just inventory-check` belongs to
task `14-manifest-columns`.

**The pull request is where the cut is reviewed.** C10's decision is that the computed cut is
accepted as the crate map, reviewed like any other change in the pull request that commits it, and
changed afterwards only by a pull request that regenerates the manifest. So the body lists every
`crate` row with its level, file count and line sum, and every cut edge with the references it
carries, so a reviewer judges the crate map itself and not only the script.

**What this task must not do.** Write no `in scope or out with a reason` cell and no
`pipeline stage or none` cell: both are task `06-manifest-scope`'s, derived from its entry-point
include closure. Write no judgement column — template-heavy, equality tier,
tolerance, comparison function, proof path, fuzz target — and no status cell: task
`14-manifest-columns` seeds and carries those forward, and status is derived from the Rust tree and
never written into a cell. Add no `just inventory-check` (task 14), no `port/cut-tasks` or
`just task-cuts` (task `15-task-cuts`), no `port/crate-skeletons` and no directory under `crates/`
(task `34-foundation-spike`), and no `docs/PORTING.md` (task `19-porting-contract`). Nothing under
`vendor/OrcaSlicer/` changes and the submodule pointer stays at
`f92a7c065c5718b8ef1baac1cd704df2911238f4`.

## Done when

- `just inventory` exits 0 and prints the manifest row count, the module count, the component
  count, the largest component's module count and line sum, the in-scope cut's edge count, and the
  total include references the cut carries.
- Running `just inventory` twice in a row leaves `port/manifest.tsv` and `port/cut.tsv`
  byte-identical: `git diff --exit-code port/` prints nothing after the second run.
- `port/cut.tsv` is committed with a header row, a `crate` row for every crate the cut names, a
  `dep` row for every retained crate-to-crate edge naming the source crate, the target crate and the
  references it carries, and an `edge` row for every cut edge naming the file, the header it
  includes and the references it carries.
- The script asserts, and the output shows, that the module graph with the cut edges removed has no
  non-trivial strongly connected component, and that the include graph at file granularity has none
  either.
- `GCode.cpp` and every `GCode/*` row share one `target crate`, and `Model` and every `Format/*`
  row share one `target crate`; no `edge` row has both its source and its target module inside one
  of those two groups.
- `Print.cpp` and `GCode.cpp` do **not** share a `target crate`. That is the line that catches a
  module granularity coarser than the one this task derives: were the module the `subsystem` column,
  all 287 library-root files would be one module, every other criterion here would pass, and the
  crate map would be wrong.
- `grep -n 'subsystem' port/inventory` shows the `subsystem` column is not what the module graph is
  built from, and a reviewer reads the module derivation in the script and finds the directory rule
  and the root same-stem-unit rule.
- `just inventory` prints the module count, the strongly connected component count, the largest
  component's module count and line sum, and the crate count, and the pull request body records all
  five beside the spec's 43 modules, 5 components and 39-module component, with the difference
  explained.
- Every in-scope manifest row has a `crate-level level`, a `target crate` and a `target module`,
  including an in-scope header with no same-stem `.cpp`, whose `target module` is its own stem in
  snake_case; every out-of-scope row has all three empty.
- No two in-scope rows share a `target module` unless they are a `.cpp` and its same-stem `.hpp`,
  and the script prints the list of modules it disambiguated by the parent-directory rule, which is
  empty at the pin.
- At least one crate has no `dep` row and is at level 0, and for a crate a reviewer picks from the
  `crate` rows, its level is one more than the highest level among the crates its `dep` rows name —
  a check answerable from the `crate` and `dep` rows alone.
- No judgement column and no status cell differs from what was committed before this task, and
  `just inventory-check`, `port/cut-tasks`, `port/crate-skeletons`, `crates/` and
  `docs/PORTING.md` are neither created nor changed here.
- The pull request body lists every `crate` row with its level, file count and line sum, and every
  cut edge with the references it carries.
- `git -C vendor/OrcaSlicer status --porcelain` prints nothing and the submodule pointer is
  unmoved.
