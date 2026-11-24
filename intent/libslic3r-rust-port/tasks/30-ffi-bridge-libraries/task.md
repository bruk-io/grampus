---
kind: task
step: 17
delivers: [R17]
depends: [29-ffi-bridge-core]
size: 5
---
# Task: Bind admesh, qhull, libnest2d and Boost.Polygon's Voronoi builder

## What to build

This is part 2 of the plan's step 17: the four remaining bridges, the smoke test that calls into
each of the five bound libraries, and the determination that NLopt is not bound. Part 1 — the
`slic3r-ffi` crate, its `build.rs`, the Clipper2 bridge and the two `just port-check` lines — is
the task this one depends on, and its `build.rs`, its `-ffp-contract=off` flag, its superbuild
prefix handling and its substitute check are extended here rather than rewritten.

Add `crates/slic3r-ffi/src/admesh.rs`, `src/qhull.rs`, `src/libnest2d.rs` and `src/voronoi.rs`,
extend `build.rs` and `tests/ffi_smoke.rs`, and complete the crate-doc table in `src/lib.rs`.
Nothing under `vendor/OrcaSlicer/` is edited or patched, and the root `Cargo.toml` is not touched.

**admesh.** Compile the six sources `deps_src/admesh/CMakeLists.txt:4-12` names — `connect.cpp`,
`normals.cpp`, `shared.cpp`, `stl_io.cpp`, `stlinit.cpp`, `util.cpp` — with both include
directories that file gives (`:14-18`): `deps_src/admesh` and its parent `deps_src`, since the
headers reach siblings through it. The vendored target links Eigen and header-only Boost
(`:20-23`), so both come from the superbuild prefix's include directory, the same prefix part 1
resolves. The bridge exposes the `stl_*` surface in `deps_src/admesh/stl.h` that the in-scope
callers reach — `src/libslic3r/TriangleMesh.cpp` is the main one.

**qhull.** The vendored CMake builds the reentrant C library and the C++ interface as one static
library for Slic3r (`deps_src/qhull/CMakeLists.txt:134-135`), from the single combined source list
at `:88-129` — fifteen `src/libqhull_r/*.c` files and nineteen `src/libqhullcpp/*.cpp` files — with
`deps_src/qhull/src` on the include path (`:145`), which is the shape
`src/libslic3r/TriangleMesh.cpp:22-29` assumes. Compile that list the same way, C and C++ units
each with `-ffp-contract=off`.

One trap decides whether this bridge is the right reference at all: `deps_src/qhull/CMakeLists.txt:13`
is `find_package(Qhull 7.2 QUIET)`, and the vendored sources are compiled only when that fails
(`:15-19`, which prints `Using qhull from system.` when it succeeds). The superbuild builds and
installs Qhull 8.0.2 (`deps/Qhull/Qhull.cmake`), and the oracle build points `CMAKE_PREFIX_PATH` at
that prefix, so the built `libslic3r` may well have linked the superbuild's Qhull rather than the
vendored copy. Determine which one it linked from the step-7 build itself — the configure output's
status line, or `Qhull_DIR` in `vendor/OrcaSlicer/build/arm64/CMakeCache.txt` — bind that one, and
record the answer with its evidence in the crate-doc table's qhull row, saying so in the pull
request, because the requirement is written expecting the vendored copy. The oracle the per-unit
proofs compare against is the library that was actually linked, not the one a document assumed.

**The two answers produce two different `build.rs` shapes, and the determination picks one**, so
that the reviewer has one thing to check rather than a contradiction:

- **Answer "vendored"** — the step-7 build compiled the vendored copy, meaning
  `find_package(Qhull 7.2 QUIET)` failed: `build.rs` compiles the combined `libqhull_r` and
  `libqhullcpp` source list the vendored CMake names at `deps_src/qhull/CMakeLists.txt:88-129`, with
  `deps_src/qhull/src` on the include path, and links no installed qhull.
- **Answer "superbuild"** — `Qhull_FOUND` was true and the build linked the superbuild's Qhull
  8.0.2 (`deps/Qhull/Qhull.cmake:3`, which fetches `v8.0.2.zip`), which is the likely answer because
  the oracle build puts that prefix on `CMAKE_PREFIX_PATH`: `build.rs` links that prefix's installed
  `qhullcpp` and `qhullstatic_r`, puts its `include` directory on the path, and compiles **no**
  vendored qhull source.

Whichever it is, the crate-doc row names it with the evidence it was read from, and the pull request
says so, because the requirement is written expecting the vendored copy.

**libnest2d.** Compile `deps_src/libnest2d/src/libnest2d.cpp` (`CMakeLists.txt:21`, `:24`) with the
four compile
definitions the vendored target makes PUBLIC (`:28`): `LIBNEST2D_THREADING_tbb`,
`LIBNEST2D_STATIC`, `LIBNEST2D_OPTIMIZER_nlopt` and `LIBNEST2D_GEOMETRIES_libslic3r`. The last of
those is why this bridge is not self-contained: libnest2d's geometry backend is libslic3r itself
(`deps_src/libnest2d/include/libnest2d/backends/libslic3r/geometries.hpp`), and the vendored target
links `libslic3r` (`:27`), so the bridge compiles against the vendored libslic3r headers and links
the library built in step 7 from `vendor/OrcaSlicer/build/arm64`, the same build the proof harness
links.

**This unit's include set is three directories, not one**, and all three are needed because that
one-line `.cpp` is `#include <libnest2d/libnest2d.hpp>` and that header pulls in a great deal under
those four definitions:

- `deps_src/libnest2d/include` (`CMakeLists.txt:26`), the library's own;
- `vendor/OrcaSlicer/src`, because under `LIBNEST2D_GEOMETRIES_libslic3r` the header's `:11` pulls
  `backends/libslic3r/geometries.hpp`, whose `:13-14` are `#include <libslic3r/ExPolygon.hpp>` and
  `#include <libslic3r/ClipperUtils.hpp>`;
- the superbuild prefix's `include` directory, because under `LIBNEST2D_OPTIMIZER_nlopt` the
  header's `:16-17` pull `optimizers/nlopt/subplex.hpp` and `genetic.hpp`, both of which reach
  `optimizers/nlopt/nlopt_boilerplate.hpp:9`, which is `#include <nlopt.hpp>` — and because Eigen,
  Boost, TBB and Clipper2 headers are reached the same way.

The surface to bind is what the in-scope callers reach, taken from `port/manifest.tsv`
rather than guessed — `MinAreaBoundingBox.cpp` through `utils/rotcalipers.hpp`,
`MeshSplitImpl.hpp`, `Support/TreeSupport.cpp` and `ExclusionVolumeGeometry.hpp` are the candidates
at the pin, while `Arrange.cpp`, `Orient.cpp`, `IMEXArrange.cpp` and the `SLA/` users are rows the
manifest marks out of scope and get no binding.

**Boost.Polygon's Voronoi builder.** There is no vendored source: Boost is a superbuild project and
the pin's own recipe fetches 1.84.0 by URL with a sha256 (`deps/Boost/Boost.cmake:43-44`), so the
bridge is bound against the headers in the superbuild prefix's include directory and against no
other Boost on the machine. Boost.Polygon is header-only, so this bridge is a translation unit of
this crate's own that includes `<boost/polygon/voronoi.hpp>` as the vendored headers do
(`src/libslic3r/Geometry/Voronoi.hpp:17`, `src/libslic3r/Arachne/SkeletalTrapezoidation.hpp:7`) and
exposes the builder over the input and diagram types `Voronoi.hpp:33-36` declares —
`voronoi_diagram<double>` with its `point_data` and `segment_data` inputs — returning its cells,
edges and vertices across the bridge. `Geometry/Voronoi.{hpp,cpp}` and its `VoronoiDiagram` class
are libslic3r files with manifest rows of their own; they are ported against this bridge in a later
task and not here.

**The NLopt determination.** NLopt is bound only if the manifest marks an in-scope caller. At the
pin the expected answer is that it is not: it is absent from
`src/libslic3r/CMakeLists.txt`'s `target_link_libraries`, the only `#include <nlopt.h>` in the
library is `src/libslic3r/Optimize/NLoptOptimizer.hpp:15`, the only file in the tree that includes
that header is `src/libslic3r/SLA/SupportTreeBuildsteps.cpp`, which the SLA deduction puts out of
scope, and the remaining reach is transitive through libnest2d's nlopt optimizer headers, included
only by `Arrange.cpp:28` and `SLA/SupportTree.cpp:19-20`, both out of scope. That is still an
outcome to establish rather than assume: check it against the committed manifest. **NLopt is
already a compile-time header dependency of the libnest2d bridge**, not only a possible link one:
`libnest2d.hpp:16-17` reach `nlopt_boilerplate.hpp:9`, which is `#include <nlopt.hpp>`, so that
translation unit needs the superbuild prefix's NLopt headers to compile at all. Record that in
NLopt's row with those two citations as its evidence, and record any link dependency the same way
if linking the bridge also pulls NLopt symbols — `LIBNEST2D_OPTIMIZER_nlopt` and `NLopt::nlopt` are
both on the vendored target. Either way NLopt's row in the crate-doc table says `not bound` with
its `path:line` evidence and names what it is reached through, and **no NLopt bridge module is
added**.

**The smoke test.** `tests/ffi_smoke.rs` ends this task with one case per bound library — Clipper2
from part 1, plus admesh, qhull, libnest2d and Boost.Polygon's Voronoi builder — each calling
through the bridge and asserting on a value the C++ produced rather than one the test computed
itself. No identifier in it matches `mock`, `stub`, `fake` or `dummy` in any case, which a later
task turns into a check, and every `unsafe` block in the crate carries a `// SAFETY:` comment. The
substitute check part 1 added stays green: no Rust substitute for any of the six is added as a
dependency anywhere in the workspace.

**Why this is a 5.** Four more bridges plus a smoke test that calls into each, and Boost.Polygon has
no vendored source, so it is bound against the superbuild prefix instead. libnest2d drags the built
libslic3r and four compile definitions in with it, qhull's own CMake can silently prefer the
superbuild's copy over the vendored one, and NLopt's expected outcome is a row saying it is not
bound, which still has to be established rather than assumed. It is not split further because the
four share one `build.rs` and one smoke test, and the NLopt answer falls out of the libnest2d link.

**What it waits for.** Part 1, for the crate, the `build.rs` with its flag and prefix handling, the
crate-doc table and the two checks; and, through it, the superbuild prefix and the built
`libslic3r`.

## Done when

- `cargo test -p slic3r-ffi` passes, and `tests/ffi_smoke.rs` has a case per library that calls
  into admesh, qhull, libnest2d and Boost.Polygon's Voronoi builder through the bridge, beside the
  Clipper2 case, each asserting on a value the C++ returned.
- `build.rs` compiles admesh's six `.cpp` files with both of its include directories.
- `build.rs`'s qhull handling matches the determination: where the step-7 build linked the vendored
  copy, it compiles the combined `libqhull_r` and `libqhullcpp` list
  (`deps_src/qhull/CMakeLists.txt:88-129`) with `deps_src/qhull/src` on the include path and links
  no installed qhull; where it linked the superbuild's Qhull 8.0.2, it links that prefix's
  `qhullcpp` and `qhullstatic_r` with its `include` directory on the path and compiles no vendored
  qhull source. One of the two holds, and the crate-doc row says which.
- `build.rs` compiles `deps_src/libnest2d/src/libnest2d.cpp` with its four compile definitions and
  all three include directories — `deps_src/libnest2d/include`, `vendor/OrcaSlicer/src` and the
  superbuild prefix's `include` — and that unit compiles, which it cannot with only the first.
- Every translation unit added here carries `-ffp-contract=off`, and `just port-check`'s flag check
  still passes.
- The Boost.Polygon translation unit is compiled with the superbuild prefix's `include` directory
  as its only Boost include path, and `build.rs` passes no `/usr/local`, `/opt/homebrew` or
  `/opt/local` include directory to any unit — which mirrors the `CMAKE_IGNORE_PREFIX_PATH` the
  oracle build uses, and is checkable from the include-path list `build.rs` emits.
- The libnest2d bridge compiles against the vendored libslic3r headers and links the library built
  in step 7 from `vendor/OrcaSlicer/build/arm64`.
- The crate-doc table in `src/lib.rs` has all six rows filled, each with its `path:line` evidence,
  and the qhull row names which qhull the step-7 build linked, with the evidence it was read from.
- NLopt's row says `not bound`, cites `libnest2d.hpp:16-17` and `nlopt_boilerplate.hpp:9` as the
  evidence that it is a compile-time header dependency of the libnest2d bridge, and says whether it
  is also linked transitively by it; no `src/nlopt.rs` or equivalent module exists.
- `just port-check` exits 0, and still exits non-zero when a `crates/*/Cargo.toml` is given a
  dependency on a substitute for any of the six libraries.
- `git -C vendor/OrcaSlicer status --porcelain` prints nothing after a full build and test run, the
  root `Cargo.toml` is unchanged, and no identifier in `tests/ffi_smoke.rs` matches `mock`, `stub`,
  `fake` or `dummy`.
