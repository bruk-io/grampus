//! `slic3r-ffi`: the `cxx` bridges to the vendored and superbuild C and C++ libraries libslic3r
//! calls directly (R17). It ports no libslic3r source file -- no per-unit completion trailer
//! belongs in this crate and it gets no `port/manifest.tsv` row of its own; the manifest's crate
//! map is what later port tasks read to know which crate calls this one.
//!
//! One row per library this crate is responsible for, filled in as each is bound. Task
//! 29-ffi-bridge-core (plan step 17, part 1) fills Clipper2's row and makes the NLopt
//! determination (recorded below as **No**, not a binding); the task it waits on nothing for and
//! that depends on it (step 17, part 2) fills the four remaining bindings -- admesh, qhull,
//! libnest2d and Boost.Polygon's Voronoi builder.
//!
//! | Library | Source at the pin | Bound | Evidence |
//! |---|---|---|---|
//! | Clipper2 | `vendor/OrcaSlicer/deps_src/clipper2/Clipper2Lib/` (vendored copy, patched) | Yes -- [`clipper2`] | `deps_src/clipper2/CMakeLists.txt:26-32` (the five sources); `Clipper2Lib/src/clipper.engine.cpp:14,32` (`tbbmalloc`); `clipper.offset.cpp:301` (round step count kept Clipper1-style), `:377` (concave corners joined at the crossing) -- the two behaviour changes that make the vendored copy unmatched by any published crate |
//! | admesh | `vendor/OrcaSlicer/deps_src/admesh/` (vendored) | Not yet -- task 29-ffi-bridge-core's dependent task | -- |
//! | qhull | superbuild (`vendor/OrcaSlicer/deps/qhull/`) | Not yet -- task 29-ffi-bridge-core's dependent task | -- |
//! | libnest2d | `vendor/OrcaSlicer/src/libnest2d/` (vendored, in-tree) | Not yet -- task 29-ffi-bridge-core's dependent task | -- |
//! | Boost.Polygon Voronoi | superbuild (`vendor/OrcaSlicer/deps/Boost/`) | Not yet -- task 29-ffi-bridge-core's dependent task | `src/libslic3r/Geometry/Voronoi.hpp:17`, `src/libslic3r/Arachne/SkeletalTrapezoidation.hpp:7` (`#include <boost/polygon/voronoi.hpp>`) |
//! | NLopt | superbuild, `deps/NLopt/NLopt.cmake` fetches v2.5.0 by URL | **No** -- determined by this task | Absent from `src/libslic3r/CMakeLists.txt`'s `target_link_libraries`; its only libslic3r includer of `Optimize/NLoptOptimizer.hpp` (which itself `#include <nlopt.h>` at `:15`) is `SLA/SupportTreeBuildsteps.cpp`, out of scope per `port/manifest.tsv`'s row for that file (`out: SLA support-tree generation's per-step algorithm; no in-scope caller invokes it.`) and R9. Reaches libslic3r only transitively through libnest2d's optimizer backends (`Arrange.cpp:28`, `SupportTree.cpp:19-20`, both `libnest2d/optimizers/nlopt/*.hpp`, not `nlopt.h` directly), which is libnest2d's binding to account for, not a reason to bind NLopt itself here |
//!
//! Any `unsafe` block this crate writes carries a safety-justification line comment (`just
//! port-status --per-commit` checks for one on every line using the word `unsafe`). As of this
//! table the crate has exactly one such block: [`clipper2`]'s `unsafe extern "C++" { ... }`
//! declaration, `cxx`'s own bridge syntax rather than a hand-written `unsafe` expression -- it
//! carries no body of its own, and the only unsafety is in the bindings `cxx` generates from it.
//! It carries its own safety-justification comment at the declaration, in `src/clipper2.rs`.
//! `port-status` selects this crate for scanning on that comment's marker alone (the crate has
//! no `port/manifest.tsv` row to select it by) and checks the marker without demanding the
//! completion trailer only a manifest row, or a trailer already written, could authorize.

pub mod clipper2;
