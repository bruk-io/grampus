//! `just compare-stages` (task `39-compare-stages`, R20). Covers `stages.rs`'s dispatch end to
//! end against committed fragments and a committed fixture tree, plus the full `stages::run`
//! dispatch against a trivial stand-in for `grampus-cli` (`/usr/bin/true`) for the cases that do
//! not depend on the port binary's own output: "not ported", and "a fully-ported stage with no
//! committed oracle dump". `grampus-cli`'s real binary writes no per-seam dump today (nothing
//! under `crates/slic3r-*/` is ported — see `crates/grampus-cli/src/surface.rs`), so the
//! "matches"/"differs" and fallback cases below exercise the real parsing and comparison code
//! directly against committed fragment pairs rather than through a subprocess that cannot yet
//! produce the port side of either comparison; see this task's pull request body.

use std::path::{Path, PathBuf};

use oracle_compare::stages::{self, DumpOutcome, StageResult};
use oracle_compare::tolerance::Tolerance;

fn data_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/data/stages")
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repo root exists")
}

fn scratch_dir(name: &str) -> PathBuf {
    let dir = repo_root()
        .join("oracle/.cache/tmp/oracle-compare-stages-test")
        .join(format!("{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("can create scratch dir");
    dir
}

// --- "A stage that matches" / "a stage that differs" (committed dump fragments) -------------

#[test]
fn committed_dump_fragment_pair_that_matches() {
    let dir = data_dir().join("dump_match");
    let oracle = stages::read_dump_records(&dir.join("oracle.dump")).expect("oracle.dump reads");
    let port = stages::read_dump_records(&dir.join("port.dump")).expect("port.dump reads");
    assert_eq!(
        stages::compare_dump_records(&oracle, &port, &Tolerance::Exact),
        DumpOutcome::Match
    );
}

#[test]
fn committed_dump_fragment_pair_that_differs() {
    let dir = data_dir().join("dump_differ");
    let oracle = stages::read_dump_records(&dir.join("oracle.dump")).expect("oracle.dump reads");
    let port = stages::read_dump_records(&dir.join("port.dump")).expect("port.dump reads");
    match stages::compare_dump_records(&oracle, &port, &Tolerance::Exact) {
        DumpOutcome::Differ { record_index, .. } => assert_eq!(record_index, 3),
        other => panic!("expected a difference, got {other:?}"),
    }
}

// --- The fallback path (committed exported-G-code fragments) --------------------------------

#[test]
fn fallback_on_identical_committed_gcode_reports_fallback_and_never_names_last_seam() {
    // Reuses task 08's own committed identical pair rather than adding a third copy of the same
    // bytes: both are real committed fragments, which is what task.md asks this case to cover.
    let gcode_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/data/identical");
    let oracle = std::fs::read(gcode_dir.join("a.gcode")).expect("a.gcode reads");
    let port = std::fs::read(gcode_dir.join("b.gcode")).expect("b.gcode reads");

    let outcome = stages::evaluate_fallback(&oracle, &port, Tolerance::Exact);
    assert!(
        !outcome.differs,
        "a byte-identical pair must not read as differing"
    );

    let report = stages::StagesReport {
        lines: vec![stages::StageLine {
            stage: "psGCodeExport".to_string(),
            result: StageResult::Fallback {
                tolerance: Tolerance::Exact,
                outcome,
            },
        }],
        first_differing_stage: None,
    };
    let rendered = stages::render_text(&report);
    assert!(rendered.contains("fallback: exported G-code"), "{rendered}");
    assert!(
        !rendered.contains("psSkirtBrim"),
        "fallback must never name the last seam as the first difference: {rendered}"
    );
}

// --- The full dispatch: "not ported" and "missing dump" (fixture tree + a stand-in port binary)

fn run_dir(name: &str) -> PathBuf {
    let dir = scratch_dir(name);
    dir
}

fn stand_in_port_bin() -> PathBuf {
    // `/usr/bin/true`: accepts and ignores any arguments, always exits 0. Good enough for the
    // two cases below, neither of which depends on what the port binary actually wrote --
    // "not ported" never looks at its output at all, and "missing oracle dump" fails before
    // either side's dump is read (see `stages::run`'s own ordering).
    PathBuf::from("/usr/bin/true")
}

#[test]
fn not_ported_stage_is_reported_without_touching_any_dump() {
    // A manifest holding only the one not-ported row -- not the shared `manifest.tsv`, whose
    // other rows derive `ported` and would need the stand-in port binary to have actually
    // written their dumps, which `/usr/bin/true` never does (see the two tests below, which
    // exercise exactly that).
    let manifest = data_dir().join("fixture_tree/manifest_not_ported_only.tsv");
    let tree = data_dir().join("fixture_tree");
    let scratch = run_dir("not-ported");

    let cfg = stages::StagesConfig {
        manifest_path: &manifest,
        tree: &tree,
        oracle_dumps_dir: &data_dir().join("oracle_dumps"),
        port_bin: &stand_in_port_bin(),
        fixture: &repo_root().join("fixtures/20mm_cube.stl"),
        config: &repo_root().join("port/goldens/configs/base-fff.json"),
        scratch_dir: &scratch,
    };

    let report = stages::run(&cfg)
        .expect("run succeeds: nothing fully ported but the one row this stage needs");
    let line = report
        .lines
        .iter()
        .find(|l| l.stage == "myNotPortedStage")
        .expect("myNotPortedStage has a line");
    assert!(matches!(line.result, StageResult::NotPorted));
}

#[test]
fn fully_ported_stage_with_no_committed_oracle_dump_fails_naming_the_fix() {
    let manifest = data_dir().join("fixture_tree/manifest.tsv");
    let tree = data_dir().join("fixture_tree");
    let scratch = run_dir("missing-oracle-dump");

    // `oracle_dumps_dir` here deliberately holds nothing at all, so `myMissingDumpStage.dump`
    // (its row's module is the same `ported_seam.rs` the fixture tree's `myDumpStage` row also
    // uses, and derives `ported` the same way) is absent.
    let empty_dumps_dir = scratch.join("empty-oracle-dumps");
    std::fs::create_dir_all(&empty_dumps_dir).unwrap();

    let cfg = stages::StagesConfig {
        manifest_path: &manifest,
        tree: &tree,
        oracle_dumps_dir: &empty_dumps_dir,
        port_bin: &stand_in_port_bin(),
        fixture: &repo_root().join("fixtures/20mm_cube.stl"),
        config: &repo_root().join("port/goldens/configs/base-fff.json"),
        scratch_dir: &scratch,
    };

    let err = stages::run(&cfg).expect_err("a fully-ported stage with no oracle dump must fail");
    assert!(
        err.contains("just oracle-dump 3DBenchy base-fff"),
        "error should name the fix: {err}"
    );
}

#[test]
fn fully_ported_stage_whose_port_run_wrote_no_dump_is_an_internal_inconsistency_not_a_pass() {
    let manifest = data_dir().join("fixture_tree/manifest.tsv");
    let tree = data_dir().join("fixture_tree");
    let scratch = run_dir("no-port-dump");

    // The committed oracle dump directory does carry `myDumpStage.dump`. `myMissingDumpStage`
    // and `myNoPortDumpStage` both point at the same `ported_seam.rs` module and so both derive
    // `ported` too, but neither has a `.dump` of its own in `oracle_dumps/` -- `stages::run`
    // checks the oracle side before the port side, so this test's row order (`manifest.tsv`
    // lists `myMissingDumpStage` before `myNoPortDumpStage`) would hit that error first unless
    // this test gives `myMissingDumpStage` a dump too. It does, via a tiny ad hoc copy, so this
    // test actually reaches and proves the *other* branch: the port's own `--dump-stages` run
    // (here, `/usr/bin/true`, which writes nothing) produced no dump for a stage this run
    // derives as fully ported.
    let dumps_dir = scratch.join("oracle-dumps");
    std::fs::create_dir_all(&dumps_dir).unwrap();
    std::fs::copy(
        data_dir().join("oracle_dumps/myDumpStage.dump"),
        dumps_dir.join("myDumpStage.dump"),
    )
    .unwrap();
    std::fs::copy(
        data_dir().join("oracle_dumps/myDumpStage.dump"),
        dumps_dir.join("myMissingDumpStage.dump"),
    )
    .unwrap();
    std::fs::copy(
        data_dir().join("oracle_dumps/myDumpStage.dump"),
        dumps_dir.join("myNoPortDumpStage.dump"),
    )
    .unwrap();

    let cfg = stages::StagesConfig {
        manifest_path: &manifest,
        tree: &tree,
        oracle_dumps_dir: &dumps_dir,
        port_bin: &stand_in_port_bin(),
        fixture: &repo_root().join("fixtures/20mm_cube.stl"),
        config: &repo_root().join("port/goldens/configs/base-fff.json"),
        scratch_dir: &scratch,
    };

    let err = stages::run(&cfg)
        .expect_err("a fully-ported stage the port's own run produced no dump for must fail");
    assert!(
        err.contains("internal inconsistency"),
        "error should name it as an internal inconsistency, not silently pass: {err}"
    );
}
