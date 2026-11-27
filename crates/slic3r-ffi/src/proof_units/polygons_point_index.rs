//! Proof path 1 (R19, task `73-foundation-arachne-indices`) for the `PolygonsPointIndex.hpp`
//! unit. See `cxx/proof_units/polygons_point_index.cpp` for how the one-polygon `Polygons` is
//! built from parallel `xs`/`ys` slices.

#[cxx::bridge(namespace = "slic3r_ffi::proof_units::polygons_point_index")]
mod ffi {
    // SAFETY: this is cxx's own unsafe-extern declaration block, not a hand-written unsafe
    // expression (see src/clipper2.rs's identical note) -- it carries no body, and the only
    // unsafety is in the bindings `cxx` generates from it.
    unsafe extern "C++" {
        include!("cxx/proof_units/polygons_point_index.h");

        /// `PolygonsPointIndex(&polys, 0, point_idx)`'s `p()`/`next().p()`/`prev().p()`,
        /// returning `[p.x, p.y, next.x, next.y, prev.x, prev.y]`.
        fn path1_p_next_prev(xs: &[i64], ys: &[i64], point_idx: u32) -> Vec<i64>;
    }
}

pub use ffi::path1_p_next_prev;
