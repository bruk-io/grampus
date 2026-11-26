//! Proof path 2 (R19, task `56-foundation-point`) for the `MTUtils.hpp` unit: a direct call
//! through the real vendored `Slic3r::grid<double>` function (`cxx/proof_units/mt_utils.cpp`).

#[cxx::bridge(namespace = "slic3r_ffi::proof_units::mt_utils")]
mod ffi {
    // SAFETY: this is cxx's own unsafe-extern declaration block, not a hand-written unsafe
    // expression (see src/clipper2.rs's identical note) -- it carries no body, and the only
    // unsafety is in the bindings `cxx` generates from it.
    unsafe extern "C++" {
        include!("cxx/proof_units/mt_utils.h");

        /// The real vendored `Slic3r::grid<double>(start, stop, stride)`. See
        /// `cxx/proof_units/mt_utils.cpp` for the call.
        fn path2_grid(start: f64, stop: f64, stride: f64) -> Vec<f64>;
    }
}

pub use ffi::path2_grid;
