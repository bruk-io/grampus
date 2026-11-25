//! `just compare-stages` (task `39-compare-stages`, R20).
//!
//! Every test here drives the real comparison code over real files. Nothing stands in for the
//! code under test and nothing stands in for the oracle (R19):
//!
//! * The code under test is `oracle_compare::stages` — the status derivation, the tier read, the
//!   per-stage dump comparison, the seamless-stage fallback and the rendering. It is called
//!   directly, never through a script pretending to be something else.
//! * The oracle's side of every comparison is a real dump or a real exported-G-code fragment
//!   read from an `oracle_dumps_dir`, in the format `oracle/stage-dump/dump_seams.cpp` writes.
//! * The port's side is a directory of dump/G-code files, handed to `compare_stage_outputs`
//!   through [`stages::PortOutputs`] — the same struct `stages::run` fills from a real
//!   `grampus-cli` subprocess. Those files are fixture *data* for the comparison, the way
//!   `tests/data/identical/*.gcode` is fixture data for `levels.rs`. No test here invokes
//!   anything in the port's place. `run`'s own subprocess half is covered without a stand-in
//!   too, in two places that are not this file: `just compare-stages` drives the real
//!   `grampus-cli` binary through `run` end to end (every stage reads `not ported`, exit 0 —
//!   today's truthful state, recorded in this task's pull request body), and
//!   `crates/grampus-cli/tests/dump_stages.rs` runs that same real binary directly to prove what
//!   it does with `--dump-stages`: accepts the flag, exits at `Surface::build_model`, writes no
//!   dump. That file does not call `run_port_binary`; what it pins down is the behaviour on the
//!   far side of it.
//!
//! An earlier revision of this file reached `run`'s match/differ/fallback branches by spawning a
//! committed shell script that parsed `grampus-cli`'s flags and wrote dumps in its place. That
//! script is gone: `stages::run` is now a thin spawn-then-delegate wrapper over
//! `stages::compare_stage_outputs`, which is where all of that logic lives and which these tests
//! call directly. One claim did not survive the change — the `oracle-compare --stages` binary's
//! own non-zero exit on a differing stage, which no test can drive end to end until the port
//! emits its first dump. See this task's pull request body.

use std::path::{Path, PathBuf};

use oracle_compare::stages::{self, DumpOutcome, PortOutputs, StageResult};
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

fn fixture_tree() -> PathBuf {
    data_dir().join("fixture_tree")
}

/// Creates `scratch/<name>` and returns it.
fn subdir(scratch: &Path, name: &str) -> PathBuf {
    let dir = scratch.join(name);
    std::fs::create_dir_all(&dir).expect("can create a fixture directory");
    dir
}

/// Writes a stage dump at `path` in the committed dump format, carrying `seam` in its own
/// `; seam:` header (`stages::dump_seam_header` reads it back) and `value` as its one
/// `field=print_z` record's hex-float payload.
fn write_dump(path: &Path, seam: &str, value: &str) {
    std::fs::write(
        path,
        format!(
            "; provenance\n\
             ;   oracle-commit: f92a7c065c5718b8ef1baac1cd704df2911238f4\n\
             ;   oracle: built (unrecorded)\n\
             ; seam: {seam}\n\
             ; object: 0\n\
             field=print_z layer=0 region=- value={value}\n"
        ),
    )
    .expect("can write a fixture dump");
}

/// The port side of a comparison, as `stages::run` hands it over after a real port run. The
/// `diagnostic` here is what `run` fills with `describe_port_output`; a caller that ran no port
/// process says so, and these tests assert that text is carried into the port-side errors rather
/// than dropped.
const NO_PORT_RUN: &str = " (no port process was run by this test)";

fn port_outputs<'a>(dump_dir: &'a Path, gcode_dir: &'a Path) -> PortOutputs<'a> {
    PortOutputs {
        dump_dir,
        gcode_dir,
        diagnostic: NO_PORT_RUN,
    }
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

// --- The full dispatch: not ported --------------------------------------------------------

#[test]
fn not_ported_stage_is_reported_without_touching_any_dump() {
    // A manifest holding only the one not-ported row, whose `target module` names a file that
    // genuinely does not exist in the fixture tree.
    let scratch = scratch_dir("not-ported");
    let report = stages::compare_stage_outputs(
        &fixture_tree().join("manifest_not_ported_only.tsv"),
        &fixture_tree(),
        &subdir(&scratch, "oracle-dumps"),
        &port_outputs(
            &subdir(&scratch, "port-dumps"),
            &subdir(&scratch, "port-gcode"),
        ),
    )
    .expect("run succeeds: nothing is fully ported, which is not a failure");

    let line = report
        .lines
        .iter()
        .find(|l| l.stage == "myNotPortedStage")
        .expect("myNotPortedStage has a line");
    assert!(matches!(line.result, StageResult::NotPorted));
    assert!(report.first_differing_stage.is_none());
    assert!(
        stages::render_text(&report).contains("myNotPortedStage: not ported"),
        "{}",
        stages::render_text(&report)
    );
}

// --- The full dispatch: a missing committed oracle artifact ---------------------------------

#[test]
fn fully_ported_stage_with_no_committed_oracle_dump_fails_naming_the_fix() {
    let scratch = scratch_dir("missing-oracle-dump");
    // `oracle-dumps` here deliberately holds nothing at all, so `myDumpStage.dump` (the first
    // stage this run reaches) is absent.
    let err = stages::compare_stage_outputs(
        &fixture_tree().join("manifest.tsv"),
        &fixture_tree(),
        &subdir(&scratch, "oracle-dumps"),
        &port_outputs(
            &subdir(&scratch, "port-dumps"),
            &subdir(&scratch, "port-gcode"),
        ),
    )
    .expect_err("a fully-ported stage with no oracle dump must fail");
    assert!(
        err.contains("just oracle-dump 3DBenchy base-fff"),
        "error should name the fix: {err}"
    );
}

// --- The full dispatch: a matching and a differing stage ------------------------------------

#[test]
fn a_matching_dump_stage_is_reported_as_a_match_at_the_tier_its_rows_declare() {
    let scratch = scratch_dir("dump-match");
    let oracle_dumps = subdir(&scratch, "oracle-dumps");
    let port_dumps = subdir(&scratch, "port-dumps");
    write_dump(
        &oracle_dumps.join("myDumpStage.dump"),
        "myDumpStage",
        "0x1p0",
    );
    write_dump(&port_dumps.join("myDumpStage.dump"), "myDumpStage", "0x1p0");

    let report = stages::compare_stage_outputs(
        &fixture_tree().join("manifest_dump_only.tsv"),
        &fixture_tree(),
        &oracle_dumps,
        &port_outputs(&port_dumps, &subdir(&scratch, "port-gcode")),
    )
    .expect("a byte-identical dump pair must not error");

    assert_eq!(report.lines.len(), 1);
    match &report.lines[0].result {
        StageResult::Dump { outcome, .. } => assert_eq!(*outcome, DumpOutcome::Match),
        other => panic!("expected a Dump result, got {other:?}"),
    }
    assert!(report.first_differing_stage.is_none());
    // The tier the row declares (`exact`) is printed, not assumed -- task.md's "it prints which
    // tier it held each compared stage to".
    let rendered = stages::render_text(&report);
    assert!(
        rendered.contains("myDumpStage: exact -- match"),
        "{rendered}"
    );
}

#[test]
fn the_first_differing_dump_stage_is_named() {
    let scratch = scratch_dir("dump-differ");
    let oracle_dumps = subdir(&scratch, "oracle-dumps");
    let port_dumps = subdir(&scratch, "port-dumps");
    write_dump(
        &oracle_dumps.join("myDumpStage.dump"),
        "myDumpStage",
        "0x1p0",
    );
    // A deliberately differing value -- 0x1p1 (2.0) against the oracle's 0x1p0 (1.0).
    write_dump(&port_dumps.join("myDumpStage.dump"), "myDumpStage", "0x1p1");

    let report = stages::compare_stage_outputs(
        &fixture_tree().join("manifest_dump_only.tsv"),
        &fixture_tree(),
        &oracle_dumps,
        &port_outputs(&port_dumps, &subdir(&scratch, "port-gcode")),
    )
    .expect("a differing dump pair is reported, not an error");

    assert_eq!(report.first_differing_stage.as_deref(), Some("myDumpStage"));
    match &report.lines[0].result {
        StageResult::Dump {
            outcome: DumpOutcome::Differ { record_index, .. },
            ..
        } => assert_eq!(*record_index, 0),
        other => panic!("expected a differing Dump result, got {other:?}"),
    }
    assert!(stages::render_text(&report).contains("first differing stage: myDumpStage"));
}

// --- The full dispatch: the two seamless stages fall back to the exported G-code -------------

#[test]
fn fallback_stages_say_so_and_never_name_the_preceding_seam_as_their_difference() {
    // The whole of task.md's fallback "Done when", through the real dispatch rather than a
    // hand-built report: a manifest with one ordinary seam (`psSkirtBrim`, made to differ) ahead
    // of the two seamless stages, so the assertion that neither fallback line names `psSkirtBrim`
    // is capable of failing -- in a report that does not mention `psSkirtBrim` at all it would
    // hold for any possible rendering.
    let scratch = scratch_dir("fallback-dispatch");
    let oracle_dumps = subdir(&scratch, "oracle-dumps");
    let port_dumps = subdir(&scratch, "port-dumps");
    let port_gcode = subdir(&scratch, "port-gcode");

    write_dump(
        &oracle_dumps.join("psSkirtBrim.dump"),
        "psSkirtBrim",
        "0x1p0",
    );
    write_dump(&port_dumps.join("psSkirtBrim.dump"), "psSkirtBrim", "0x1p1");

    // Task 08's own committed identical exported-G-code pair, rather than a third copy of the
    // same bytes: the oracle's exported G-code and the port's, both real committed fragments.
    let identical = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/data/identical");
    std::fs::copy(
        identical.join("a.gcode"),
        oracle_dumps.join("plate_1.gcode"),
    )
    .unwrap();
    std::fs::copy(identical.join("b.gcode"), port_gcode.join("plate_1.gcode")).unwrap();

    let report = stages::compare_stage_outputs(
        &fixture_tree().join("manifest_seam_and_fallback.tsv"),
        &fixture_tree(),
        &oracle_dumps,
        &port_outputs(&port_dumps, &port_gcode),
    )
    .expect("a differing seam plus two matching fallback stages is a report, not an error");

    // The two seamless stages were routed to the G-code fallback, not to the dump path.
    for stage in ["psGCodeExport", "psConflictCheck"] {
        let line = report
            .lines
            .iter()
            .find(|l| l.stage == stage)
            .unwrap_or_else(|| panic!("{stage} has a line"));
        assert!(
            matches!(line.result, StageResult::Fallback { .. }),
            "{stage} must be routed to the exported-G-code fallback, got {:?}",
            line.result
        );
    }

    // `psSkirtBrim`, an ordinary seam, really did differ and really is named as the first
    // difference -- so the fallback lines below are rendered in a report that does mention it.
    assert_eq!(
        report.first_differing_stage.as_deref(),
        Some("psSkirtBrim"),
        "the differing ordinary seam must be the first differing stage"
    );

    let rendered = stages::render_text(&report);
    let fallback_lines: Vec<&str> = rendered
        .lines()
        .filter(|l| l.starts_with("psGCodeExport:") || l.starts_with("psConflictCheck:"))
        .collect();
    assert_eq!(
        fallback_lines.len(),
        2,
        "expected one line each for psGCodeExport and psConflictCheck: {rendered}"
    );
    for line in &fallback_lines {
        assert!(
            line.contains("fallback: exported G-code"),
            "{line} should say it is in fallback: {rendered}"
        );
        assert!(
            !line.contains("psSkirtBrim"),
            "a fallback line must never name the preceding seam as its own difference: {line}"
        );
    }
}

#[test]
fn a_fully_ported_fallback_stage_with_no_oracle_gcode_fails_on_the_gcode_path() {
    // Proves the dispatch decision (`is_fallback_stage`) and not just `evaluate_fallback` in
    // isolation: a manifest naming only `psGCodeExport`/`psConflictCheck`, both fully ported,
    // against an empty oracle-dumps directory. The dump-record branch would fail naming
    // `psGCodeExport.dump`; the fallback branch fails naming `plate_1.gcode` instead -- the error
    // text alone tells us which branch ran.
    let scratch = scratch_dir("fallback-routing");
    let err = stages::compare_stage_outputs(
        &fixture_tree().join("manifest_fallback_only.tsv"),
        &fixture_tree(),
        &subdir(&scratch, "oracle-dumps"),
        &port_outputs(
            &subdir(&scratch, "port-dumps"),
            &subdir(&scratch, "port-gcode"),
        ),
    )
    .expect_err("a fully-ported fallback stage with no oracle gcode must fail");
    assert!(
        err.contains("plate_1.gcode"),
        "expected the fallback (gcode) path to run, naming plate_1.gcode: {err}"
    );
    assert!(
        !err.contains(".dump"),
        "the dump-record path must not have run for a fallback stage: {err}"
    );
}

// --- The full dispatch: the port's side is missing ------------------------------------------

#[test]
fn fully_ported_stage_with_no_port_dump_at_all_is_reported_as_the_structural_gap_it_is() {
    // Today's real state for every stage: the oracle dump is committed, and the port's dump
    // directory is empty because `emit_stage_dump` (crates/grampus-cli/src/surface.rs) is a
    // documented no-op at every seam. `crates/grampus-cli/tests/dump_stages.rs` proves the real
    // binary writes nothing; this proves the comparison refuses to read that as a pass.
    let scratch = scratch_dir("no-port-dump-at-all");
    let oracle_dumps = subdir(&scratch, "oracle-dumps");
    write_dump(
        &oracle_dumps.join("myDumpStage.dump"),
        "myDumpStage",
        "0x1p0",
    );

    let err = stages::compare_stage_outputs(
        &fixture_tree().join("manifest_dump_only.tsv"),
        &fixture_tree(),
        &oracle_dumps,
        &port_outputs(
            &subdir(&scratch, "port-dumps"),
            &subdir(&scratch, "port-gcode"),
        ),
    )
    .expect_err("a fully-ported stage the port wrote no dump for must fail, not pass");
    assert!(
        err.contains("wrote no stage dump at all") && err.contains("emit_stage_dump"),
        "error should name the structural gap and the call site: {err}"
    );
    assert!(
        err.contains(NO_PORT_RUN),
        "the port run's own diagnostic must be carried into the error, not dropped: {err}"
    );
}

#[test]
fn fully_ported_stage_missing_from_a_non_empty_port_dump_dir_is_an_inconsistency_not_a_pass() {
    // The other port-side case: the port wrote *some* stage dumps but not this stage's, which is
    // an inconsistency between the status derivation and the port's own `Surface` rather than a
    // pass. This cannot arise on the tree this task lands on -- the port writes no dumps at all
    // today -- so the port-side directory below is populated as fixture data, by this test, to
    // hold the comparison to the right behaviour when it can arise. Nothing is run in the port's
    // place to produce it; see this file's own doc comment and the pull request body.
    let scratch = scratch_dir("port-dump-dir-incomplete");
    let oracle_dumps = subdir(&scratch, "oracle-dumps");
    let port_dumps = subdir(&scratch, "port-dumps");

    // All of `manifest.tsv`'s stages but `myNotPortedStage` derive `ported` (same
    // `ported_seam.rs` module), so all three need an oracle dump, each carrying its own
    // `; seam:` header so the `dump_seam_header` cross-check never fires here.
    for stage in ["myDumpStage", "myMissingDumpStage", "myNoPortDumpStage"] {
        write_dump(&oracle_dumps.join(format!("{stage}.dump")), stage, "0x1p0");
    }
    // The port side carries two of the three: `myNoPortDumpStage` is deliberately absent from a
    // directory that is not empty, which is what distinguishes this from the case above.
    for stage in ["myDumpStage", "myMissingDumpStage"] {
        write_dump(&port_dumps.join(format!("{stage}.dump")), stage, "0x1p0");
    }

    let err = stages::compare_stage_outputs(
        &fixture_tree().join("manifest.tsv"),
        &fixture_tree(),
        &oracle_dumps,
        &port_outputs(&port_dumps, &subdir(&scratch, "port-gcode")),
    )
    .expect_err("a fully-ported stage absent from a non-empty port dump dir must fail");
    assert!(
        err.contains("internal inconsistency"),
        "error should name it as an internal inconsistency, not silently pass: {err}"
    );
    assert!(
        err.contains("myNoPortDumpStage"),
        "error should name the stage the port wrote nothing for: {err}"
    );
    assert!(
        err.contains(NO_PORT_RUN),
        "the port run's own diagnostic must be carried into the error, not dropped: {err}"
    );
}

// --- An oracle dump no manifest row claims --------------------------------------------------

#[test]
fn uncovered_oracle_dump_is_printed_as_a_warning_not_silently_dropped() {
    // Four of R5's eleven seams (06-manifest-scope's own gap) have committed oracle dumps but no
    // manifest row at all. This command cannot fix the manifest (out of this task's scope), but
    // it must not stay silent about the gap either.
    let scratch = scratch_dir("uncovered-dump");
    let oracle_dumps = subdir(&scratch, "oracle-dumps");
    let port_dumps = subdir(&scratch, "port-dumps");
    write_dump(
        &oracle_dumps.join("myDumpStage.dump"),
        "myDumpStage",
        "0x1p0",
    );
    write_dump(&oracle_dumps.join("posIroning.dump"), "posIroning", "0x1p0");
    write_dump(&port_dumps.join("myDumpStage.dump"), "myDumpStage", "0x1p0");

    let report = stages::compare_stage_outputs(
        &fixture_tree().join("manifest_dump_only.tsv"),
        &fixture_tree(),
        &oracle_dumps,
        &port_outputs(&port_dumps, &subdir(&scratch, "port-gcode")),
    )
    .expect("an uncovered dump is a warning, not a failure");

    assert_eq!(
        report.uncovered_oracle_dumps,
        vec!["posIroning".to_string()]
    );
    assert!(
        stages::render_text(&report).contains("posIroning: no manifest row names this stage"),
        "{}",
        stages::render_text(&report)
    );
}

// --- `run`'s own half: spawning the port --------------------------------------------------

#[test]
fn run_fails_naming_the_port_binary_it_could_not_start() {
    // `run`'s only `Err` of its own (everything else is `compare_stage_outputs`'s): the real
    // `grampus-cli` could not be started at all. What the real binary does when it *can* be
    // started is proven against the real binary in `crates/grampus-cli/tests/dump_stages.rs`.
    let scratch = scratch_dir("port-bin-missing");
    let missing = scratch.join("no-such-grampus-cli");
    let err = stages::run(&stages::StagesConfig {
        manifest_path: &fixture_tree().join("manifest_dump_only.tsv"),
        tree: &fixture_tree(),
        oracle_dumps_dir: &subdir(&scratch, "oracle-dumps"),
        port_bin: &missing,
        fixture: &repo_root().join("fixtures/20mm_cube.stl"),
        config: &repo_root().join("port/goldens/configs/base-fff.json"),
        scratch_dir: &scratch,
    })
    .expect_err("an unstartable port binary must fail, naming itself");
    assert!(
        err.contains("cannot run port binary") && err.contains("no-such-grampus-cli"),
        "{err}"
    );
}
