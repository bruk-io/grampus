//! Proof path 2 (R19, task `60-foundation-trees-grids`) for the `AABBTreeIndirect.hpp` unit.
//!
//! Both functions build a real `Slic3r::AABBTreeIndirect::Tree<3, double>` over the same
//! flattened vertex/face arrays the Rust side builds its own tree from, and call the real
//! vendored `Slic3r::AABBTreeIndirect::intersect_ray_first_hit`/
//! `squared_distance_to_indexed_triangle_set` free functions -- see
//! `cxx/proof_units/aabb_tree_indirect.cpp`.

#[cxx::bridge(namespace = "slic3r_ffi::proof_units::aabb_tree_indirect")]
mod ffi {
    /// `(squared_distance, hit_idx)` -- a `cxx` bridge struct carrying only plain values (the
    /// one shape decision 1 of `docs/PORTING.md`'s "Decisions recorded on 2026-11-25" sanctions
    /// for crossing the bridge, the same shape `clipper_z_utils.rs`'s `ProofZPoint` uses).
    struct PathTwoSquaredDistance {
        sq: f64,
        idx: i64,
    }

    // SAFETY: this is cxx's own unsafe-extern declaration block, not a hand-written unsafe
    // expression (see src/clipper2.rs's identical note) -- it carries no body, and the only
    // unsafety is in the bindings `cxx` generates from it.
    unsafe extern "C++" {
        include!("cxx/proof_units/aabb_tree_indirect.h");

        /// `Slic3r::AABBTreeIndirect::intersect_ray_first_hit(vertices, faces, tree, origin,
        /// dir, hit, eps)`, returning the hit face index or `-1` for no hit.
        fn path2_intersect_ray_first_hit(
            vertices: &Vec<f64>,
            faces: &Vec<i64>,
            ox: f64,
            oy: f64,
            oz: f64,
            dx: f64,
            dy: f64,
            dz: f64,
        ) -> i64;

        /// `Slic3r::AABBTreeIndirect::squared_distance_to_indexed_triangle_set(vertices, faces,
        /// tree, point, hit_idx_out, hit_point_out)`.
        fn path2_squared_distance(
            vertices: &Vec<f64>,
            faces: &Vec<i64>,
            px: f64,
            py: f64,
            pz: f64,
        ) -> PathTwoSquaredDistance;
    }
}

pub use ffi::{path2_intersect_ray_first_hit, PathTwoSquaredDistance};

/// `path2_squared_distance` re-exported as `(f64, i64)`, so the Rust-side test compares plain
/// values rather than a bridge struct -- see `tiers.rs`'s own note on why `ProofValue` is not
/// implemented for a bridge type directly.
pub fn path2_squared_distance(
    vertices: &Vec<f64>,
    faces: &Vec<i64>,
    px: f64,
    py: f64,
    pz: f64,
) -> (f64, i64) {
    let r = ffi::path2_squared_distance(vertices, faces, px, py, pz);
    (r.sq, r.idx)
}
