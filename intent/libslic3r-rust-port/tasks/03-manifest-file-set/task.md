---
kind: task
step: 11
delivers: [R8, R9]
size: 5
---
# Task: Enumerate every libslic3r source file at the pin into the manifest

## What to build

Part 1 of the plan's step 11, "The scope manifest and the crate cut": **the file set and the
per-file derived columns**. This file's `delivers` is the whole step's, and R9 in particular is
part 2's work. This task waits on nothing, reads only the vendored source, and starts immediately,
alongside step 1 and step 7.

**Add `port/inventory`** and the `just inventory` recipe, which regenerates `port/manifest.tsv`;
commit the generated `port/manifest.tsv`. Like every script outside the Rust workspace it is a
standalone executable with PEP 723 inline metadata and no file extension, run through `uv`.

**Write the manifest's full header row now**, so later parts fill columns rather than reshape the
file. In step 11's order: `path`, `subsystem`, `lines`, `file-level include-DAG level`,
`crate-level level`, `in scope or out with a reason`, `target crate and module`,
`pipeline stage or none`, `template-heavy`, `equality tier`, `tolerance`, `comparison function`,
`proof path`, `fuzz target`, `status`. This task fills the first four in every row and leaves the
rest empty.

**The file set is the filesystem walk, and `src/libslic3r/CMakeLists.txt` is never the source of
rows.** The rows are every matching file found under `vendor/OrcaSlicer/src/libslic3r` on disk. The
build file is read only for the one-directional cross-check described under the traps below, because
it does not name every file in the tree: at the pin the walk finds 607 files while the build file
mentions far fewer, and files such as `ExPolygonCollection.{cpp,hpp}`, `Semver.hpp`,
`TryCatchSignalSEH.{cpp,hpp}`, `AStar.hpp`, `Exception.hpp` and the `CSGMesh/` and
`TextureToColor/` headers appear in no source list. A manifest built from the build file would be
short by that many rows, and every later part of step 11 — tasks `06-manifest-scope`,
`07-crate-cut`, `14-manifest-columns` and `15-task-cuts` — would inherit the short set.

**The file set.** One row per `.h`, `.hpp`, `.cpp` or `.mm` file under
`vendor/OrcaSlicer/src/libslic3r` at the pin. It includes `MacUtils.mm`, `Format/ModelIO.hpp` and
`Format/ModelIO.mm`, which `src/libslic3r/CMakeLists.txt:604-610` appends to the library's sources
under `if (APPLE)` and which are therefore part of the library on the macOS arm64 platform this
work pins. It excludes the generated `libslic3r_version.h`, which exists in the tree only as
`libslic3r_version.h.in`. **The script asserts its row count against the set it finds on disk, and
never against a number written in a document** — no row count is hardcoded in the script, written
into the manifest, or copied out of the spec or `vendor/README.md`.

**The four columns this task derives, on every run, from the vendored tree:**

- `path`, relative to `vendor/OrcaSlicer/src/libslic3r`.
- `subsystem`: the file's top-level subdirectory under `src/libslic3r`, so that
  `Arachne/utils/...`, `Feature/FuzzySkin/...` and `Fill/Lightning/...` carry `Arachne`, `Feature`
  and `Fill` rather than their nested directory, and a file directly under `src/libslic3r` carries
  `root`. The value is derived from the tree. The fifteen accepted names are written into
  `port/inventory` as a literal list — `Algorithm`, `Arachne`, `CAD`, `CSGMesh`, `Execution`,
  `Feature`, `Fill`, `Format`, `GCode`, `Geometry`, `Optimize`, `SLA`, `Support`, `TextureBake`,
  `TextureToColor` — and the script fails, naming the directory, on any top-level subdirectory not
  in it. That is a set of names and not a count, so it does not breach this task's rule that no
  assertion is made against a number written in a document; the same fifteen names are what
  `vendor/README.md` records, and where the tree and the record disagree both are corrected in this
  task's pull request.
- `lines`, counted as `cat | wc -l`, which is how the spec counts the library.
- `file-level include-DAG level`: the file's longest-path level in the intra-library `#include`
  graph over this file set. **Verify the acyclicity rather than assume it**: run Tarjan, or an
  equivalent strongly-connected-component pass, over that graph and fail with the component printed
  if a non-trivial component exists. The column has no meaning without that property, and R8's
  claim that the graph is acyclic at file granularity rests on it. Edges come only from `#include`
  directives that resolve to a file in this set; an include of a `deps_src/`, superbuild or system
  header is neither an edge nor a row. **The SCC pass is a function taking an adjacency mapping**,
  and `port/inventory` carries a `--selftest` mode that runs it over a synthetic three-node cycle,
  so the pass's negative case is proven without a cycle being injected into the vendored tree, which
  this task may not touch.

**Three traps that change the answer. Each is handled here and asserted.** The first two are the
build file's, and neither changes a column this task produces — `path`, `subsystem`, `lines` and the
include-DAG level all come from the tree. They are handled here because the parse feeds one
assertion: **the script cross-checks that every path named in `lisbslic3r_sources` (read with the
upstream misspelling) and in the whitespace-split `libslic3r_cgal` list has a row, and fails naming
any that does not.** The cross-check runs in that direction only, because 25 files on disk are named
in no source list at the pin.

1. **The source variable is misspelled upstream.** `src/libslic3r/CMakeLists.txt:29` declares
   `set(lisbslic3r_sources`, and that spelling is used again at `:584`, `:605`, `:612` and `:615`.
   A script that looks for `libslic3r_sources` finds nothing at all, and the cross-check then passes
   over an empty list and proves nothing.
2. **The `libslic3r_cgal` list puts two files on one line.** At
   `src/libslic3r/CMakeLists.txt:633-640` each entry is a pair — `CutSurface.hpp CutSurface.cpp`,
   and the same for `IntersectionPoints`, `MeshBoolean`, `MeshRepair`, `TryCatchSignal` and
   `Triangulation` — so splitting that list on newlines loses half of them, and the cross-check then
   misses six files. Split on whitespace.
3. **Includes resolve case-sensitively.** This one does change a column — the include-DAG level.
   `Support/SupportMaterial.cpp:68` has
   `#include "utils.hpp"` inside a debug-only branch, and the tracked file is `Utils.hpp`. On this
   filesystem a case-insensitive resolver silently counts two files where there is one. The
   resolver matches the tracked name exactly, and the script asserts that no two rows' paths differ
   only by case.

**What this task does not do.** Each of these is a later part of step 11, so the list is the
boundary and not a gap: the four scope passes, the named deductions, SLA marked in or out per file,
and the `pipeline stage or none` column (task `06-manifest-scope`); the
feedback-arc cut, `port/cut.tsv`, the `crate-level level` column and the `target crate and module`
column (task `07-crate-cut`);
`just inventory-check`, the seeded and carried-forward judgement columns and the derived `status`
column (task `14-manifest-columns`); and `port/cut-tasks` with `just task-cuts <phase>` (task
`15-task-cuts`). Write those columns empty and no logic for them.

**Why this is a size 5, and why it is not split further.** The complexity is in the traps rather
than the enumeration: three separate readings of the vendored tree each change the answer silently,
and two of them produce a manifest that looks right — a `libslic3r_cgal` half-list and a
double-counted `Utils.hpp` — rather than an error, so each needs its own assertion. On top of that
the include-DAG level cannot be computed until the file-granularity graph is proven acyclic, which
is a graph pass with its own failure output. It is not split further because all of it is one pass
over one tree: the same parse of `src/libslic3r/CMakeLists.txt` that the misspelled variable and the
one-line pairs affect feeds the include graph that the case-sensitivity trap corrupts, and a split
would hand two tasks the same parser to write.

**What this task must not touch.** Nothing under `vendor/OrcaSlicer/` changes and the submodule
pointer stays at `f92a7c065c5718b8ef1baac1cd704df2911238f4`. No file list, file count or line count
is written into any document — the manifest is generated and the script's assertions are against
the tree. The manifest carries no coverage column: step 11 states there is none, because a column no
step owns drifts. Nothing is added to `.asdlc/checks` or `.github/workflows/ci.yml`; step 16 puts
`just inventory-check` in the checks once task `14-manifest-columns` has built it.

## Done when

- `port/inventory` exists with no file extension, the `uv` shebang and a PEP 723 inline metadata
  block, and `just inventory` runs it and rewrites `port/manifest.tsv`.
- Running `just inventory` on a clean tree leaves the committed `port/manifest.tsv`
  byte-identical.
- `port/manifest.tsv`'s header row holds the fifteen column names above, in that order, and every
  data row has that many tab-separated fields.
- Every row has a non-empty `path`, `subsystem`, `lines` and `file-level include-DAG level`, and
  every row's other eleven cells are empty.
- The script's row-count assertion compares against the set of `.h`, `.hpp`, `.cpp` and `.mm` files
  it finds **on disk** under `vendor/OrcaSlicer/src/libslic3r`; a reviewer reads the assertion and
  finds no literal count in it or anywhere else in the script, and finds that no row comes from
  `src/libslic3r/CMakeLists.txt`.
- `ExPolygonCollection.cpp`, `ExPolygonCollection.hpp`, `Semver.hpp`, `AStar.hpp` and
  `Exception.hpp` each have a row although no source list in `src/libslic3r/CMakeLists.txt` names
  them, which is what distinguishes the filesystem walk from a build-file parse.
- `MacUtils.mm`, `Format/ModelIO.hpp` and `Format/ModelIO.mm` each have a row, and
  `grep -c 'libslic3r_version.h' port/manifest.tsv` prints 0.
- `grep -n 'lisbslic3r_sources' port/inventory` finds the misspelled variable the cross-check reads.
- All twelve `libslic3r_cgal` files have rows: the `.hpp` and the `.cpp` of `CutSurface`,
  `IntersectionPoints`, `MeshBoolean`, `MeshRepair`, `TryCatchSignal` and `Triangulation` — and the
  cross-check reports the number of build-file paths it matched against rows, which is non-zero and
  includes all twelve.
- `awk -F'\t' '$1=="Utils.hpp"' port/manifest.tsv` prints one row, no row's path is `utils.hpp`,
  and the script's case assertion passes.
- Every row's `subsystem` is one of the fifteen names `vendor/README.md` records or `root`, and no
  nested directory name — `Arachne/utils`, `Arachne/BeadingStrategy`, `Feature/FuzzySkin`,
  `Feature/Interlocking`, `Fill/Lightning` — appears in that column.
- A normal run prints that no non-trivial strongly connected component exists over the vendored file
  set, and `port/inventory --selftest` exits non-zero on its built-in three-node cyclic graph and
  prints the component — so the pass's failure path is proven with the vendored tree untouched.
- The script prints the row count and the total line sum on every run.
- `grep -r '/Applications/' justfile oracle port` finds nothing, which completes R1's proof path
  that task `01-oracle-fetch` left at `justfile oracle` because `port/` did not exist yet.
- The branch adds only `port/inventory`, `port/manifest.tsv` and the `just inventory` recipe;
  `port/cut.tsv` does not exist and no `inventory-check` or `task-cuts` recipe is in the `justfile`.
- `git -C vendor/OrcaSlicer status --porcelain` is empty and `git -C vendor/OrcaSlicer rev-parse HEAD`
  prints `f92a7c065c5718b8ef1baac1cd704df2911238f4`.
