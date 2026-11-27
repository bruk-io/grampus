//! Proof path 1 (task `76-foundation-voronoi-curves`) for the `Geometry/Bicubic.hpp` manifest
//! row: `CubicKernelWrapper<BicubicInternal::CubicBSplineKernel<float>>::kernel(float)`,
//! instantiated directly in `cxx/proof_units/bicubic.cpp` (`docs/PORTING.md`, "Decisions
//! recorded on 2026-11-26", decision 2 -- a `proof path: 1` row's oracle route is the per-unit
//! bridge regardless of whether the symbol is externally linked or a header-only template).

#[cxx::bridge(namespace = "slic3r_ffi::proof_units::bicubic")]
mod ffi {
    // SAFETY: this is cxx's own unsafe-extern declaration block, not a hand-written unsafe
    // expression (see src/clipper2.rs's identical note) -- it carries no body of its own, and
    // the only unsafety is in the bindings `cxx` generates from it.
    unsafe extern "C++" {
        include!("cxx/proof_units/bicubic.h");

        /// `CubicKernelWrapper<BicubicInternal::CubicBSplineKernel<float>>::kernel(float)`
        /// (`Bicubic.hpp:200-215`), taking/returning `f64` as this bridge's wire width (an exact
        /// widening of the `float` the shim casts back down to; see the proof test's own doc).
        fn path1_cubic_bspline_kernel(x: f64) -> f64;
    }
}

pub use ffi::path1_cubic_bspline_kernel;
