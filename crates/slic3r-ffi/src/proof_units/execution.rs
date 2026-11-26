//! Proof path 2 (R19, task `55-foundation-libslic3r-execution`) for the
//! `Execution/Execution.hpp` unit: a direct call through the real vendored
//! `Slic3r::execution::reduce` dispatch function, specialised for a minimal, real, serial
//! proof-local policy (`cxx/proof_units/execution.cpp`'s definition of this wrapper -- see its
//! header comment for why a proof-local policy stands in for a real concrete one here).

#[cxx::bridge(namespace = "slic3r_ffi::proof_units::execution")]
mod ffi {
    // SAFETY: this is cxx's own unsafe-extern declaration block, not a hand-written unsafe
    // expression (see src/clipper2.rs's identical note) -- it carries no body, and the only
    // unsafety is in the bindings `cxx` generates from it.
    unsafe extern "C++" {
        include!("cxx/proof_units/execution.h");

        /// `values` summed through the vendored `Slic3r::execution::reduce`'s own forwarding,
        /// with `mergefn = +` and `accessfn = *2`. See `cxx/proof_units/execution.cpp` for the
        /// policy this dispatches through.
        fn path2_reduce_doubles_and_sums(values: &Vec<i64>) -> i64;
    }
}

pub use ffi::path2_reduce_doubles_and_sums;
