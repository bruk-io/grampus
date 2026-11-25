//! Proves the `GRAMPUS_MANIFEST_PATH` half of `compare_unit`'s manifest-location contract (task
//! 31-proof-harness, R19's "locates `port/manifest.tsv` from an environment variable" Done-when
//! line) through the one real API the whole fan-out calls -- `compare_unit`, not
//! `resolve_manifest_path` alone and not `compare_unit_at`, which bypasses the variable on
//! purpose and is exactly why this half went unproven before this file existed (see this task's
//! pull request body).
//!
//! This lives in its own integration-test *file* rather than adding a `#[test]` to
//! `harness_paths.rs` or `manifest_validation.rs`: each file under `tests/` compiles to its own
//! process, so setting `GRAMPUS_MANIFEST_PATH` here can never race another test file's own use of
//! (or reliance on the absence of) that variable. The file has exactly one `#[test]`, so there is
//! also no same-process race between two tests over the same process-wide environment variable.
//!
//! The manifest this test points at is written to `oracle/.cache/tmp/` (git-ignored, this task's
//! own scratch area) at test run time, not committed, and its one row's `path` cell
//! (`Fixture/ManifestEnvVar.cpp`) names a unit that does not exist in the real
//! `port/manifest.tsv`. If `compare_unit` ever silently fell back to the workspace-root default
//! instead of honouring the variable, that row would not be found there either and this test
//! would fail with `TierError::RowNotFound`, not pass for the wrong reason.

use std::fs;
use std::io::Write;

use slic3r_proof::{compare_unit, Oracle};

#[test]
fn compare_unit_honours_the_manifest_path_env_var() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../oracle/.cache/tmp");
    fs::create_dir_all(&dir).expect("oracle/.cache/tmp must be creatable (it is git-ignored)");
    let manifest_path = dir.join("manifest_env_var_test.tsv");

    let mut file = fs::File::create(&manifest_path).expect("write the scratch manifest");
    writeln!(
        file,
        "path\tsubsystem\tlines\tfile-level include-DAG level\tcrate-level level\tin scope or out with a reason\ttarget crate\ttarget module\tpipeline stage or none\ttemplate-heavy\tequality tier\ttolerance\tcomparison function\tproof path\tfuzz target\tstatus"
    )
    .unwrap();
    writeln!(
        file,
        "Fixture/ManifestEnvVar.cpp\troot\t1\t0\t0\tin\tfixture-crate\tcrates/fixture-crate/src/manifest_env_var.rs\tnone\tno\texact\t\t\t1\tn/a\tnot-started"
    )
    .unwrap();
    drop(file);

    // SAFETY (of the test, not the process): this file has exactly one `#[test]`, so there is no
    // other thread in this binary that could observe a half-set variable.
    std::env::set_var(slic3r_proof::tiers::MANIFEST_PATH_VAR, &manifest_path);

    let result = compare_unit("Fixture/ManifestEnvVar.cpp", Oracle::Built, 1i64, 1i64);

    std::env::remove_var(slic3r_proof::tiers::MANIFEST_PATH_VAR);

    result.expect(
        "compare_unit must read the row from GRAMPUS_MANIFEST_PATH's file, not the workspace-root \
         default -- the row only exists in the scratch manifest this test just wrote",
    );
}
