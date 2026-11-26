//! Proof paths 1 and 2 (R19, task `56-foundation-point`) for the `Point.cpp`/`Point.hpp` unit.
//!
//! `path1_int128_orient` calls the externally linked `Slic3r::int128::orient`
//! (`Point.cpp:255-260`); `path2_point_rotate` drives `Slic3r::Point::rotate(double, const
//! Point&)` directly through a real vendored `Point`. See `cxx/proof_units/point.cpp` for both.

#[cxx::bridge(namespace = "slic3r_ffi::proof_units::point")]
mod ffi {
    // SAFETY: this is cxx's own unsafe-extern declaration block, not a hand-written unsafe
    // expression (see src/clipper2.rs's identical note) -- it carries no body, and the only
    // unsafety is in the bindings `cxx` generates from it.
    unsafe extern "C++" {
        include!("cxx/proof_units/point.h");

        /// `Slic3r::int128::orient(p1, p2, p3)` (`Point.cpp:255-260`): `+1` CCW, `0` collinear,
        /// `-1` CW.
        fn path1_int128_orient(p1x: i64, p1y: i64, p2x: i64, p2y: i64, p3x: i64, p3y: i64) -> i32;

        /// `Slic3r::Point(px, py).rotate(angle, Slic3r::Point(cx, cy))`, returning `[x, y]`.
        fn path2_point_rotate(px: i64, py: i64, cx: i64, cy: i64, angle: f64) -> Vec<i64>;
    }
}

pub use ffi::{path1_int128_orient, path2_point_rotate};
