//! Proof path 2 (R19) for the `LifecycleEvents.hpp` unit -- see `cxx/proof_units/
//! lifecycle_events.cpp`'s header comment for why this row's `// PROOF: 2` marker (matching its
//! manifest cell) is proven through this bridge rather than through a stage dump.
//!
//! The enum value crosses the bridge as a plain `i32` ordinal, never a libslic3r type (decision
//! 1, `docs/PORTING.md`).

#[cxx::bridge(namespace = "slic3r_ffi::proof_units::lifecycle_events")]
mod ffi {
    // SAFETY: this is cxx's own unsafe-extern declaration block, not a hand-written unsafe
    // expression (see src/clipper2.rs's identical note) -- it carries no body, and the only
    // unsafety is in the bindings `cxx` generates from it.
    unsafe extern "C++" {
        include!("cxx/proof_units/lifecycle_events.h");

        /// `Slic3r::lifecycle_event_to_string`, called with `event` as the vendored
        /// `LifecycleEvent` enum's ordinal (declaration order, no explicit discriminants).
        /// Renamed on the Rust side (`rust_name`) so a port crate that also imports its own
        /// same-named port function can `use` both without a collision.
        #[rust_name = "oracle_lifecycle_event_to_string"]
        fn lifecycle_event_to_string(event: i32) -> String;
        /// `Slic3r::lifecycle_evt_code_to_string`, called with `code` as the vendored
        /// `LifecycleEvtCode` enum's ordinal. Renamed on the Rust side for the same reason.
        #[rust_name = "oracle_lifecycle_evt_code_to_string"]
        fn lifecycle_evt_code_to_string(code: i32) -> String;
    }
}

pub use ffi::{oracle_lifecycle_event_to_string, oracle_lifecycle_evt_code_to_string};
