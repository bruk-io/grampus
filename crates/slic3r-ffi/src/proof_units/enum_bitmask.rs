//! Proof path 2 (R19, task `55-foundation-libslic3r-execution`) for the `enum_bitmask.hpp`
//! unit: a direct exercise of the real vendored `Slic3r::enum_bitmask<T>` class template
//! (`src/libslic3r/enum_bitmask.hpp`), driven with a proof-local three-value enum and
//! reconstructed through the template's own public `has()` method
//! (`cxx/proof_units/enum_bitmask.cpp`'s definition of this wrapper -- see its header comment
//! for why a proof-local enum stands in for a real in-scope one here).

#[cxx::bridge(namespace = "slic3r_ffi::proof_units::enum_bitmask")]
mod ffi {
    // SAFETY: this is cxx's own unsafe-extern declaration block, not a hand-written unsafe
    // expression (see src/clipper2.rs's identical note) -- it carries no body, and the only
    // unsafety is in the bindings `cxx` generates from it.
    unsafe extern "C++" {
        include!("cxx/proof_units/enum_bitmask.h");

        /// Three options OR'd together through the vendored template's own `operator|`
        /// overloads, reconstructed bit-for-bit through `has()`. See
        /// `cxx/proof_units/enum_bitmask.cpp` for the derivation.
        fn path2_combined_mask() -> i64;
    }
}

pub use ffi::path2_combined_mask;
