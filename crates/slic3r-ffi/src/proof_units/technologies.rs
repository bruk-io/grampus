//! Proof path 1 (R19) for the `Technologies.hpp` unit: the header's eighteen `0`/`1` `#define`s,
//! in its own declaration order. The returned vector's length is the oracle's own count, so a
//! flag added to or removed from the header on a future pin makes the proof fail loudly instead
//! of silently comparing a short prefix.

#[cxx::bridge(namespace = "slic3r_ffi::proof_units::technologies")]
mod ffi {
    // SAFETY: this is cxx's own unsafe-extern declaration block, not a hand-written unsafe
    // expression (see src/clipper2.rs's identical note) -- it carries no body, and the only
    // unsafety is in the bindings `cxx` generates from it.
    unsafe extern "C++" {
        include!("cxx/proof_units/technologies.h");

        fn path1_flags() -> Vec<i32>;
    }
}

pub use ffi::path1_flags;
