//! Proof shim for the `Utils.hpp`/`utils.cpp` unit (task 57-foundation-utils, R19). Shaped like
//! proof path 1 (calls an externally-linked symbol directly, no `#include` of a vendored
//! `.cpp`), not proof path 3 as the manifest row's `proof path` cell names -- see
//! `cxx/proof_units/utils.cpp`'s header comment for the measurement that rules path 3 out, and
//! this task's pull request body for the same note.

#[cxx::bridge(namespace = "slic3r_ffi::proof_units::utils")]
mod ffi {
    // SAFETY: this is cxx's own unsafe-extern declaration block, not a hand-written unsafe
    // expression (see src/clipper2.rs's identical note) -- it carries no body, and the only
    // unsafety is in the bindings `cxx` generates from it.
    unsafe extern "C++" {
        include!("cxx/proof_units/utils.h");

        /// `Slic3r::get_utf8_sequence_length(const char*, size_t)` (`utils.cpp:1390-1452`):
        /// bytes in the UTF-8 sequence starting at `seq[0]`. `seq` must be non-empty.
        fn path1_get_utf8_sequence_length(seq: Vec<u8>) -> u64;
    }
}

pub use ffi::path1_get_utf8_sequence_length;
