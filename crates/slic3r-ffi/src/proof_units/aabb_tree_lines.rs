//! Proof path 2 (R19, task `61-foundation-line-slicing`) for the `AABBTreeLines.hpp` unit.
//!
//! `path2_distance_and_outside` builds a real vendored
//! `Slic3r::AABBTreeLines::LinesDistancer<Slic3r::Linef>` over the given lines and calls its
//! real `distance_from_lines_extra<true>`/`outside`. See `cxx/proof_units/aabb_tree_lines.cpp`.

#[cxx::bridge(namespace = "slic3r_ffi::proof_units::aabb_tree_lines")]
mod ffi {
    // SAFETY: this is cxx's own unsafe-extern declaration block, not a hand-written unsafe
    // expression (see src/clipper2.rs's identical note) -- it carries no body, and the only
    // unsafety is in the bindings `cxx` generates from it.
    unsafe extern "C++" {
        include!("cxx/proof_units/aabb_tree_lines.h");

        /// `Slic3r::AABBTreeLines::LinesDistancer<Slic3r::Linef>(lines)
        /// .distance_from_lines_extra<true>(point)`/`.outside(point)`, returning `[distance,
        /// nearest_line_index, nearest_point.x, nearest_point.y, outside]`.
        fn path2_distance_and_outside(
            ax: &[f64],
            ay: &[f64],
            bx: &[f64],
            by: &[f64],
            qx: f64,
            qy: f64,
        ) -> Vec<f64>;
    }
}

pub use ffi::path2_distance_and_outside;
