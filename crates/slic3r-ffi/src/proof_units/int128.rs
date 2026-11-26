//! Proof path 1 (R19) for the `Int128.hpp` unit.
//!
//! `Int128.hpp` is header-only with no externally linked symbol of its own, so
//! `cxx/proof_units/int128.cpp` `#include`s the vendored header directly and calls its methods
//! fresh rather than reaching a symbol in `liblibslic3r.a` -- the same shape
//! `cxx/proof_units/object_id.cpp`'s `path2_header_observations` uses for a header-only unit.
//! Everything crosses the bridge as a plain integer or float: `coord_t` is `int64_t` at the pin
//! (`src/libslic3r/libslic3r.h:45-48`).

#[cxx::bridge(namespace = "slic3r_ffi::proof_units::int128")]
mod ffi {
    // SAFETY: this is cxx's own unsafe-extern declaration block, not a hand-written unsafe
    // expression (see src/clipper2.rs's identical note) -- it carries no body, and the only
    // unsafety is in the bindings `cxx` generates from it.
    unsafe extern "C++" {
        include!("cxx/proof_units/int128.h");

        fn multiply_hi(lhs: i64, rhs: i64) -> i64;
        fn multiply_lo(lhs: i64, rhs: i64) -> u64;
        fn multiply_as_double(lhs: i64, rhs: i64) -> f64;
        fn multiply_sign(lhs: i64, rhs: i64) -> i32;
        fn sign_determinant_2x2(a11: i64, a12: i64, a21: i64, a22: i64) -> i32;
        fn compare_rationals(p1: i64, q1: i64, p2: i64, q2: i64) -> i32;
        fn sign_determinant_2x2_filtered(a11: i64, a12: i64, a21: i64, a22: i64) -> i32;
        fn compare_rationals_filtered(p1: i64, q1: i64, p2: i64, q2: i64) -> i32;
        fn add_hi(lhs: i64, rhs: i64) -> i64;
        fn add_lo(lhs: i64, rhs: i64) -> u64;
        fn sub_hi(lhs: i64, rhs: i64) -> i64;
        fn sub_lo(lhs: i64, rhs: i64) -> u64;
        fn neg_hi(lhs: i64) -> i64;
        fn neg_lo(lhs: i64) -> u64;
        fn add_assign_hi(lhs: i64, rhs: i64) -> i64;
        fn add_assign_lo(lhs: i64, rhs: i64) -> u64;
        fn sub_assign_hi(lhs: i64, rhs: i64) -> i64;
        fn sub_assign_lo(lhs: i64, rhs: i64) -> u64;
    }
}

pub use ffi::{
    add_assign_hi, add_assign_lo, add_hi, add_lo, compare_rationals, compare_rationals_filtered,
    multiply_as_double, multiply_hi, multiply_lo, multiply_sign, neg_hi, neg_lo,
    sign_determinant_2x2, sign_determinant_2x2_filtered, sub_assign_hi, sub_assign_lo, sub_hi,
    sub_lo,
};
