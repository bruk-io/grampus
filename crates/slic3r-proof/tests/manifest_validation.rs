//! Exercises `slic3r-proof`'s manifest-row validation and both tiers directly against a
//! committed fixture (task 31-proof-harness, R19's manifest-row rules), bypassing
//! `GRAMPUS_MANIFEST_PATH` through `compare_unit_at`. The real `port/manifest.tsv` has no row
//! exercising the float tier or either malformed-row refusal today -- every real row's
//! `equality tier` is `exact` with an empty `tolerance` (`tests/harness_paths.rs` proves the
//! real default-path, real-row case) -- so these rows are synthetic, but the parsing and
//! comparison logic they exercise is `tiers.rs`'s real code, not a stand-in for it.

use slic3r_proof::tiers::{compare_unit_at, Oracle, TierError};

fn fixture_path() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/data/manifest/fixture.tsv")
}

#[test]
fn exact_row_compares_bit_for_bit() {
    compare_unit_at(
        fixture_path(),
        "Fixture/ExactRow.cpp",
        Oracle::Built,
        7i64,
        7i64,
    )
    .expect("equal integers at an exact tier must pass");
}

#[test]
fn exact_row_rejects_any_difference() {
    let err = compare_unit_at(
        fixture_path(),
        "Fixture/ExactRow.cpp",
        Oracle::Built,
        7i64,
        8i64,
    )
    .expect_err("differing integers at an exact tier must fail");
    assert!(matches!(err, TierError::Mismatch { .. }), "{err}");
}

#[test]
fn float_row_passes_within_its_declared_tolerance() {
    compare_unit_at(
        fixture_path(),
        "Fixture/FloatRow.cpp",
        Oracle::Built,
        1.000_05_f64,
        1.0_f64,
    )
    .expect("0.00005 deviation is within the fixture's declared 0.01 tolerance");
}

#[test]
fn float_row_fails_outside_its_declared_tolerance() {
    let err = compare_unit_at(
        fixture_path(),
        "Fixture/FloatRow.cpp",
        Oracle::Built,
        2.0_f64,
        1.0_f64,
    )
    .expect_err("1.0 deviation exceeds the fixture's declared 0.01 tolerance");
    assert!(matches!(err, TierError::ToleranceExceeded { .. }), "{err}");
}

#[test]
fn a_tolerance_with_no_reason_is_refused_and_names_the_row() {
    let err = compare_unit_at(
        fixture_path(),
        "Fixture/NoReason.cpp",
        Oracle::Built,
        1.0_f64,
        1.0_f64,
    )
    .expect_err("a tolerance cell with a value and no reason must be refused");
    assert!(
        err.to_string().contains("Fixture/NoReason.cpp"),
        "error should name the row: {err}"
    );
    assert!(
        matches!(err, TierError::ToleranceMissingReason { .. }),
        "{err}"
    );
}

#[test]
fn an_empty_proof_path_is_refused_and_names_the_row() {
    let err = compare_unit_at(
        fixture_path(),
        "Fixture/EmptyProofPath.cpp",
        Oracle::Built,
        1i64,
        1i64,
    )
    .expect_err("an empty proof path cell must be refused even though the values match");
    assert!(
        err.to_string().contains("Fixture/EmptyProofPath.cpp"),
        "error should name the row: {err}"
    );
    assert!(matches!(err, TierError::EmptyProofPath { .. }), "{err}");
}

#[test]
fn an_unimplemented_comparison_function_is_refused() {
    let err = compare_unit_at(
        fixture_path(),
        "Fixture/BadFunction.cpp",
        Oracle::Built,
        1.0_f64,
        1.0_f64,
    )
    .expect_err("'rel' is not an implemented comparison function");
    assert!(
        err.to_string().contains("rel"),
        "error should name it: {err}"
    );
    assert!(
        matches!(err, TierError::UnimplementedComparisonFunction { .. }),
        "{err}"
    );
}

#[test]
fn a_float_row_with_no_declared_tolerance_is_refused() {
    let err = compare_unit_at(
        fixture_path(),
        "Fixture/NoToleranceFloat.cpp",
        Oracle::Built,
        1.0_f64,
        1.0_f64,
    )
    .expect_err("a float-tier row must declare a tolerance");
    assert!(
        err.to_string().contains("Fixture/NoToleranceFloat.cpp"),
        "error should name the row: {err}"
    );
    assert!(
        matches!(err, TierError::FloatRowMissingTolerance { .. }),
        "{err}"
    );
}

#[test]
fn an_absent_manifest_file_is_refused_named() {
    let err = compare_unit_at(
        "tests/data/manifest/does-not-exist.tsv",
        "Fixture/ExactRow.cpp",
        Oracle::Built,
        1i64,
        1i64,
    )
    .expect_err("a missing manifest file must fail named, not compare at a default");
    assert!(matches!(err, TierError::ManifestMissing { .. }), "{err}");
}

#[test]
fn an_unknown_row_is_refused_and_names_the_row() {
    let err = compare_unit_at(
        fixture_path(),
        "Fixture/NoSuchRow.cpp",
        Oracle::Built,
        1i64,
        1i64,
    )
    .expect_err("no row is named this");
    assert!(
        err.to_string().contains("Fixture/NoSuchRow.cpp"),
        "error should name it: {err}"
    );
    assert!(matches!(err, TierError::RowNotFound { .. }), "{err}");
}
