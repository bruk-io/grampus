//! Proof path 3's cell, reached through the externally-linked surface (R19, task
//! `66-foundation-geometry`) -- see `cxx/proof_units/geometry.h`'s own comment for why.

#[cxx::bridge(namespace = "slic3r_ffi::proof_units::geometry")]
mod ffi {
    // SAFETY: this is cxx's own unsafe-extern declaration block, not a hand-written unsafe
    // expression (see src/clipper2.rs's identical note) -- it carries no body, and the only
    // unsafety is in the bindings `cxx` generates from it.
    unsafe extern "C++" {
        include!("cxx/proof_units/geometry.h");

        /// `Slic3r::Geometry::orient(const Point&, const Point&, const Point&)`
        /// (`Geometry.hpp:44-61`), returning the `Orientation` enum's own discriminant.
        fn path_orient(ax: i64, ay: i64, bx: i64, by: i64, cx: i64, cy: i64) -> i32;
        /// `Slic3r::Geometry::is_ccw(const Polygon&)` (`:66-90`), driven through a real
        /// `Slic3r::Polygon` built from the flat point slices.
        fn path_is_ccw(xs: &[i64], ys: &[i64]) -> bool;
        /// `Slic3r::Geometry::segments_intersect(const Point&, const Point&, const Point&, const
        /// Point&)` (`:134-184`).
        #[allow(clippy::too_many_arguments)]
        fn path_segments_intersect(
            ip1x: i64,
            ip1y: i64,
            ip2x: i64,
            ip2y: i64,
            jp1x: i64,
            jp1y: i64,
            jp2x: i64,
            jp2y: i64,
        ) -> bool;
        /// `Slic3r::Geometry::directions_parallel(double, double, double)`
        /// (`Geometry.cpp:40-45`).
        fn path_directions_parallel(angle1: f64, angle2: f64, max_diff: f64) -> bool;
        /// `Slic3r::Geometry::directions_perpendicular(double, double, double)` (`:47-52`).
        fn path_directions_perpendicular(angle1: f64, angle2: f64, max_diff: f64) -> bool;
        /// `Slic3r::Geometry::rad2deg_dir(double)` (`:64-69`).
        fn path_rad2deg_dir(angle: f64) -> f64;
        /// `Slic3r::Geometry::linint(double, double, double, double, double)` (`:84-87`).
        fn path_linint(value: f64, oldmin: f64, oldmax: f64, newmin: f64, newmax: f64) -> f64;
        /// `Slic3r::Geometry::arrange(size_t, const Vec2d&, coordf_t, const BoundingBoxf*,
        /// Pointfs&)` (`:171-293`), returning `[ok (0/1), x0, y0, x1, y1, ...]`.
        #[allow(clippy::too_many_arguments)]
        fn path_arrange(
            total_parts: usize,
            part_w: f64,
            part_h: f64,
            dist: f64,
            bb_defined: bool,
            bb_min_x: f64,
            bb_min_y: f64,
            bb_max_x: f64,
            bb_max_y: f64,
        ) -> Vec<f64>;
        /// `Slic3r::Geometry::is_rotation_ninety_degrees(double)` (`Geometry.hpp:545-551`).
        fn path_is_rotation_ninety_degrees(a: f64) -> bool;
        /// `Slic3r::Geometry::is_point_inside_polygon_corner(const Point&, const Point&, const
        /// Point&, const Point&)` (`Geometry.cpp:864-908`).
        #[allow(clippy::too_many_arguments)]
        fn path_is_point_inside_polygon_corner(
            ax: i64,
            ay: i64,
            bx: i64,
            by: i64,
            cx: i64,
            cy: i64,
            qx: i64,
            qy: i64,
        ) -> bool;
    }
}

pub use ffi::{
    path_arrange, path_directions_parallel, path_directions_perpendicular, path_is_ccw,
    path_is_point_inside_polygon_corner, path_is_rotation_ninety_degrees, path_linint, path_orient,
    path_rad2deg_dir, path_segments_intersect,
};
