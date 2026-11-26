//! Proof paths 1 and 2 (R19, task `61-foundation-line-slicing`) for the `Line.cpp`/`Line.hpp`
//! unit.
//!
//! `path1_intersection_infinite` calls the externally linked `Slic3r::Line::intersection_infinite`
//! (`Line.cpp:27-46`); `path2_line_length`/`path2_line_midpoint_normal` drive `Line.hpp`'s own
//! header-only `length()`/`midpoint()`/`normal()` directly through a real vendored `Line`. See
//! `cxx/proof_units/line.cpp` for both.

#[cxx::bridge(namespace = "slic3r_ffi::proof_units::line")]
mod ffi {
    // SAFETY: this is cxx's own unsafe-extern declaration block, not a hand-written unsafe
    // expression (see src/clipper2.rs's identical note) -- it carries no body, and the only
    // unsafety is in the bindings `cxx` generates from it.
    unsafe extern "C++" {
        include!("cxx/proof_units/line.h");

        /// `Slic3r::Line(Point(ax,ay), Point(bx,by)).intersection_infinite(Line(Point(cx,cy),
        /// Point(dx,dy)), &point)` (`Line.cpp:27-46`), returning `[hit, x, y]` (`hit` is `0` or
        /// `1`; `x`/`y` are `0` when `hit == 0`).
        fn path1_intersection_infinite(
            ax: i64,
            ay: i64,
            bx: i64,
            by: i64,
            cx: i64,
            cy: i64,
            dx: i64,
            dy: i64,
        ) -> Vec<i64>;

        /// `Slic3r::Line(Point(ax,ay), Point(bx,by)).length()` (`Line.hpp:166`).
        fn path2_line_length(ax: i64, ay: i64, bx: i64, by: i64) -> f64;

        /// `Slic3r::Line(Point(ax,ay), Point(bx,by)).midpoint()`/`.normal()` (`Line.hpp:167,183`),
        /// returning `[midpoint.x, midpoint.y, normal.x, normal.y]`.
        fn path2_line_midpoint_normal(ax: i64, ay: i64, bx: i64, by: i64) -> Vec<i64>;

        /// `Slic3r::Line(Point(ax,ay), Point(bx,by)).distance_to_infinite_squared(Point(px,py),
        /// &closest)` (`Line.hpp:173`), returning `[sq_dist, closest.x, closest.y]` -- added on
        /// review: this function had no proof of any kind before.
        fn path2_line_distance_to_infinite_squared(
            ax: i64,
            ay: i64,
            bx: i64,
            by: i64,
            px: i64,
            py: i64,
        ) -> Vec<f64>;
    }
}

pub use ffi::{
    path1_intersection_infinite, path2_line_distance_to_infinite_squared, path2_line_length,
    path2_line_midpoint_normal,
};
