//! The `cxx` bridge over Boost.Polygon's Voronoi builder (task 30-ffi-bridge-libraries, plan
//! step 17, R17).
//!
//! No vendored source: Boost is a superbuild project, and the pin's own recipe fetches 1.84.0 by
//! URL with a sha256 (`deps/Boost/Boost.cmake:43-44`). Boost.Polygon is header-only, so this
//! bridge is a translation unit of this crate's own (`cxx/voronoi_glue.cc`) that includes
//! `<boost/polygon/voronoi.hpp>` the same way the vendored headers do
//! (`src/libslic3r/Geometry/Voronoi.hpp:17`, `src/libslic3r/Arachne/SkeletalTrapezoidation.hpp:7`),
//! resolved against the superbuild prefix's `include` directory and no other Boost on the
//! machine.
//!
//! It exposes `boost::polygon::construct_voronoi` over segment input built from
//! `boost::polygon::point_data<int32_t>`/`segment_data<int32_t>` -- the 32-bit signed integer
//! range `boost/polygon/voronoi.hpp`'s own doc comment asks for -- and reads back the resulting
//! `voronoi_diagram<double>`'s cell, vertex and edge counts, not (yet) its actual cells, vertices
//! and edges. This is narrower on both ends than `Voronoi.hpp:200-207` feeds the builder in
//! production: `segment_traits<VoronoiDiagram::Segment>` over 64-bit `Slic3r::Point`
//! (`coord_t = int64_t`, `libslic3r.h:48`), through its own traits specialization, not the
//! `point_data<double>`/`segment_data<double>` aliases `Voronoi.hpp:33-36` itself declares (those
//! are `voronoi_diagram_type::coordinate_type`-parameterized, i.e. double, matching the output
//! diagram, not this bridge's int32 input). The int32 choice is deliberate -- it is the type the
//! builder's own doc comment is written for -- but it is this bridge's own narrower surface, not
//! an aliasing of those lines; a later task that ports `Geometry/Voronoi.{hpp,cpp}` and its
//! `VoronoiDiagram` class against this bridge will need to extend it to the int64 input and to
//! the diagram's actual topology, the same way `admesh.rs` and `clipper2.rs` grow their own bound
//! surface per ported unit rather than up front.

#[cxx::bridge(namespace = "slic3r_ffi::voronoi")]
mod ffi {
    /// One input segment: `boost::polygon::segment_data<int32_t>`'s two endpoints.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    struct CSegment {
        x0: i32,
        y0: i32,
        x1: i32,
        y1: i32,
    }

    /// The counts read back from a built `voronoi_diagram<double>`: how many cells, vertices and
    /// edges the library itself produced.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    struct CVoronoiStats {
        cell_count: i32,
        vertex_count: i32,
        edge_count: i32,
    }

    // SAFETY: cxx's own unsafe-extern declaration block (see clipper2.rs's module doc for why
    // this is not a hand-written unsafe expression) -- the only unsafety is in the bindings cxx
    // generates from it.
    unsafe extern "C++" {
        include!("cxx/voronoi_shim.h");

        /// `boost::polygon::construct_voronoi(segments.begin(), segments.end(), &vd)` over the
        /// given segments, then `vd.num_cells()` / `num_vertices()` / `num_edges()` -- counts
        /// the builder itself produced, not ones restated here.
        fn voronoi_diagram_stats(segments: &Vec<CSegment>) -> CVoronoiStats;
    }
}

pub use ffi::{CSegment, CVoronoiStats};

/// The cell, vertex and edge counts `boost::polygon::construct_voronoi` produces for the given
/// segments.
pub fn diagram_stats(segments: &[CSegment]) -> CVoronoiStats {
    ffi::voronoi_diagram_stats(&segments.to_vec())
}
