//! The `cxx` bridge over qhull (task 30-ffi-bridge-libraries, plan step 17, R17).
//!
//! **Which qhull.** `deps_src/qhull/CMakeLists.txt:13` is `find_package(Qhull 7.2 QUIET)`, and
//! the vendored sources at `:20-137` are compiled only when that lookup fails (`:15`,
//! `Qhull_FOUND`). The step-7 build's own configure log
//! (`oracle/.cache/main-oracle-build.log`, the `-- Using qhull from system.` line inside the
//! `libslic3r` configure block, not the dependency superbuild's own `dep_Qhull` build earlier in
//! the same log) prints exactly the message `deps_src/qhull/CMakeLists.txt:17` emits on success,
//! and `vendor/OrcaSlicer/build/arm64/CMakeCache.txt`'s `Qhull_DIR` points at
//! `deps/build/arm64/OrcaSlicer_dep/usr/local/lib/cmake/Qhull` -- the superbuild's own install,
//! which built Qhull 8.0.2 (`deps/Qhull/Qhull.cmake:3`, `v8.0.2.zip`). So the step-7 build linked
//! the superbuild's Qhull, not the vendored copy: this bridge links that prefix's installed
//! `qhullcpp` and `qhullstatic_r` (`build.rs`), puts its `include` directory on the path, and
//! compiles no vendored qhull source.
//!
//! It exposes the surface `src/libslic3r/TriangleMesh.cpp:1364-1409` reaches:
//! `orgQhull::Qhull::runQhull` over a 3D point cloud with the `"Qt"` (triangulate) option, then
//! the resulting `facetList()`/`vertexList()`.

#[cxx::bridge(namespace = "slic3r_ffi::qhull")]
mod ffi {
    /// One input point for `orgQhull::Qhull::runQhull`'s flat `realT` coordinate array.
    #[derive(Clone, Copy, Debug, PartialEq)]
    struct CPoint3 {
        x: f64,
        y: f64,
        z: f64,
    }

    /// The counts read back from a qhull run: how many facets and vertices the library itself
    /// produced.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    struct CHullStats {
        facet_count: i32,
        vertex_count: i32,
    }

    // SAFETY: cxx's own unsafe-extern declaration block (see clipper2.rs's module doc for why
    // this is not a hand-written unsafe expression) -- the only unsafety is in the bindings cxx
    // generates from it.
    unsafe extern "C++" {
        include!("cxx/qhull_shim.h");

        /// `orgQhull::Qhull::runQhull("", 3, points.len(), coords, "Qt")`
        /// (`TriangleMesh.cpp:1365-1372`), then `facetList().count()` / `vertexList().count()`.
        fn qhull_convex_hull_stats(points: &Vec<CPoint3>) -> CHullStats;
    }
}

pub use ffi::{CHullStats, CPoint3};

/// The facet and vertex counts qhull's own triangulated convex hull produces for `points`.
pub fn convex_hull_stats(points: &[CPoint3]) -> CHullStats {
    ffi::qhull_convex_hull_stats(&points.to_vec())
}
