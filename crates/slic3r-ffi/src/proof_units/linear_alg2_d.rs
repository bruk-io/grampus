//! Proof path 1 (R19, task `72-foundation-arachne-grids`) for the `linearAlg2D.hpp` unit.
//!
//! `path1_point_is_left_of_line` calls `Slic3r::Arachne::LinearAlg2D::pointIsLeftOfLine`
//! directly; `path1_get_angle_left` calls `Slic3r::Arachne::LinearAlg2D::getAngleLeft` directly
//! -- both `static inline` header-only functions, reached by including the header the same way
//! `cxx/proof_units/point.cpp`'s header-only shims do. See `cxx/proof_units/linear_alg2_d.cpp`.

#[cxx::bridge(namespace = "slic3r_ffi::proof_units::linear_alg2_d")]
mod ffi {
    // SAFETY: this is cxx's own unsafe-extern declaration block, not a hand-written unsafe
    // expression (see src/clipper2.rs's identical note) -- it carries no body, and the only
    // unsafety is in the bindings `cxx` generates from it.
    unsafe extern "C++" {
        include!("cxx/proof_units/linear_alg2_d.h");

        /// `Slic3r::Arachne::LinearAlg2D::pointIsLeftOfLine(p, a, b)`
        /// (`Arachne/utils/linearAlg2D.hpp:26-29`).
        fn path1_point_is_left_of_line(px: i64, py: i64, ax: i64, ay: i64, bx: i64, by: i64)
            -> i64;

        /// `Slic3r::Arachne::LinearAlg2D::getAngleLeft(a, b, c)`
        /// (`Arachne/utils/linearAlg2D.hpp:48-65`).
        fn path1_get_angle_left(ax: i64, ay: i64, bx: i64, by: i64, cx: i64, cy: i64) -> f32;
    }
}

pub use ffi::{path1_get_angle_left, path1_point_is_left_of_line};
