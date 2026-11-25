//! Proof path 3 (R19, plan step 18) for the `ClipperZUtils.cpp` unit: a test translation unit
//! that `#include`s the vendored `.cpp` whole, so a `static` helper the pin never exports can be
//! called. `cxx/proof_units/clipper_z_utils.cpp` is that translation unit, and its header
//! comment carries the duplicate-symbol constraint that choice leaves standing.
//!
//! The shim takes and returns plain integers: `coord_t` is `int64_t` at the pin
//! (`src/libslic3r/libslic3r.h:45-48`), so a coordinate is carried as a 64-bit integer and never
//! through an `f64`.

#[cxx::bridge(namespace = "slic3r_ffi::proof_units::clipper_z_utils")]
mod ffi {
    /// One `Slic3r::ClipperZUtils::ZPoint` (`Vec3crd`: three `coord_t`/`int64_t` fields), carried
    /// as plain integers across the bridge -- proof path 3's shape.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    struct ProofZPoint {
        x: i64,
        y: i64,
        z: i64,
    }

    // SAFETY: this is cxx's own unsafe-extern declaration block, not a hand-written unsafe
    // expression (see src/clipper2.rs's identical note) -- it carries no body, and the only
    // unsafety is in the bindings `cxx` generates from it.
    unsafe extern "C++" {
        include!("cxx/proof_units/clipper_z_utils.h");

        /// Proof path 3 (R19): `Slic3r::ClipperZUtils::to_paths64`, `static` (internal linkage)
        /// in the vendored `ClipperZUtils.cpp`, reached only because
        /// `cxx/proof_units/clipper_z_utils.cpp` `#include`s that file and defines this
        /// forwarding wrapper in the same translation unit (see that file's header comment for
        /// the inclusion rule).
        fn path3_to_paths64(path: &Vec<ProofZPoint>) -> Vec<ProofZPoint>;
    }
}

pub use ffi::{path3_to_paths64, ProofZPoint};
