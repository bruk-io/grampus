//! The oracle for the `clonable_ptr.hpp` unit (R19): the vendored template's own instantiated
//! code, driven in a fixed order, one `i64` per observation.
//!
//! Every observation crosses the bridge as a plain integer, and every one is a value, a count or
//! a relation -- never an address, which would be meaningless across the two languages. See
//! `cxx/proof_units/clonable_ptr.cpp` for the sequence and for why the element type is proof
//! input rather than the pin's own `clonable_ptr<const ConfigOption>`.

#[cxx::bridge(namespace = "slic3r_ffi::proof_units::clonable_ptr")]
mod ffi {
    // SAFETY: this is cxx's own unsafe-extern declaration block, not a hand-written unsafe
    // expression (see src/clipper2.rs's identical note) -- it carries no body, and the only
    // unsafety is in the bindings `cxx` generates from it.
    unsafe extern "C++" {
        include!("cxx/proof_units/clonable_ptr.h");

        /// The `clonable_ptr.hpp` row's oracle: 27 observations over the vendored template's
        /// constructors, `swap`, `reset`, `release`, `operator bool`, `operator*`/`operator->`
        /// and the six comparison operators, in the order
        /// `cxx/proof_units/clonable_ptr.cpp` scripts them.
        fn path2_observations() -> Vec<i64>;
    }
}

pub use ffi::path2_observations;
