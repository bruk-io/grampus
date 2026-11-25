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
//! The module (and bridge namespace) is named `nest2d`, not `libnest2d`: see `cxx/
//! libnest2d_shim.h`'s header comment for why the identical name would shadow the vendored
//! library's own top-level `::libnest2d` namespace from inside this bridge's `.cc`.
//!
//! The surface bound is `libnest2d::placers::EdgeCache<ExPolygon>`
//! (`deps_src/libnest2d/include/libnest2d/placers/nfpplacer.hpp:152-336`), the one in-scope
//! caller reaches (`src/libslic3r/Support/TreeSupport.cpp:43-44,3834`, `port/manifest.tsv:520`,
//! `in`). `MinAreaBoundingBox.{cpp,hpp}` (manifest:378-379, no caller in the four entry points'
//! closure) and the arrangement callers `Arrange.cpp`/`Orient.cpp`/`IMEXArrange.cpp`/
//! `ModelArrange.cpp` (manifest: out, placement fixed in the fixture) are not bound here.
//! `MeshSplitImpl.hpp` only reaches `libnest2d/tools/benchmark.h`, a timer utility with no
//! geometry surface of its own.

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
    /// (`EdgeCache::circumference()`) and the point at relative distance 0.0 along it
    /// (`EdgeCache::coords(0.0)`), which for a closed contour is its first vertex.
    #[derive(Clone, Copy, Debug, PartialEq)]
    struct CEdgeCacheStats {
        circumference: f64,
        first_point: CPoint,
    }

    // SAFETY: cxx's own unsafe-extern declaration block (see clipper2.rs's module doc for why
    // this is not a hand-written unsafe expression) -- the only unsafety is in the bindings cxx
    // generates from it.
    unsafe extern "C++" {
        include!("cxx/libnest2d_shim.h");

        /// Builds a single-contour `Slic3r::ExPolygon` from `contour`, constructs
        /// `libnest2d::placers::EdgeCache<ExPolygon>` over it (`nfpplacer.hpp:286`), and reads
        /// back its circumference and first point -- values `EdgeCache` computed by walking the
        /// polygon's own edges, not ones restated here.
        fn libnest2d_edge_cache_stats(contour: &Vec<CPoint>) -> CEdgeCacheStats;
    }
}

pub use ffi::{CEdgeCacheStats, CPoint};

/// `EdgeCache<ExPolygon>`'s circumference and first point for the closed contour `points`
/// describes.
pub fn edge_cache_stats(points: &[CPoint]) -> CEdgeCacheStats {
    ffi::libnest2d_edge_cache_stats(&points.to_vec())
}
