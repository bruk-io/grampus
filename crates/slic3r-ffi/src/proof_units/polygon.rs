//! Proof path 3 (R19, task `62-foundation-polygon`) for the `Polygon.cpp`/`Polygon.hpp` unit --
//! one cell across both rows, so every function below shares this one route rather than some
//! going through an ordinary header-include shim: `point_in_polygon` (`Polygon.cpp:738-766`,
//! file-scope `static`) is the one symbol with no external linkage to call any other way, and
//! is reached only by `#include`ing `Polygon.cpp` whole into `cxx/proof_units/polygon.cpp` --
//! see that file for the duplicate-symbol measurement this `#include` needed before it was
//! written. Every other function here is driven through the same real `Slic3r::Polygon`, built
//! inside that same translation unit.

#[cxx::bridge(namespace = "slic3r_ffi::proof_units::polygon")]
mod ffi {
    // SAFETY: this is cxx's own unsafe-extern declaration block, not a hand-written unsafe
    // expression (see src/clipper2.rs's identical note) -- it carries no body, and the only
    // unsafety is in the bindings `cxx` generates from it.
    unsafe extern "C++" {
        include!("cxx/proof_units/polygon.h");

        /// `Slic3r::point_in_polygon(Point(px,py), path)` (`Polygon.cpp:738-766`), where `path`
        /// is built from the parallel `xs`/`ys` slices (`cxx` has no `Vec<(i64, i64)>`, the same
        /// convention `cxx/proof_units/bounding_box.cpp`'s `path2_merge_points` uses): `1` inside,
        /// `0` outside, `-1` on the boundary.
        fn path3_point_in_polygon(px: i64, py: i64, xs: &[i64], ys: &[i64]) -> i32;
        /// `Polygon::area() const` (`Polygon.cpp:79-82`).
        fn path3_area(xs: &[i64], ys: &[i64]) -> f64;
        /// `Polygon::is_counter_clockwise() const` (`:84-87`).
        fn path3_is_counter_clockwise(xs: &[i64], ys: &[i64]) -> bool;
        /// `Polygon::centroid() const` (`:154-169`), returning `[x, y]`.
        fn path3_centroid(xs: &[i64], ys: &[i64]) -> Vec<i64>;
        /// `Polygon::contains(const Point&) const` (`Polygon.hpp:73`).
        fn path3_contains(xs: &[i64], ys: &[i64], px: i64, py: i64) -> bool;
        /// `Polygon::point_projection(const Point&) const` (`Polygon.cpp:334-369`), returning
        /// `[x, y]`.
        fn path3_point_projection(xs: &[i64], ys: &[i64], px: i64, py: i64) -> Vec<i64>;
        /// `polygon_is_convex(const Points&)` (`:477-493`).
        fn path3_polygon_is_convex(xs: &[i64], ys: &[i64]) -> bool;
        /// `make_circle_num_segments(double, size_t)` (`:796-806`), returning flat
        /// `[x0, y0, x1, y1, ...]`.
        fn path3_make_circle_num_segments(radius: f64, num_segments: u64) -> Vec<i64>;
        /// `remove_sticks(Polygon&)` (`:567-591`), returning `[modified (0/1), x0, y0, ...]`.
        fn path3_remove_sticks(xs: &[i64], ys: &[i64]) -> Vec<i64>;
        /// `Polygon::parameter_by_length() const` (`:371-379`).
        fn path3_parameter_by_length(xs: &[i64], ys: &[i64]) -> Vec<f32>;
    }
}

pub use ffi::{
    path3_area, path3_centroid, path3_contains, path3_is_counter_clockwise,
    path3_make_circle_num_segments, path3_parameter_by_length, path3_point_in_polygon,
    path3_point_projection, path3_polygon_is_convex, path3_remove_sticks,
};
