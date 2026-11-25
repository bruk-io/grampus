//! Proof path 1 (R19) for the `PrintConfigConstants.hpp` unit: the three `#define`s at
//! `src/libslic3r/PrintConfigConstants.hpp:5-7`, each returned at the C++ type its own
//! `PrintConfig.cpp` call site pins it to (`ConfigOptionFloat` wraps `double`, `ConfigOptionInt`
//! wraps `int`, `ConfigOptionBool` wraps `bool`; `Config.hpp:824,1018,1907`), so the value the
//! Rust side compares against is the one the oracle itself would store rather than one widened
//! or narrowed in transit.

#[cxx::bridge(namespace = "slic3r_ffi::proof_units::print_config_constants")]
mod ffi {
    // SAFETY: this is cxx's own unsafe-extern declaration block, not a hand-written unsafe
    // expression (see src/clipper2.rs's identical note) -- it carries no body, and the only
    // unsafety is in the bindings `cxx` generates from it.
    unsafe extern "C++" {
        include!("cxx/proof_units/print_config_constants.h");

        fn path1_initial_layer_height() -> f64;
        fn path1_initial_raft_layers() -> i32;
        fn path1_initial_reduce_crossing_wall() -> bool;
    }
}

pub use ffi::{
    path1_initial_layer_height, path1_initial_raft_layers, path1_initial_reduce_crossing_wall,
};
