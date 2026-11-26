//! Proof bridge for the `MaterialType.cpp`/`MaterialType.hpp` unit (R19), task
//! `35-trial-run`'s fixed-point (near-miss) leg. See
//! `cxx/proof_units/material_type.cpp`'s own header comment for why this calls the externally
//! linked `Slic3r::MaterialType` class statics directly despite the row's seeded proof-path-3
//! cell.

#[cxx::bridge(namespace = "slic3r_ffi::proof_units::material_type")]
mod ffi {
    // SAFETY: this is cxx's own unsafe-extern declaration block, not a hand-written unsafe
    // expression (see src/clipper2.rs's identical note) -- it carries no body, and the only
    // unsafety is in the bindings `cxx` generates from it.
    unsafe extern "C++" {
        include!("cxx/proof_units/material_type.h");

        /// `[min_temp, max_temp, found (0/1)]` from
        /// `Slic3r::MaterialType::get_temperature_range`.
        fn path_temperature_range(name: &str) -> Vec<i64>;
        /// `[chamber_min_temp, chamber_max_temp, found (0/1)]`.
        fn path_chamber_temperature_range(name: &str) -> Vec<i64>;
        /// `[adhesion_coefficient, found (0.0/1.0)]`.
        fn path_adhesion_coefficient(name: &str) -> Vec<f64>;
        /// `[yield_strength, found (0.0/1.0)]`.
        fn path_yield_strength(name: &str) -> Vec<f64>;
        /// `[thermal_length, found (0.0/1.0)]`.
        fn path_thermal_length(name: &str) -> Vec<f64>;
    }
}

pub use ffi::{
    path_adhesion_coefficient, path_chamber_temperature_range, path_temperature_range,
    path_thermal_length, path_yield_strength,
};
