//! The shape and the round trip in one file (task `19-port-surface`): asserts `Surface`
//! exposes the five calls in order with no out-of-order escape, and round-trips a committed
//! golden through `resolved_config`.

use std::path::{Path, PathBuf};

use grampus_cli::resolved_config::{ConfigField, ResolvedConfig, ResolvedConfigError};
use grampus_cli::surface::{Surface, SurfaceError};

/// `CARGO_MANIFEST_DIR` is `crates/grampus-cli`; the repository root is two levels up — same
/// convention `crates/oracle-compare/tests/normalisation.rs` uses.
fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../")
}

fn golden_path(name: &str) -> PathBuf {
    repo_root().join("port/goldens/configs").join(name)
}

fn test_data_path(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/data")
        .join(name)
}

// --- The five-call shape ---------------------------------------------------------------

/// Calls the five entry points in order. None can succeed yet (nothing under `crates/slic3r-*/`
/// is ported), so every call returns `NotYetPorted` naming itself — the proof here is that
/// calling them *in the right order* never reports `OutOfOrder`, which is the failure mode the
/// next test exercises deliberately.
#[test]
fn five_calls_in_order_each_report_not_yet_ported() {
    let mut surface = Surface::new();

    let mesh_path = repo_root().join("fixtures/20mm_cube.stl");
    let golden = golden_path("base-fff.json");
    let config = ResolvedConfig::from_golden(&golden).expect("base-fff.json is a committed golden");
    let out_path = repo_root().join("oracle/.cache/tmp/surface-test.gcode");

    assert_eq!(
        surface.build_model(&mesh_path),
        Err(SurfaceError::NotYetPorted("build_model"))
    );
    assert_eq!(
        surface.apply_config(&config),
        Err(SurfaceError::NotYetPorted("apply_config"))
    );
    assert_eq!(
        surface.validate(),
        Err(SurfaceError::NotYetPorted("validate"))
    );
    assert_eq!(
        surface.process(),
        Err(SurfaceError::NotYetPorted("process"))
    );
    assert_eq!(
        surface.export_gcode(&out_path),
        Err(SurfaceError::NotYetPorted("export_gcode"))
    );
}

/// `process` called right after `build_model`, skipping `apply_config` and `validate`: the
/// mechanical check, not a reading of this test, is what catches this — `process` reports
/// `OutOfOrder` rather than `NotYetPorted`, because its immediate precondition (`validate`
/// having gone first) does not hold.
#[test]
fn process_before_apply_config_is_out_of_order() {
    let mut surface = Surface::new();
    let mesh_path = repo_root().join("fixtures/20mm_cube.stl");

    surface
        .build_model(&mesh_path)
        .expect_err("build_model is not yet ported");

    assert_eq!(
        surface.process(),
        Err(SurfaceError::OutOfOrder {
            called: "process",
            expected_after: "validate",
        })
    );
}

/// The very first call out of order too: `apply_config` before any `build_model`.
#[test]
fn apply_config_before_build_model_is_out_of_order() {
    let mut surface = Surface::new();
    let golden = golden_path("base-fff.json");
    let config = ResolvedConfig::from_golden(&golden).expect("base-fff.json is a committed golden");

    assert_eq!(
        surface.apply_config(&config),
        Err(SurfaceError::OutOfOrder {
            called: "apply_config",
            expected_after: "build_model",
        })
    );
}

// --- The resolved-configuration round trip ----------------------------------------------

/// Reads a committed golden, reads back the kinds of values the surface needs (bed shape,
/// exclusion areas, wipe-tower placement, filament maps) unchanged, and confirms the absence of
/// the two G-code-only keys does not fail the read and both are reported as derived at emission
/// time rather than read.
#[test]
fn resolved_config_round_trip_on_committed_golden() {
    let golden = golden_path("base-fff.json");
    let config = ResolvedConfig::from_golden(&golden).expect("base-fff.json is a committed golden");

    // provenance is read as the header it is, never as a configuration key.
    assert!(config.provenance().contains_key("oracle-commit"));
    assert_eq!(config.get("provenance"), None);

    // Bed shape.
    assert_eq!(
        config.get("printable_area"),
        Some(ConfigField::Value(serde_json::json!([
            "0x0", "256x0", "256x256", "0x256"
        ])))
    );
    // Exclusion area.
    assert_eq!(
        config.get("bed_exclude_area"),
        Some(ConfigField::Value(serde_json::json!([
            "0x0", "18x0", "18x28", "0x28"
        ])))
    );
    // Wipe-tower placement.
    assert_eq!(
        config.get("wipe_tower_x"),
        Some(ConfigField::Value(serde_json::json!(["15"])))
    );
    assert_eq!(
        config.get("wipe_tower_y"),
        Some(ConfigField::Value(serde_json::json!(["220"])))
    );
    // Filament map.
    assert_eq!(
        config.get("filament_map"),
        Some(ConfigField::Value(serde_json::json!(["1"])))
    );

    // The two G-code-only keys: absent from the golden, but reading them does not fail and
    // both come back reported as derived at emission time.
    assert_eq!(
        config.get("first_layer_temperature"),
        Some(ConfigField::DerivedAtEmission)
    );
    assert_eq!(
        config.get("first_layer_bed_temperature"),
        Some(ConfigField::DerivedAtEmission)
    );

    // A key that is genuinely absent (not one of the two derived keys) reads back `None`, not
    // a derived report and not an empty value.
    assert_eq!(config.get("no_such_key_in_any_golden"), None);
}

/// A golden with no `provenance` header fails with a named error rather than being treated as a
/// configuration with an empty header.
#[test]
fn golden_with_no_provenance_header_fails_with_named_error() {
    let no_provenance = test_data_path("no_provenance.json");
    let result = ResolvedConfig::from_golden(&no_provenance);
    assert!(
        matches!(result, Err(ResolvedConfigError::MissingProvenance { .. })),
        "expected MissingProvenance, got {result:?}"
    );
}
