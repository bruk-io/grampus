//! Proof path 2 (R19, task `60-foundation-trees-grids`) for the `KDTreeIndirect.hpp` unit.
//!
//! `path2_find_closest_point`/`path2_find_nearby_points` each build a real
//! `Slic3r::KDTreeIndirect<2, double, ...>` over the same point set the Rust side builds its own
//! tree from, and call the real vendored `Slic3r::find_closest_point`/`Slic3r::find_nearby_points`
//! free functions -- see `cxx/proof_units/kd_tree_indirect.cpp`.

#[cxx::bridge(namespace = "slic3r_ffi::proof_units::kd_tree_indirect")]
mod ffi {
    // SAFETY: this is cxx's own unsafe-extern declaration block, not a hand-written unsafe
    // expression (see src/clipper2.rs's identical note) -- it carries no body, and the only
    // unsafety is in the bindings `cxx` generates from it.
    unsafe extern "C++" {
        include!("cxx/proof_units/kd_tree_indirect.h");

        /// `Slic3r::find_closest_point(tree, query)` over a tree built from `coords`
        /// (`x0,y0,x1,y1,...`).
        fn path2_find_closest_point(coords: &Vec<f64>, qx: f64, qy: f64) -> u64;

        /// `Slic3r::find_nearby_points(tree, center, max_distance)` over the same tree shape.
        fn path2_find_nearby_points(
            coords: &Vec<f64>,
            cx: f64,
            cy: f64,
            max_distance: f64,
        ) -> Vec<u64>;
    }
}

pub use ffi::{path2_find_closest_point, path2_find_nearby_points};
