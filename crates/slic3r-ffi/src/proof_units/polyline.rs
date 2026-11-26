//! Proof paths 1 and 2 (R19, task `63-foundation-polyline`) for the `Polyline.cpp`/`Polyline.hpp`
//! unit.
//!
//! `path1_clip_end` drives `Slic3r::Polyline::clip_end(double)` (`Polyline.cpp:61-101`);
//! `path2_total_length`/`path2_is_closed`/`path2_new_scale` drive `Polyline.hpp`'s own
//! header-only surface directly through a real vendored `Polyline`. See
//! `cxx/proof_units/polyline.cpp` for both.

#[cxx::bridge(namespace = "slic3r_ffi::proof_units::polyline")]
mod ffi {
    // SAFETY: this is cxx's own unsafe-extern declaration block, not a hand-written unsafe
    // expression (see src/clipper2.rs's identical note) -- it carries no body, and the only
    // unsafety is in the bindings `cxx` generates from it.
    unsafe extern "C++" {
        include!("cxx/proof_units/polyline.h");

        /// `Slic3r::Polyline(xs, ys).clip_end(distance)` (`Polyline.cpp:61-101`), returning the
        /// clipped points flattened as `[x0, y0, x1, y1, ...]`.
        fn path1_clip_end(xs: &[i64], ys: &[i64], distance: f64) -> Vec<i64>;

        /// `Slic3r::total_length({Slic3r::Polyline(xs, ys)})` (`Polyline.hpp:168-173`).
        fn path2_total_length(xs: &[i64], ys: &[i64]) -> f64;

        /// `Slic3r::Polyline(xs, ys).is_closed()` (`Polyline.hpp:132`).
        fn path2_is_closed(xs: &[i64], ys: &[i64]) -> bool;

        /// `Slic3r::Polyline::new_scale(points)` (`Polyline.hpp:55-63`), returning the scaled
        /// points flattened as `[x0, y0, x1, y1, ...]`.
        fn path2_new_scale(xs: &[f64], ys: &[f64]) -> Vec<i64>;
    }
}

pub use ffi::{path1_clip_end, path2_is_closed, path2_new_scale, path2_total_length};
