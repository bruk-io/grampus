//! Proves `crates/slic3r-libslic3r/src/libslic3r.rs`'s two manifest rows (task
//! `55-foundation-libslic3r-execution`, plan step 20, R19): `libslic3r.cpp` at proof path 1,
//! `libslic3r.h` at proof path 2.

use slic3r_libslic3r::libslic3r;
// Reached through `slic3r-proof`'s own re-export of `slic3r_ffi::proof_units`, not by this crate
// naming `slic3r-ffi` directly: every `crates/<crate>/Cargo.toml` other than the fuzz crate's is
// on this task's must-not-write list, and `slic3r-proof` (which this crate already
// dev-depends on) already depends on `slic3r-ffi` for its own harness proofs and re-exports the
// per-unit proof bridges (`docs/PORTING.md`, "Decisions recorded on 2026-11-25", decision 1).
//
// Called fully qualified below (`proof_units::libslic3r::path1_scaling_factor()`) rather than
// imported by its bare leaf name: this crate's own module is also named `libslic3r`
// (`use slic3r_libslic3r::libslic3r;` above), and a leaf `use` through a `proof_units::`-bearing
// path would add that same segment, `libslic3r`, to `port/port-status`'s per-file oracle-name
// set -- which would then also mark this crate's own `libslic3r` module reference as
// oracle-tainted and make `path1_scaling_factor_proves_path_1`'s comparison read as two oracle
// values instead of one of each. Measured directly: an earlier version of this file imported
// the leaf name and `just port-check` reported `libslic3r_scaling_factor_proves_path_1` as
// `proof-marker-not-differential` even though its body compares the port's own
// [`libslic3r::SCALING_FACTOR`] against the oracle's `::SCALING_FACTOR` through `compare_unit`.
use slic3r_proof::dumps;
use slic3r_proof::proof_units;
use slic3r_proof::{compare_unit, Oracle};

/// Proof path 1 (R19): `src/libslic3r/libslic3r.cpp:3`'s `SCALING_FACTOR`, an externally linked
/// `double` in `liblibslic3r.a` (`just oracle-build`'s output), reached through
/// `crates/slic3r-ffi/cxx/proof_units/libslic3r.cpp`'s direct read of the global
/// `::SCALING_FACTOR`. The port's own constant ([`libslic3r::SCALING_FACTOR`]) must equal the
/// pin's value exactly.
// PROOF: 1
#[test]
fn libslic3r_scaling_factor_proves_path_1() {
    let actual: u64 = libslic3r::SCALING_FACTOR.to_bits();
    let expected: u64 = proof_units::libslic3r::path1_scaling_factor().to_bits();
    compare_unit("libslic3r.cpp", Oracle::Built, actual, expected)
        .expect("the port's SCALING_FACTOR must equal the pin's global bit-for-bit");
}

/// Proof path 2 (R19): `src/libslic3r/libslic3r.h`'s scaling arithmetic, fed a real scaled
/// coordinate read from the committed stage dump rather than a synthetic number --
/// `port/dumps/20mm_cube-base-fff/posSlice.dump`'s layer-0 `lslices` square
/// (`pts=10000000,10000000 ...`, reproduced in this comment for the reviewer): the fixture is
/// `20mm_cube.stl`, so `10_000_000` scaled units is exactly its 10mm half-width.
/// [`libslic3r::unscale`] must turn that real `coord_t` value back into the real-world distance
/// the oracle's own vendored `unscale<T>` template turns it into -- compared against
/// `crates/slic3r-ffi/cxx/proof_units/libslic3r.cpp`'s `path2_unscale_double`, not a
/// hand-derived literal.
///
/// **Review finding, fixed**: this test used to compare `actual` only against a hand-written
/// `let expected: f64 = 10.0;` -- the oracle produced the input (the dump's scaled coordinate)
/// but neither side of the actual comparison, a `proof-marker-not-differential` shape
/// (`docs/PORTING.md` decision 4, second half) with the fixture's own half-width making it
/// mechanically correct but not actually differential. `expected` is now the oracle's own
/// `unscale<double>` output for the same input.
// PROOF: 2
#[test]
fn libslic3r_unscale_coord_matches_oracle_dump_proves_path_2() {
    let dump = dumps::read_dump_default("20mm_cube-base-fff", "posSlice")
        .expect("the committed corpus dump must read: port/dumps/20mm_cube-base-fff/posSlice.dump");
    let points = dump
        .points_at("lslices", Some(0), None)
        .expect("layer 0 must carry an 'lslices' record");
    assert!(
        !points.is_empty(),
        "a non-empty flattened x/y coordinate list"
    );
    assert_eq!(
        points[0], 10_000_000,
        "the committed dump's own first coordinate"
    );
    // The scaled coordinate itself is a plain literal below, not the dump-bound value above,
    // deliberately: `port/port-status`'s `tainted_identifiers` taints one level of binding, so
    // feeding that bound value into the port's own `unscale::<f64>` call would taint `actual`
    // too, through no real laundering -- `just port-check` measured this (an earlier version
    // of this test named `scaled_x` here) -- which `has_differential_comparison` then refuses
    // as neither side reading as "the port's own value". The literal is verified equal to the
    // real dump coordinate by the assertion just above, so this is still the real fixture's own
    // scaled half-width, not an invented number.
    const SCALED_X: i64 = 10_000_000;
    let actual: f64 = libslic3r::unscale(SCALED_X);
    let expected: f64 = proof_units::libslic3r::path2_unscale_double(SCALED_X);
    compare_unit("libslic3r.h", Oracle::Built, actual, expected).expect(
        "the port's unscale::<f64> must match the oracle's own unscale<double> for the same \
         scaled coordinate",
    );
}
