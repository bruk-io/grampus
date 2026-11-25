//! The `cxx` bridge over libnest2d (task 30-ffi-bridge-libraries, plan step 17, R17).
//!
//! `deps_src/libnest2d/src/libnest2d.cpp` -- the vendored source this crate's `build.rs`
//! compiles unmodified -- is `#include <libnest2d/libnest2d.hpp>` plus explicit template
//! instantiations of `_Nester<NfpPlacer, FirstFitSelection>` and
//! `_Nester<BottomLeftPlacer, FirstFitSelection>` (`CMakeLists.txt:21,24`). Under
//! `LIBNEST2D_GEOMETRIES_libslic3r` those instantiations generate real calls into libslic3r's
//! own compiled geometry functions (`backends/libslic3r/geometries.hpp` routes `ClipperUtils`,
//! `ExPolygon`, ...), which is why `build.rs` links the library step 7 built
//! (`vendor/OrcaSlicer/build/arm64/src/libslic3r/Release/liblibslic3r.a`) and its own dependency
//! chain, not only this crate's own glue.
//!
//! This file (and Rust module) is named `libnest2d.rs`, matching the task's own naming, but the
//! `cxx` bridge *namespace* below is `slic3r_ffi::nest2d`, not `slic3r_ffi::libnest2d`: see
//! `cxx/libnest2d_shim.h`'s header comment for why the identical namespace name would shadow the
//! vendored library's own top-level `::libnest2d` namespace from inside this bridge's `.cc`. The
//! namespace is a C++-side concern independent of the Rust file/module name.
//!
//! The entry point into `libnest2d::placers::EdgeCache<ExPolygon>`
//! (`deps_src/libnest2d/include/libnest2d/placers/nfpplacer.hpp:152-336`), the class the one
//! in-scope caller reaches (`src/libslic3r/Support/TreeSupport.cpp:3834`, `port/manifest.tsv:520`,
//! `in`). `MinAreaBoundingBox.{cpp,hpp}` (manifest:378-379, no caller in the four entry points'
//! closure) and the arrangement callers `Arrange.cpp`/`Orient.cpp`/`IMEXArrange.cpp`/
//! `ModelArrange.cpp` (manifest: out, placement fixed in the fixture) are not bound here.
//! `MeshSplitImpl.hpp` only reaches `libnest2d/tools/benchmark.h`, a timer utility with no
//! geometry surface of its own.
//!
//! **Not the whole class, and not yet all five members `TreeSupport.cpp:3835-3839` calls on it.**
//! This bridge binds the contour's `circumference()` (`nfpplacer.hpp:312`) and `coords(double)`
//! (`:303`), the same way `clipper2.rs` and `admesh.rs` grow their own bound surface per ported
//! unit rather than up front. `holeCount()` (`:334`), the hole-indexed `circumference(unsigned)`
//! (`:316`) and `coords(unsigned, double)` (`:307`) the caller also reaches are not bound here --
//! this bridge carries no hole input to call them with. They grow in with the `TreeSupport.cpp`
//! port, which is the task that needs them.

#[cxx::bridge(namespace = "slic3r_ffi::nest2d")]
mod ffi {
    /// One vertex of the `ExPolygon` this bridge builds -- `Slic3r::Point`'s `coord_t` is
    /// `int64_t` (`libslic3r.h:48`).
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    struct CPoint {
        x: i64,
        y: i64,
    }

    /// What `EdgeCache` itself computed: the contour's total circumference
    /// (`EdgeCache::circumference()`) and the point at the given relative distance along it
    /// (`EdgeCache::coords(distance)`), which at distance 0.0 is its first vertex.
    #[derive(Clone, Copy, Debug, PartialEq)]
    struct CEdgeCacheStats {
        circumference: f64,
        point_at_distance: CPoint,
    }

    // SAFETY: cxx's own unsafe-extern declaration block (see clipper2.rs's module doc for why
    // this is not a hand-written unsafe expression) -- the only unsafety is in the bindings cxx
    // generates from it.
    unsafe extern "C++" {
        include!("cxx/libnest2d_shim.h");

        /// Builds a single-contour `Slic3r::ExPolygon` from `contour`, constructs
        /// `libnest2d::placers::EdgeCache<ExPolygon>` over it (`nfpplacer.hpp:286`), and reads
        /// back its circumference and the point at `distance` along it -- values `EdgeCache`
        /// computed by walking the polygon's own edges, not ones restated here. `distance` is
        /// expected in `<0.0, 1.0>` (`nfpplacer.hpp:303`'s own doc comment); a contour of fewer
        /// than two points returns zeroed stats rather than reaching `EdgeCache` at all (see
        /// `libnest2d_glue.cc`).
        fn libnest2d_edge_cache_stats(contour: &Vec<CPoint>, distance: f64) -> CEdgeCacheStats;
    }
}

pub use ffi::{CEdgeCacheStats, CPoint};

/// `EdgeCache<ExPolygon>`'s circumference and the point at `distance` along the closed contour
/// `points` describes (`distance` in `<0.0, 1.0>`, 0.0 being the first vertex).
pub fn edge_cache_stats(points: &[CPoint], distance: f64) -> CEdgeCacheStats {
    ffi::libnest2d_edge_cache_stats(&points.to_vec(), distance)
}
