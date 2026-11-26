//! Proof path 1 (R19, task `55-foundation-libslic3r-execution`) for the
//! `libslic3r.cpp`/`libslic3r.h` unit: a direct read of `::SCALING_FACTOR`, the externally
//! linked global `src/libslic3r/libslic3r.cpp:3` defines (`cxx/proof_units/libslic3r.cpp`'s
//! definition of this wrapper). Also carries [`path2_unscale_double`] for the `libslic3r.h`
//! row's own `// PROOF: 2` test, a direct call through the vendored `unscale<T>` template --
//! see `cxx/proof_units/libslic3r.cpp` for why this was added on a review pass.

#[cxx::bridge(namespace = "slic3r_ffi::proof_units::libslic3r")]
mod ffi {
    // SAFETY: this is cxx's own unsafe-extern declaration block, not a hand-written unsafe
    // expression (see src/clipper2.rs's identical note) -- it carries no body, and the only
    // unsafety is in the bindings `cxx` generates from it.
    unsafe extern "C++" {
        include!("cxx/proof_units/libslic3r.h");

        /// `src/libslic3r/libslic3r.cpp:3`'s `SCALING_FACTOR`, an externally linked `double` in
        /// `liblibslic3r.a` (`just oracle-build`'s output).
        fn path1_scaling_factor() -> f64;

        /// `Slic3r::unscale<double>(scaled)` (`libslic3r.h:137-138`) through the real vendored
        /// template, for the same scaled coordinate the port's own `unscale::<f64>` is compared
        /// against.
        fn path2_unscale_double(scaled: i64) -> f64;
    }
}

pub use ffi::{path1_scaling_factor, path2_unscale_double};
