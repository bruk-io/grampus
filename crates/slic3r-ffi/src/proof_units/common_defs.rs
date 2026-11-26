//! Proof path 2 (R19) for the `CommonDefs.hpp` unit (task 52-foundation-common-defs,
//! `port/foundation.tsv` row `foundation-01`).
//!
//! `path2_nozzle_type_discriminants` returns `Slic3r::NozzleType`'s seven enumerators, in
//! declaration order, cast to `int64_t` by the vendored header's own compiled definition --
//! the sanctioned answer (`docs/PORTING.md`, "Decisions recorded on 2026-11-25", decision 4) to
//! a header row the committed stage-dump corpus carries no field for.

#[cxx::bridge(namespace = "slic3r_ffi::proof_units::common_defs")]
mod ffi {
    // SAFETY: this is cxx's own unsafe-extern declaration block, not a hand-written unsafe
    // expression (see src/clipper2.rs's identical note) -- it carries no body, and the only
    // unsafety is in the bindings `cxx` generates from it.
    unsafe extern "C++" {
        include!("cxx/proof_units/common_defs.h");

        /// `Slic3r::NozzleType`'s seven enumerators (`CommonDefs.hpp:11-20`), in declaration
        /// order.
        fn path2_nozzle_type_discriminants() -> Vec<i64>;
    }
}

pub use ffi::path2_nozzle_type_discriminants;
