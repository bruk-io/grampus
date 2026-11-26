//! Proof paths 1 and 2 (R19, task `62-foundation-polygon`) for the `MultiPoint.cpp`/
//! `MultiPoint.hpp` unit.
//!
//! Every shim drives `Slic3r::MultiPoint`'s methods through a real `Slic3r::Polygon` (the one
//! concrete, instantiable subclass this task's own units provide; `MultiPoint` itself is
//! abstract) -- see `cxx/proof_units/multi_point.cpp` for the full set and for where a call is
//! explicitly `MultiPoint::`-qualified to bypass a Polygon override of the same name.

#[cxx::bridge(namespace = "slic3r_ffi::proof_units::multi_point")]
mod ffi {
    // SAFETY: this is cxx's own unsafe-extern declaration block, not a hand-written unsafe
    // expression (see src/clipper2.rs's identical note) -- it carries no body, and the only
    // unsafety is in the bindings `cxx` generates from it.
    unsafe extern "C++" {
        include!("cxx/proof_units/multi_point.h");

        /// `MultiPoint::scale(double factor)` (`MultiPoint.cpp:17-21`), returning the resulting
        /// points as flat `[x0, y0, x1, y1, ...]`.
        fn path1_scale(xs: &[i64], ys: &[i64], factor: f64) -> Vec<i64>;
        /// `MultiPoint::translate(const Point&)` (`:32-36`).
        fn path1_translate(xs: &[i64], ys: &[i64], vx: i64, vy: i64) -> Vec<i64>;
        /// `MultiPoint::rotate(double cos_angle, double sin_angle)` (`:38-46`).
        fn path1_rotate_cos_sin(xs: &[i64], ys: &[i64], cos_angle: f64, sin_angle: f64)
            -> Vec<i64>;
        /// `MultiPoint::length() const` (`:59-67`), explicitly `MultiPoint::`-qualified in the
        /// shim since `Polygon::length()` hides it by name.
        fn path1_length(xs: &[i64], ys: &[i64]) -> f64;
        /// `MultiPoint::find_point(const Point&) const` (`:69-75`).
        fn path1_find_point(xs: &[i64], ys: &[i64], px: i64, py: i64) -> i32;
        /// `MultiPoint::bounding_box() const` (`:100-103`), returning
        /// `[min.x, min.y, max.x, max.y]`.
        fn path1_bounding_box(xs: &[i64], ys: &[i64]) -> Vec<i64>;
        /// `MultiPoint::intersection(const Line&, Point*) const` (`:132-139`), explicitly
        /// `MultiPoint::`-qualified since `Polygon::intersection` hides it; returning
        /// `[found (0/1), x, y]`.
        fn path1_intersection(
            xs: &[i64],
            ys: &[i64],
            ax: i64,
            ay: i64,
            bx: i64,
            by: i64,
        ) -> Vec<i64>;
        /// `MultiPoint::symmetric_y(const coord_t&)` (`:581-586`).
        fn path1_symmetric_y(xs: &[i64], ys: &[i64], x_axis: i64) -> Vec<i64>;
        /// `static MultiPoint::concave_hull_2d(const Points&, double)` (`:364-381`).
        fn path1_concave_hull_2d(xs: &[i64], ys: &[i64], tolerance: f64) -> Vec<i64>;
        /// `static MultiPoint::minimumDistanceBetweenLinesDefinedByPoints(const Points&, const
        /// Points&)` (`:414-434`); the only caller of the `private`
        /// `squaredDistanceToLineSegment` (`:393-404`), so this proves both.
        fn path1_min_distance_between_lines(
            a_xs: &[i64],
            a_ys: &[i64],
            b_xs: &[i64],
            b_ys: &[i64],
        ) -> f64;
        /// `get_extents_rotated(const Points&, double)` (`:550-574`), returning
        /// `[min.x, min.y, max.x, max.y, defined (0/1)]`.
        fn path1_get_extents_rotated(xs: &[i64], ys: &[i64], angle: f64) -> Vec<i64>;

        /// `MultiPoint::front()`/`back()` (`MultiPoint.hpp:45-46`, inline), returning
        /// `[front.x, front.y, back.x, back.y]`.
        fn path2_front_back(xs: &[i64], ys: &[i64]) -> Vec<i64>;
        /// `MultiPoint::closest_point(const Point&) const` (`:77`, inline), returning
        /// `[found (0/1), x, y]`.
        fn path2_closest_point(xs: &[i64], ys: &[i64], px: i64, py: i64) -> Vec<i64>;
        /// `MultiPoint::distance_to(const Point&) const` (`:78-83`, inline).
        fn path2_distance_to(xs: &[i64], ys: &[i64], px: i64, py: i64) -> f64;
        /// `MultiPoint::is_valid() const` (`:53`, inline), explicitly `MultiPoint::`-qualified
        /// since `Polygon::is_valid()` hides it.
        fn path2_is_valid(xs: &[i64], ys: &[i64]) -> bool;
        /// `inline double length(const Points &pts)` (`:181-189`), the free function.
        fn path2_length_free(xs: &[i64], ys: &[i64]) -> f64;
        /// `inline double area(const Points &polygon)` (`:191-196`), the free function.
        fn path2_area_free(xs: &[i64], ys: &[i64]) -> f64;
    }
}

pub use ffi::{
    path1_bounding_box, path1_concave_hull_2d, path1_find_point, path1_get_extents_rotated,
    path1_intersection, path1_length, path1_min_distance_between_lines, path1_rotate_cos_sin,
    path1_scale, path1_symmetric_y, path1_translate, path2_area_free, path2_closest_point,
    path2_distance_to, path2_front_back, path2_is_valid, path2_length_free,
};
