//! Proof path 1 (task `76-foundation-voronoi-curves`) for the `Geometry/Curves.hpp` manifest
//! row: `PiecewiseFittedCurve<2, float, CubicBSplineKernel<float>>::get_fitted_value`,
//! constructed directly with literal fields in `cxx/proof_units/curves.cpp` (bypassing
//! `fit_curve`/`fit_cubic_bspline` on both sides -- see
//! `crates/slic3r-geometry-dir/src/curves.rs`'s own module doc for why those two are not
//! ported).

#[cxx::bridge(namespace = "slic3r_ffi::proof_units::curves")]
mod ffi {
    // SAFETY: this is cxx's own unsafe-extern declaration block, not a hand-written unsafe
    // expression (see src/clipper2.rs's identical note) -- it carries no body of its own, and
    // the only unsafety is in the bindings `cxx` generates from it.
    unsafe extern "C++" {
        include!("cxx/proof_units/curves.h");

        /// `PiecewiseFittedCurve<2, float, CubicBSplineKernel<float>>::get_fitted_value`
        /// (`Curves.hpp:76-93`). `coefficients_flat` is `[x0, y0, x1, y1, ...]`, `cols` columns.
        fn path1_get_fitted_value(
            coefficients_flat: Vec<f64>,
            cols: u64,
            start: f64,
            segment_size: f64,
            endpoints_level_of_freedom: u64,
            observation_point: f64,
        ) -> Vec<f64>;
    }
}

pub use ffi::path1_get_fitted_value;
