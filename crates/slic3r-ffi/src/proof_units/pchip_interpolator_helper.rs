//! Proof bridge for the `GCode/PchipInterpolatorHelper.cpp`/`.hpp` unit (R19), task
//! `35-trial-run`'s float leg.

#[cxx::bridge(namespace = "slic3r_ffi::proof_units::pchip_interpolator_helper")]
mod ffi {
    // SAFETY: this is cxx's own unsafe-extern declaration block, not a hand-written unsafe
    // expression (see src/clipper2.rs's identical note) -- it carries no body, and the only
    // unsafety is in the bindings `cxx` generates from it.
    unsafe extern "C++" {
        include!("cxx/proof_units/pchip_interpolator_helper.h");

        /// `interpolate(q)` for every `q` in `queries`, over a `PchipInterpolatorHelper`
        /// constructed from `x`/`y`.
        fn path1_interpolate_many(x: Vec<f64>, y: Vec<f64>, queries: Vec<f64>) -> Vec<f64>;
        /// Whether constructing a `PchipInterpolatorHelper` from `x`/`y` throws
        /// `std::invalid_argument`.
        fn path1_constructing_throws(x: Vec<f64>, y: Vec<f64>) -> bool;
    }
}

pub use ffi::{path1_constructing_throws, path1_interpolate_many};
