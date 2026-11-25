//! The per-unit proof harness's own proof (plan step 18, R19): one test
//! per proof path, each compared through `slic3r_proof::tiers::compare_unit` at that unit's real
//! `port/manifest.tsv` tier. These two predate any ported Rust module, so `expected` here is the
//! value read from the vendored C++ directly (see each test's doc comment), not a value a
//! stand-in computes -- there was no port to compare the oracle against, which is exactly what a
//! harness proof checks instead: that the FFI call, the included translation unit and
//! `tiers::compare_unit`'s dispatch all actually reach the pin's own compiled code and compare
//! what they claim to. The units themselves are proven against their ports in
//! `crates/slic3r-objectid` and (when it is ported) `crates/slic3r-clipperzutils`.
//!
//! CONSTRAINT ON THIS FILE: the path-1 proof's expected value (`1`) holds only because nothing
//! else in this test binary constructs a `Slic3r::ObjectBase` before `wipe_tower_object_id()` is
//! first called (`ObjectID.cpp`'s `s_last_id` is a process-global counter shared by every
//! `ObjectBase`, and `wipe_tower_object_id()`'s `static ObjectBase mine` is the first one built
//! once that call runs). Do not add a test to this file that constructs or reaches an
//! `ObjectBase` before `object_id_wipe_tower_object_id_proves_path_1` has run -- it would turn
//! this proof into an order-dependent flake rather than a compile error. A test needing an
//! `ObjectBase` belongs in its own integration-test file instead.
//!
//! That constraint is about this file. It is **not** a claim that nothing else linked into this
//! binary can touch `Slic3r::ObjectBase`: the counter is the linked `liblibslic3r.a`'s, shared
//! with every libslic3r translation unit the link extracts. Seven archive members reference
//! `s_last_id` and none is extracted into this binary today, measured with `nm` -- see
//! `crates/slic3r-ffi/cxx/proof_units/object_id.cpp`'s header comment, which names them.

use slic3r_proof::proof_units::clipper_z_utils::{path3_to_paths64, ProofZPoint};
use slic3r_proof::proof_units::object_id::path1_wipe_tower_object_id;
use slic3r_proof::{compare_unit, Oracle};

/// Proof path 1 (R19): `src/libslic3r/ObjectID.cpp`'s `wipe_tower_object_id()`, an externally
/// linked symbol in `liblibslic3r.a` (`just oracle-build`'s output,
/// `vendor/OrcaSlicer/build/arm64/src/libslic3r/Release/liblibslic3r.a`, reached through
/// `crates/slic3r-ffi/build.rs`'s derived link line). Deterministic in this binary:
/// it constructs the first `Slic3r::ObjectBase` this whole test binary ever makes
/// (`ObjectID.cpp`'s file-static `s_last_id` starts at `0`, and no archive member this binary
/// extracts constructs an `ObjectBase` before it -- see the constraint above), so it returns `1`.
#[test]
fn object_id_wipe_tower_object_id_proves_path_1() {
    let actual: u64 = path1_wipe_tower_object_id();
    let expected: u64 = 1;
    compare_unit("ObjectID.cpp", Oracle::Built, actual, expected)
        .expect("wipe_tower_object_id is the first ObjectBase this process constructs");
}

/// Proof path 3 (R19): `src/libslic3r/ClipperZUtils.cpp`'s `static` (internal-linkage)
/// `to_paths64`, reached only because `cxx/proof_units/clipper_z_utils.cpp` in the FFI crate
/// `#include`s that vendored file (see its header comment for the inclusion rule). It
/// repacks a `Slic3r::ClipperZUtils::ZPath` into a `Clipper2Lib_Z::Path64` with no arithmetic, so
/// every coordinate must come back unchanged.
#[test]
fn clipper_z_utils_to_paths64_proves_path_3() {
    let input = vec![
        ProofZPoint {
            x: 10,
            y: 20,
            z: 30,
        },
        ProofZPoint { x: -5, y: 0, z: 7 },
        ProofZPoint {
            x: 100,
            y: -100,
            z: -1,
        },
    ];
    let actual: Vec<i64> = path3_to_paths64(&input)
        .into_iter()
        .flat_map(|p| [p.x, p.y, p.z])
        .collect();
    let expected: Vec<i64> = input.into_iter().flat_map(|p| [p.x, p.y, p.z]).collect();
    compare_unit("ClipperZUtils.cpp", Oracle::Built, actual, expected)
        .expect("to_paths64 is a pure repack: every coordinate comes back unchanged");
}
