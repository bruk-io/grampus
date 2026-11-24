//! Exercises `src/json.rs` against small committed fragments under `tests/data/json_*` (no oracle
//! binary, no network) and, under `tests/data/real/`, against the actual pair of settings and
//! statistics dumps task `10-comparator-json` committed from two real `just oracle-slice` runs of
//! `vendor/OrcaSlicer/tests/data/frog_legs.obj base-fff` — see `tests/data/README.md` for which
//! runs and what each reached.

use oracle_compare::json::{compare_settings, compare_statistics, JsonDifference, JsonOutcome};
use oracle_compare::Level;

fn data_dir() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/data")
}

fn read_pair(name: &str, a_name: &str, b_name: &str) -> (Vec<u8>, Vec<u8>) {
    let dir = data_dir().join(name);
    let a = std::fs::read(dir.join(a_name))
        .unwrap_or_else(|e| panic!("reading {}/{a_name}: {e}", dir.display()));
    let b = std::fs::read(dir.join(b_name))
        .unwrap_or_else(|e| panic!("reading {}/{b_name}: {e}", dir.display()));
    (a, b)
}

fn read_json_pair(name: &str) -> (Vec<u8>, Vec<u8>) {
    read_pair(name, "a.json", "b.json")
}

#[test]
fn identical_settings_reach_l0() {
    let (a, b) = read_json_pair("json_settings_identical");
    let report = compare_settings(&a, &b, false);
    assert_eq!(report.outcome, JsonOutcome::Level(Level::L0));
    // L0 implies L1 for a JSON artifact (one outcome, not five independent levels).
    assert!(report.passed(Level::L0));
    assert!(report.passed(Level::L1));
}

#[test]
fn settings_differing_only_in_pipeline_plugin_need_the_allowance_on() {
    let (a, b) = read_json_pair("json_settings_pipeline_plugin");

    // Off by default: the caller that does not ask for the allowance gets `none`, naming a
    // pointer under the `slicing_pipeline_plugin` key as the first (and only) difference — the
    // fixture's two arrays differ in length (`[]` vs `["posSlice"]`, the real armed/unarmed
    // shape), so the walk's most specific pointer is an array index under that key, not the bare
    // key itself; either way it must be *that* key and no other.
    let off = compare_settings(&a, &b, false);
    match &off.outcome {
        JsonOutcome::None(JsonDifference::Pointer { pointer, .. }) => {
            assert!(
                pointer == "/slicing_pipeline_plugin"
                    || pointer.starts_with("/slicing_pipeline_plugin/"),
                "expected a pointer under /slicing_pipeline_plugin, got {pointer}"
            );
        }
        other => panic!("expected a pointer difference with the allowance off, got {other:?}"),
    }
    assert!(!off.passed(Level::L0));
    assert!(!off.passed(Level::L1));

    // On: the same pair reaches L1 (identical after the one declared allowance), never L0 (the
    // raw bytes still differ).
    let on = compare_settings(&a, &b, true);
    assert_eq!(on.outcome, JsonOutcome::Level(Level::L1));
    assert!(!on.passed(Level::L0));
    assert!(on.passed(Level::L1));
}

#[test]
fn settings_differing_in_any_other_key_names_that_key_as_the_pointer() {
    let (a, b) = read_json_pair("json_settings_other_key");
    // The allowance being on must not excuse a difference in an unrelated key.
    for allow in [false, true] {
        let report = compare_settings(&a, &b, allow);
        match &report.outcome {
            JsonOutcome::None(JsonDifference::Pointer { pointer, a, b }) => {
                assert_eq!(pointer, "/layer_height");
                assert_eq!(a.as_ref().unwrap(), "0.2");
                assert_eq!(b.as_ref().unwrap(), "0.3");
            }
            other => panic!("expected a pointer difference at /layer_height, got {other:?}"),
        }
    }
}

#[test]
fn statistics_differing_only_in_generator_reach_l1_not_l0() {
    let (a, b) = read_json_pair("json_statistics_generator");
    let report = compare_statistics(&a, &b);
    assert_eq!(report.outcome, JsonOutcome::Level(Level::L1));
    assert!(!report.passed(Level::L0));
    assert!(report.passed(Level::L1));
}

#[test]
fn statistics_differing_in_a_summary_number_names_the_pointer() {
    let (a, b) = read_json_pair("json_statistics_other_key");
    let report = compare_statistics(&a, &b);
    match &report.outcome {
        JsonOutcome::None(JsonDifference::Pointer { pointer, a, b }) => {
            assert_eq!(pointer, "/summary/time");
            assert_eq!(a.as_ref().unwrap().as_f64(), Some(731.69));
            assert_eq!(b.as_ref().unwrap().as_f64(), Some(800.0));
        }
        other => panic!("expected a pointer difference at /summary/time, got {other:?}"),
    }
}

#[test]
fn malformed_json_is_reported_not_panicked() {
    // a.json is not valid JSON at all (truncated), b.json is. Reported as `none` with the first
    // raw differing byte, never a panic and never this binary's named-IO-error path (that is
    // `main.rs`'s job, for a file that cannot even be *read*, which is not this case).
    let (a, b) = read_json_pair("json_malformed");

    let settings = compare_settings(&a, &b, false);
    match &settings.outcome {
        JsonOutcome::None(JsonDifference::NoPointer {
            first_differing_byte,
        }) => {
            assert!(first_differing_byte.is_some());
        }
        other => panic!("expected an unparseable/no-pointer outcome, got {other:?}"),
    }
    assert!(!settings.passed(Level::L0));
    assert!(!settings.passed(Level::L1));

    let statistics = compare_statistics(&a, &b);
    match &statistics.outcome {
        JsonOutcome::None(JsonDifference::NoPointer {
            first_differing_byte,
        }) => {
            assert!(first_differing_byte.is_some());
        }
        other => panic!("expected an unparseable/no-pointer outcome, got {other:?}"),
    }
}

#[test]
fn no_level_ever_reports_l2_l3_or_l4_for_a_json_artifact() {
    let (a, b) = read_json_pair("json_settings_identical");
    let report = compare_settings(&a, &b, false);
    for level in [Level::L2, Level::L3, Level::L4] {
        assert!(
            !report.passed(level),
            "{level} must never pass a JSON artifact"
        );
    }
}

// --- The real pair (task 10's own obligation: `tests/data/real/`, from two actual
// `just oracle-slice vendor/OrcaSlicer/tests/data/frog_legs.obj base-fff --export-settings
// --export-statistics` runs). See `tests/levels.rs` for the same two runs' G-code excerpt.

fn real_pair(file: &str) -> (Vec<u8>, Vec<u8>) {
    read_pair("real", &format!("run1/{file}"), &format!("run2/{file}"))
}

#[test]
fn real_settings_pair_reaches_l0() {
    // Measured, not assumed: both runs of the pinned oracle on the same fixture and
    // configuration produced byte-identical `settings.json` (diff/cmp confirmed before this test
    // was written — see `tests/data/README.md`).
    let (a, b) = real_pair("settings.json");
    let report = compare_settings(&a, &b, false);
    assert_eq!(report.outcome, JsonOutcome::Level(Level::L0));
}

#[test]
fn real_statistics_pair_reaches_l0() {
    // Also measured: both runs' `plate_1.statistics.json` came out byte-identical, so this
    // reaches `Level::L0`, which is stronger than the `Level::L1` the `generator` allowance alone
    // would guarantee — the oracle's `generator` string did not even need the allowance here
    // because the same build produced both runs.
    let (a, b) = real_pair("plate_1.statistics.json");
    let report = compare_statistics(&a, &b);
    assert_eq!(report.outcome, JsonOutcome::Level(Level::L0));
}
