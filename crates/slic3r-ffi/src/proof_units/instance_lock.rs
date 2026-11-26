//! Proof bridge for the `InstanceLock.cpp`/`InstanceLock.hpp` unit (R19), task
//! `35-trial-run`'s throw/non-const-ref-out-param leg.

#[cxx::bridge(namespace = "slic3r_ffi::proof_units::instance_lock")]
mod ffi {
    // SAFETY: this is cxx's own unsafe-extern declaration block, not a hand-written unsafe
    // expression (see src/clipper2.rs's identical note) -- it carries no body, and the only
    // unsafety is in the bindings `cxx` generates from it.
    unsafe extern "C++" {
        include!("cxx/proof_units/instance_lock.h");

        /// See `cxx/proof_units/instance_lock.cpp`'s own doc comment for the five observations
        /// this scripts, in order.
        fn path1_observations(path: &str) -> Vec<i64>;
    }
}

pub use ffi::path1_observations;
