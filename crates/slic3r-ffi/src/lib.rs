//! `slic3r-ffi`: the `cxx` bridges to the vendored and superbuild C and C++ libraries libslic3r
//! calls directly (R17). It ports no libslic3r source file -- no per-unit completion trailer
//! belongs in this crate and it gets no `port/manifest.tsv` row of its own; the manifest's crate
//! map is what later port tasks read to know which crate calls this one.
//!
//! One row per library this crate is responsible for. Task 29-ffi-bridge-core (plan step 17,
//! part 1) filled Clipper2's row and made the NLopt determination (recorded below as **No**, not
//! a binding); task 30-ffi-bridge-libraries (step 17, part 2) fills the four remaining bindings
//! -- admesh, qhull, libnest2d and Boost.Polygon's Voronoi builder.
//!
//! | Library | Source at the pin | Bound | Evidence |
//! |---|---|---|---|
//! | Clipper2 | `vendor/OrcaSlicer/deps_src/clipper2/Clipper2Lib/` (vendored copy, patched) | Yes -- [`clipper2`] | `deps_src/clipper2/CMakeLists.txt:26-32` (the five sources); `Clipper2Lib/src/clipper.engine.cpp:14,32` (`tbbmalloc`); `clipper.offset.cpp:301` (round step count kept Clipper1-style), `:377` (concave corners joined at the crossing) -- the two behaviour changes that make the vendored copy unmatched by any published crate |
//! | admesh | `vendor/OrcaSlicer/deps_src/admesh/` (vendored) | Yes -- [`admesh`] | `deps_src/admesh/CMakeLists.txt:4-23` (the six sources, the two include directories, Eigen/Boost-headeronly); `stl.h:402` (`stl_calculate_volume`, the bound surface) |
//! | qhull | superbuild (`deps/build/arm64/OrcaSlicer_dep/usr/local`, Qhull 8.0.2) -- **not** the vendored copy | Yes -- [`qhull`] | **Determination:** `deps_src/qhull/CMakeLists.txt:13,17` -- vendored qhull (the combined source list at `:88-129`, built at `:134-135`) is compiled only when `find_package(Qhull 7.2 QUIET)` fails (`if(Qhull_FOUND)` at `:17`); the step-7 configure log (`oracle/.cache/main-oracle-build.log`, libslic3r's own configure block) prints `-- Using qhull from system.`, the exact message line `:19` emits on success, and `vendor/OrcaSlicer/build/arm64/CMakeCache.txt`'s `Qhull_DIR` points at `deps/build/arm64/OrcaSlicer_dep/usr/local/lib/cmake/Qhull` (the superbuild's `deps/Qhull/Qhull.cmake:3`, which fetches `v8.0.2.zip`) -- so the step-7 build linked the superbuild's Qhull, not the vendored copy. `build.rs` links that prefix's `qhullcpp`/`qhullstatic_r` -- since task 31-proof-harness, through the link line it derives from the step-7 build tree's own `libslic3r_tests` link edge, where both archives appear by their absolute path under that prefix, which is the determination restated by the build itself rather than by this crate -- and puts the prefix's `include` on the path; it compiles no vendored qhull source. (The requirement is written expecting the vendored copy; this row records the actual answer and its evidence instead, per this task's instructions -- and R17's own text has not been updated to match; see the pull request body.) |
//! | libnest2d | `vendor/OrcaSlicer/deps_src/libnest2d/` (vendored, in-tree; links the built `libslic3r`) | Yes -- [`libnest2d`] | `deps_src/libnest2d/CMakeLists.txt:21-28` (the one source, its four compile definitions); `placers/nfpplacer.hpp:152-336` (`EdgeCache<RawShape>`, the class the one in-scope caller, `src/libslic3r/Support/TreeSupport.cpp:3834`, `port/manifest.tsv:520`, reaches) -- bound so far: `circumference()` (`:312`) and `coords(double)` (`:303`), not yet `holeCount()` (`:334`), `circumference(unsigned)` (`:316`) or `coords(unsigned, double)` (`:307`), which the same caller also calls (`TreeSupport.cpp:3835-3839`) and which this bridge carries no hole input to reach; see `libnest2d.rs`'s module doc |
//! | Boost.Polygon Voronoi | superbuild (`vendor/OrcaSlicer/deps/Boost/`) | Yes -- [`voronoi`] | `src/libslic3r/Geometry/Voronoi.hpp:17`, `src/libslic3r/Arachne/SkeletalTrapezoidation.hpp:7` (`#include <boost/polygon/voronoi.hpp>`); bound surface is `construct_voronoi` over `point_data<int32_t>`/`segment_data<int32_t>` input (the builder's own documented 32-bit range) producing a `voronoi_diagram<double>` (`Voronoi.hpp:33-36`'s own `voronoi_diagram_type` alias), read back as cell/vertex/edge counts, not (yet) their data -- narrower than the int64 `segment_traits<VoronoiDiagram::Segment>` input `Voronoi.hpp:200-207` actually feeds the builder in production; see `voronoi.rs`'s module doc |
//! | NLopt | superbuild, `deps/NLopt/NLopt.cmake` fetches v2.5.0 by URL | **No** -- determined by task 29-ffi-bridge-core | Absent from `src/libslic3r/CMakeLists.txt`'s `target_link_libraries`; its only libslic3r includer of `Optimize/NLoptOptimizer.hpp` (which itself `#include <nlopt.h>` at `:15`) is `SLA/SupportTreeBuildsteps.cpp`, out of scope per `port/manifest.tsv`'s row for that file (`out: SLA support-tree generation's per-step algorithm; no in-scope caller invokes it.`) and R9. **It is a compile-time header dependency of the libnest2d bridge regardless:** `libnest2d.hpp:16-17` (`LIBNEST2D_OPTIMIZER_nlopt`) reach `optimizers/nlopt/nlopt_boilerplate.hpp:9`, `#include <nlopt.hpp>`, so `libnest2d.rs`'s translation unit needs the superbuild prefix's NLopt headers to compile at all. **It is linked transitively too, confirmed, not inferred:** `nm -u` on the compiled (unmodified, vendored) `libnest2d.cpp` object file lists eighteen undefined `_nlopt_*` symbols (`_nlopt_create`, `_nlopt_optimize`, `_nlopt_set_min_objective`, ...), so `build.rs` links the superbuild's `libnlopt.a` to satisfy real undefined symbols the explicit `_Nester` instantiation's compiled code carries, not a precaution. Neither makes NLopt itself bound: no `src/nlopt.rs` or equivalent module exists, and no libslic3r caller in scope reaches `nlopt.h` directly. |
//!
//! Any `unsafe` block this crate writes carries a safety-justification line comment (`just
//! port-status --per-commit` checks for one on every line using the word `unsafe`). Every module
//! here follows [`clipper2`]'s pattern: the only `unsafe` is `cxx`'s own `unsafe extern "C++"
//! { ... }` declaration block, not a hand-written `unsafe` expression -- it carries no body of
//! its own, and the only unsafety is in the bindings `cxx` generates from it. Each module carries
//! its own safety-justification comment at its declaration. `port-status` selects this crate for
//! scanning on that comment's marker alone (the crate has no `port/manifest.tsv` row to select it
//! by) and checks the marker without demanding the completion trailer only a manifest row, or a
//! trailer already written, could authorize.

pub mod admesh;
pub mod clipper2;
pub mod libnest2d;
// Not a library binding and so not a row in the table above: the per-unit proof bridges (R19,
// plan step 18), one module per ported unit, each following the same `unsafe extern "C++"`
// pattern. The module list is generated by `build.rs` from the files present in
// `src/proof_units/`; see `proof_units.rs` for the three files a port task creates and why
// nothing shared is appended to.
pub mod proof_units;
pub mod qhull;
pub mod voronoi;
