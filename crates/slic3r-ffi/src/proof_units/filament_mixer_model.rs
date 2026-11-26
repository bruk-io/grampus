//! Proof path 1 (R19) for the `FilamentMixerModel.hpp` unit (task 52-foundation-common-defs,
//! `port/foundation.tsv` row `foundation-01`).
//!
//! `path1_filament_mixer_lerp` calls `Slic3r::filament_mixer_lerp`
//! (`src/libslic3r/FilamentMixer.cpp:58-64`), an externally linked symbol in the archive `just
//! oracle-build` produces that forwards its six channel bytes and mixing ratio straight into
//! `::filament_mixer::lerp` with no transformation of its own. The three output channels cross
//! the bridge as a 3-element `Vec<u8>` (r, g, b, in that order) because a bridge function may
//! not take an out-parameter.

#[cxx::bridge(namespace = "slic3r_ffi::proof_units::filament_mixer_model")]
mod ffi {
    // SAFETY: this is cxx's own unsafe-extern declaration block, not a hand-written unsafe
    // expression (see src/clipper2.rs's identical note) -- it carries no body, and the only
    // unsafety is in the bindings `cxx` generates from it.
    unsafe extern "C++" {
        include!("cxx/proof_units/filament_mixer_model.h");

        /// `Slic3r::filament_mixer_lerp` (`src/libslic3r/FilamentMixer.cpp:58-64`): returns
        /// `[r, g, b]`.
        fn path1_filament_mixer_lerp(
            r1: u8,
            g1: u8,
            b1: u8,
            r2: u8,
            g2: u8,
            b2: u8,
            t: f32,
        ) -> Vec<u8>;
    }
}

pub use ffi::path1_filament_mixer_lerp;
