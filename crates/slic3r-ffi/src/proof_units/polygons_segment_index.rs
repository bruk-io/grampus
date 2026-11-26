//! Proof path 1 (R19, task `73-foundation-arachne-indices`) for the
//! `PolygonsSegmentIndex.hpp` unit. See `cxx/proof_units/polygons_segment_index.cpp` for how
//! the one-polygon `Polygons` is built from parallel `xs`/`ys` slices.

#[cxx::bridge(namespace = "slic3r_ffi::proof_units::polygons_segment_index")]
mod ffi {
    // SAFETY: this is cxx's own unsafe-extern declaration block, not a hand-written unsafe
    // expression (see src/clipper2.rs's identical note) -- it carries no body, and the only
    // unsafety is in the bindings `cxx` generates from it.
    unsafe extern "C++" {
        include!("cxx/proof_units/polygons_segment_index.h");

        /// `PolygonsSegmentIndex(&polys, 0, point_idx)`'s `from()`/`to()`, returning
        /// `[from.x, from.y, to.x, to.y]`.
        fn path1_from_to(xs: &[i64], ys: &[i64], point_idx: u32) -> Vec<i64>;
    }
}

pub use ffi::path1_from_to;
