//! Proof paths 1 and 2 (R19) for the `ObjectID.cpp`/`ObjectID.hpp` unit.
//!
//! The two `path1_*` functions are direct calls to externally linked symbols in the archive
//! `just oracle-build` produces; `path2_header_observations` drives the vendored header's own
//! inline methods and returns one `i64` per observation. Everything crosses the bridge as a
//! plain integer: `coord_t` is `int64_t` at the pin (`src/libslic3r/libslic3r.h:45-48`), so a
//! coordinate is carried as a 64-bit integer and never through an `f64`.
//!
//! **Both `path1_*` functions read a process-global counter that starts at zero and is never
//! reset** (`src/libslic3r/ObjectID.cpp:6`, `size_t ObjectBase::s_last_id = 0`), so a test
//! calling them must be the only test in its binary. Its callers say so at the top of their own
//! files.

#[cxx::bridge(namespace = "slic3r_ffi::proof_units::object_id")]
mod ffi {
    // SAFETY: this is cxx's own unsafe-extern declaration block, not a hand-written unsafe
    // expression (see src/clipper2.rs's identical note) -- it carries no body, and the only
    // unsafety is in the bindings `cxx` generates from it.
    unsafe extern "C++" {
        include!("cxx/proof_units/object_id.h");

        /// `Slic3r::wipe_tower_object_id()` (`src/libslic3r/ObjectID.cpp:9-14`): the first
        /// `Slic3r::ObjectBase` the calling process constructs, so id `1`.
        fn path1_wipe_tower_object_id() -> u64;

        /// `Slic3r::wipe_tower_instance_id()` (`src/libslic3r/ObjectID.cpp:16-21`): the second
        /// one, so id `2` when it is called after `path1_wipe_tower_object_id` in the same
        /// process.
        fn path1_wipe_tower_instance_id() -> u64;

        /// The `ObjectID.hpp` row's oracle: the vendored header's own inline code, driven in a
        /// fixed order, one `i64` per observation. See
        /// `cxx/proof_units/object_id.cpp` for the sequence and for why every observation is a
        /// delta or a relation rather than an absolute id.
        fn path2_header_observations() -> Vec<i64>;
    }
}

pub use ffi::{
    path1_wipe_tower_instance_id, path1_wipe_tower_object_id, path2_header_observations,
};
