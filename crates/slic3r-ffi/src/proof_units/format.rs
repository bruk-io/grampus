//! Proof path 2 (R19, task `55-foundation-libslic3r-execution`) for the `format.hpp` unit: a
//! direct exercise of the real vendored `Slic3r::format`, which forwards to the pin's own
//! linked `boost::format` (`cxx/proof_units/format.cpp`'s definition of this wrapper -- see its
//! header comment for why this shim exists and what it replaces).

#[cxx::bridge(namespace = "slic3r_ffi::proof_units::format")]
mod ffi {
    // SAFETY: this is cxx's own unsafe-extern declaration block, not a hand-written unsafe
    // expression (see src/clipper2.rs's identical note) -- it carries no body, and the only
    // unsafety is in the bindings `cxx` generates from it.
    unsafe extern "C++" {
        include!("cxx/proof_units/format.h");

        /// `Slic3r::format("%1%", v)` through the real vendored header and the pin's own linked
        /// `boost::format`, returned as its UTF-8 bytes (`rust::Vec<i64>` is the one shape
        /// decision 1 sanctions here -- see `cxx/proof_units/format.cpp` for why not a
        /// `rust::String`).
        fn path2_format_positional_bytes(v: f64) -> Vec<i64>;
    }
}

/// [`ffi::path2_format_positional_bytes`], decoded back to the `String` the oracle's own
/// `boost::format` rendered.
pub fn path2_format_positional(v: f64) -> String {
    let bytes: Vec<u8> = ffi::path2_format_positional_bytes(v)
        .into_iter()
        .map(|b| b as u8)
        .collect();
    String::from_utf8(bytes).expect("boost::format renders valid UTF-8 for a %1% directive")
}
