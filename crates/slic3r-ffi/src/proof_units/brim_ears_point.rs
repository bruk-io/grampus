//! Proof path 2 (R19, task `58-foundation-bounding-box`) for the `BrimEarsPoint.hpp` unit.
//!
//! `BrimPoint::operator==` is entirely inline and never explicitly instantiated anywhere in the
//! vendored tree, so `path2_brim_point_eq` drives it directly, through a real `Slic3r::BrimPoint`
//! instantiated in `cxx/proof_units/brim_ears_point.cpp` -- the same technique
//! `cxx/proof_units/point.cpp`'s `scaled<coord_t>()`/`unscaled<double>()` shims use for
//! header-only template code (`## Decisions recorded on 2026-11-26`, decision 2).

#[cxx::bridge(namespace = "slic3r_ffi::proof_units::brim_ears_point")]
mod ffi {
    // SAFETY: this is cxx's own unsafe-extern declaration block, not a hand-written unsafe
    // expression (see src/clipper2.rs's identical note) -- it carries no body, and the only
    // unsafety is in the bindings `cxx` generates from it.
    unsafe extern "C++" {
        include!("cxx/proof_units/brim_ears_point.h");

        /// `BrimPoint::operator==(const BrimPoint &sp) const` (`BrimEarsPoint.hpp:53-57`):
        /// `pos == sp.pos` (exact, componentwise) and `abs(head_front_radius -
        /// sp.head_front_radius) < float(EPSILON)`.
        fn path2_brim_point_eq(
            ax: f32,
            ay: f32,
            az: f32,
            ar: f32,
            bx: f32,
            by: f32,
            bz: f32,
            br: f32,
        ) -> bool;
    }
}

pub use ffi::path2_brim_point_eq;
