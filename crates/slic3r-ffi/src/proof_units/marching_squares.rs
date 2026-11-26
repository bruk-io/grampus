//! Proof path 1 (R19, task `60-foundation-trees-grids`) for the `MarchingSquares.hpp` unit.
//!
//! `path1_execute` registers a minimal `marchsq::_RasterTraits` specialisation over a flat
//! `rust::Vec<double>` and calls the real vendored `marchsq::execute` free function -- see
//! `cxx/proof_units/marching_squares.cpp`.

#[cxx::bridge(namespace = "slic3r_ffi::proof_units::marching_squares")]
mod ffi {
    // SAFETY: this is cxx's own unsafe-extern declaration block, not a hand-written unsafe
    // expression (see src/clipper2.rs's identical note) -- it carries no body, and the only
    // unsafety is in the bindings `cxx` generates from it.
    unsafe extern "C++" {
        include!("cxx/proof_units/marching_squares.h");

        /// `marchsq::execute(raster, isoval, windowsize)` over a `rows * cols` flat raster,
        /// flattened to `[ring_len, r0, c0, r1, c1, ...]*` across every returned ring.
        fn path1_execute(
            raster: &Vec<f64>,
            rows: usize,
            cols: usize,
            isoval: f64,
            window_r: i64,
            window_c: i64,
        ) -> Vec<i64>;
    }
}

pub use ffi::path1_execute;
