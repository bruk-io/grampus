//! Proof path 2 (R19, task `56-foundation-point`) for the `Execution/ExecutionTBB.hpp` unit: a
//! direct call through the real vendored `Slic3r::ex_tbb` policy, dispatched through
//! `Slic3r::execution::reduce` (`cxx/proof_units/execution_tbb.cpp`), which really runs
//! `tbb::parallel_reduce` on this side.

#[cxx::bridge(namespace = "slic3r_ffi::proof_units::execution_tbb")]
mod ffi {
    // SAFETY: this is cxx's own unsafe-extern declaration block, not a hand-written unsafe
    // expression (see src/clipper2.rs's identical note) -- it carries no body, and the only
    // unsafety is in the bindings `cxx` generates from it.
    unsafe extern "C++" {
        include!("cxx/proof_units/execution_tbb.h");

        /// `values` summed through the real vendored `Slic3r::ex_tbb` policy's own
        /// `Traits<ExecutionTBB>::reduce`, with `mergefn = +` and `accessfn = +1`, granularity
        /// `2`. See `cxx/proof_units/execution_tbb.cpp` for the call.
        fn path2_reduce_increments_and_sums(values: &Vec<i64>) -> i64;
    }
}

pub use ffi::path2_reduce_increments_and_sums;
