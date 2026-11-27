//! Proof paths 1 and 2 (task `76-foundation-voronoi-curves`) for the
//! `Geometry/Circle.cpp`/`Circle.hpp` unit. `path1_*` calls `Circle.cpp`'s own externally
//! linked functions; `path2_*` drives `Circle.hpp`'s own header-only templates, instantiated
//! directly in `cxx/proof_units/circle.cpp` (`docs/PORTING.md`, "Decisions recorded on
//! 2026-11-26", decision 2).

#[cxx::bridge(namespace = "slic3r_ffi::proof_units::circle")]
mod ffi {
    // SAFETY: this is cxx's own unsafe-extern declaration block, not a hand-written unsafe
    // expression (see src/clipper2.rs's identical note) -- it carries no body of its own, and
    // the only unsafety is in the bindings `cxx` generates from it.
    unsafe extern "C++" {
        include!("cxx/proof_units/circle.h");

        /// `circle_center_taubin_newton(const Vec2ds::const_iterator&, ..., size_t)`
        /// (`Circle.cpp:31-104`). `flat` is `[x0, y0, x1, y1, ...]`.
        fn path1_circle_center_taubin_newton(flat: Vec<f64>, cycles: u64) -> Vec<f64>;
        /// `circle_taubin_newton(const Vec2ds&, size_t)` (`Circle.cpp:106-117`); returns
        /// `[center.x, center.y, radius]`.
        fn path1_circle_taubin_newton(flat: Vec<f64>, cycles: u64) -> Vec<f64>;

        /// `circle_center<Vec2d>(...)` (`Circle.hpp:17-36`).
        fn path2_circle_center(
            ax: f64,
            ay: f64,
            bx: f64,
            by: f64,
            cx: f64,
            cy: f64,
            epsilon: f64,
        ) -> Vec<f64>;
        /// `smallest_enclosing_circle_welzl(const Points&)` (`Circle.hpp:151-154`); `flat` is
        /// `[x0, y0, x1, y1, ...]` (scaled, integer); returns `[center.x, center.y, radius]`.
        fn path2_smallest_enclosing_circle_welzl(flat: Vec<i64>) -> Vec<f64>;
        /// `ray_circle_intersections_r2_lv2_c<double>(...)` (`Circle.hpp:161-175`); returns
        /// `[count, first.x, first.y, second.x, second.y]` -- the last four meaningful only
        /// while `count > 0`.
        fn path2_ray_circle_intersections(r2: f64, a: f64, b: f64, lv2: f64, c: f64) -> Vec<f64>;
    }
}

pub use ffi::{
    path1_circle_center_taubin_newton, path1_circle_taubin_newton, path2_circle_center,
    path2_ray_circle_intersections, path2_smallest_enclosing_circle_welzl,
};
