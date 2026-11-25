//! Proof path 1 (R19) for the `ObjectID.cpp` manifest row of task
//! `54-foundation-objectid-clonable`: the two externally linked `Slic3r::wipe_tower_*_id()`
//! functions and the `s_last_id` definition that are the whole of that translation unit's own
//! code, compiled from the pin by this crate's `build.rs` and called here through its shim
//! (`shim/object_id_shim.cpp`).
//!
//! CONSTRAINT ON THIS FILE -- it holds exactly one `#[test]`, deliberately. Both sides of this
//! comparison read a *process-global* counter that starts at zero and is never reset:
//! `ObjectID.cpp:6`'s `size_t ObjectBase::s_last_id = 0` on the C++ side, and
//! `object_id.rs`'s `NEXT_OBJECT_ID` on the Rust side. The expected values (`1`, then `2`) hold
//! only because nothing else in this test binary constructs an `ObjectBase` -- on either side --
//! before the single test below runs, and Rust runs a binary's tests in parallel threads.
//! Do not add a second test to this file. A test that needs an `ObjectBase`, or that drives the
//! header's inline methods, belongs in `src/object_id.rs`'s own `#[cfg(test)]` module, which is a
//! separate binary with its own fresh pair of counters -- the same constraint, for the same
//! reason, that `crates/slic3r-proof/tests/harness_paths.rs` carries at the top of its own file.

use slic3r_objectid::object_id::{wipe_tower_instance_id, wipe_tower_object_id};
use slic3r_proof::{compare_unit, Oracle};

// The proof-path-1 oracle: `shim/object_id_shim.cpp`'s two wrappers over
// `Slic3r::wipe_tower_object_id()`/`wipe_tower_instance_id()`, linked against the pin's own
// `ObjectID.cpp` compiled unmodified by this crate's `build.rs`.
extern "C" {
    fn slic3r_objectid_shim_wipe_tower_object_id() -> u64;
    fn slic3r_objectid_shim_wipe_tower_instance_id() -> u64;
}

/// `ObjectID.cpp:9-19` against `crates/slic3r-objectid/src/object_id.rs`'s
/// `wipe_tower_object_id`/`wipe_tower_instance_id`: two function-local `static ObjectBase`s off
/// one process-global counter, so the first call yields id `1` and the second id `2`. Both sides
/// are driven in the same order, and each side's counter is its own, so this compares the port's
/// output against the oracle's own output rather than against a transcribed constant.
// PROOF: 1
#[test]
fn object_id_wipe_tower_ids_match_the_oracle() {
    // Ordered deliberately: object id first on each side, then instance id, because each call
    // advances that side's own counter.
    let oracle_object_id: u64 = unsafe {
        // SAFETY: a no-argument call returning a `u64` by value -- no pointers, no allocation,
        // and no precondition on either side beyond the call-ordering constraint this file's own
        // header comment states and this test is the only test in its binary to guarantee.
        slic3r_objectid_shim_wipe_tower_object_id()
    };
    let ported_object_id = wipe_tower_object_id().id as u64;

    let oracle_instance_id: u64 = unsafe {
        // SAFETY: as above.
        slic3r_objectid_shim_wipe_tower_instance_id()
    };
    let ported_instance_id = wipe_tower_instance_id().id as u64;

    // Exact equality with no tolerance, as this row's `exact` equality tier requires.
    compare_unit(
        "ObjectID.cpp",
        Oracle::Built,
        ported_object_id,
        oracle_object_id,
    )
    .expect("wipe_tower_object_id: the first ObjectBase each side constructs");
    compare_unit(
        "ObjectID.cpp",
        Oracle::Built,
        ported_instance_id,
        oracle_instance_id,
    )
    .expect("wipe_tower_instance_id: the second ObjectBase each side constructs");
}
