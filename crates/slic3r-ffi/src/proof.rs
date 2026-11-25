//! The `cxx` bridge for the per-unit proof harness's own two proofs (task 31-proof-harness, plan
//! step 18, R19): one call per proof path this task wires, each compared through
//! `slic3r-proof`'s `tiers::compare_unit` at `crates/slic3r-proof/tests/harness_paths.rs`.
//! Neither shim reaches into libslic3r's own types from Rust -- both take and return plain
//! integers, per R19's rule for a proof shim (`coord_t` is `int64_t` at the pin,
//! `src/libslic3r/libslic3r.h:45-48`, so a coordinate is carried as a 64-bit integer and never
//! through an `f64`).

#[cxx::bridge(namespace = "slic3r_ffi::proof")]
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
        include!("cxx/proof_shims.h");

        /// Proof path 1 (R19): a direct call to an externally linked symbol in the archive
        /// `just oracle-build` produces -- `Slic3r::wipe_tower_object_id()`
        /// (`src/libslic3r/ObjectID.cpp`, `cxx/proof_shims.cpp`'s definition of this wrapper).
        fn path1_wipe_tower_object_id() -> u64;

        /// Proof path 3 (R19): `Slic3r::ClipperZUtils::to_paths64`, `static` (internal linkage)
        /// in the vendored `ClipperZUtils.cpp`, reached only because
        /// `cxx/include_vendored.cpp` `#include`s that file and defines this forwarding wrapper
        /// in the same translation unit (see that file's header comment for the inclusion rule).
        fn path3_to_paths64(path: &Vec<ProofZPoint>) -> Vec<ProofZPoint>;

        /// Proof path 1 (R19), task 54-foundation-objectid-clonable's
        /// `PrintConfigConstants.hpp` row: the three `#define`s at
        /// `src/libslic3r/PrintConfigConstants.hpp:5-7`, each returned at the C++ type its own
        /// `PrintConfig.cpp` call site pins it to (`ConfigOptionFloat` wraps `double`,
        /// `ConfigOptionInt` wraps `int`, `ConfigOptionBool` wraps `bool`;
        /// `Config.hpp:824,1018,1907`). Appended at the end of this block, changing no line
        /// above, so a sibling foundation task's append conflicts textually. See
        /// `cxx/proof_shims.cpp`'s own append for why that task's crate proves the same header
        /// through its own `build.rs` instead of calling these.
        fn path1_initial_layer_height() -> f64;
        fn path1_initial_raft_layers() -> i32;
        fn path1_initial_reduce_crossing_wall() -> bool;

        /// Proof path 1 (R19), task 54-foundation-objectid-clonable's `Technologies.hpp` row:
        /// the header's eighteen `0`/`1` `#define`s, in its own declaration order.
        fn path1_technologies_flags() -> Vec<i32>;
    }
}

pub use ffi::{
    path1_initial_layer_height, path1_initial_raft_layers, path1_initial_reduce_crossing_wall,
    path1_technologies_flags, path1_wipe_tower_object_id, path3_to_paths64, ProofZPoint,
};
