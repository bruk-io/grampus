//! Proof path 2 (R19, task `56-foundation-point`) for the `Execution/ExecutionSeq.hpp` unit: a
//! direct call through the real vendored `Slic3r::ex_seq` policy, dispatched through
//! `Slic3r::execution::reduce` (`cxx/proof_units/execution_seq.cpp`).

#[cxx::bridge(namespace = "slic3r_ffi::proof_units::execution_seq")]
mod ffi {
    // SAFETY: this is cxx's own unsafe-extern declaration block, not a hand-written unsafe
    // expression (see src/clipper2.rs's identical note) -- it carries no body, and the only
    // unsafety is in the bindings `cxx` generates from it.
    unsafe extern "C++" {
        include!("cxx/proof_units/execution_seq.h");

        /// `values` summed through the real vendored `Slic3r::ex_seq` policy's own
        /// `Traits<ExecutionSeq>::reduce`, with `mergefn = +` and `accessfn = *3`. See
        /// `cxx/proof_units/execution_seq.cpp` for the call.
        fn path2_reduce_triples_and_sums(values: &Vec<i64>) -> i64;
    }
}

pub use ffi::path2_reduce_triples_and_sums;
