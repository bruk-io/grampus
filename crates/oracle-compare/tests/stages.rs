//! `just compare-stages` (task `39-compare-stages`, R20). Covers `stages.rs`'s dispatch end to
//! end against committed fragments and a committed fixture tree, plus the full `stages::run`
//! dispatch against a committed stand-in for `grampus-cli`
//! (`tests/data/stages/fake_port.sh`) for every branch, including the "matches"/"differs"
//! dump-compare path and the genuine port-side internal-inconsistency case — not only the
//! "not ported" and "missing committed artifact" cases this file originally covered against
//! `/usr/bin/true`, which could never drive those branches at all (`/usr/bin/true` writes
//! nothing, so `run()`'s dump-compare and fallback-compare code was reachable only through the
//! committed-fragment helpers below, never through `run()` itself — see this task's pull
//! request body). `fake_port.sh` writes nothing unless `StagesConfig::port_env` points it at a
//! source directory, so every test that does not opt in keeps today's "nothing is ported, port
//! writes nothing" behaviour.

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

/// Writes a minimal, committed-format stage dump at `path`, carrying `seam` in its own
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

    // Three lines, not one: a real (differing) ordinary stage, `psSkirtBrim`, ahead of the two
    // fallback stages — so the assertion below (neither fallback line names `psSkirtBrim`) is
    // capable of failing. With only the fallback line present (the previous shape of this test)
    // `!rendered.contains("psSkirtBrim")` was true for any possible `render_text`, since no
    // `psSkirtBrim` text could ever appear in a report that never mentioned it.
    let differing_dump = stages::compare_dump_records(
        &["field=print_z layer=0 region=- value=0x1p0".to_string()],
        &["field=print_z layer=0 region=- value=0x1p1".to_string()],
        &Tolerance::Exact,
    );
    assert!(matches!(differing_dump, DumpOutcome::Differ { .. }));

    let report = stages::StagesReport {
        lines: vec![
            stages::StageLine {
                stage: "psSkirtBrim".to_string(),
                result: StageResult::Dump {
                    tolerance: Tolerance::Exact,
                    outcome: differing_dump,
                },
            },
            stages::StageLine {
                stage: "psGCodeExport".to_string(),
                result: StageResult::Fallback {
                    tolerance: Tolerance::Exact,
                    outcome: stages::evaluate_fallback(&oracle, &port, Tolerance::Exact),
                },
            },
            stages::StageLine {
                stage: "psConflictCheck".to_string(),
                result: StageResult::Fallback {
                    tolerance: Tolerance::Exact,
                    outcome: stages::evaluate_fallback(&oracle, &port, Tolerance::Exact),
                },
            },
        ],
        first_differing_stage: Some("psSkirtBrim".to_string()),
        uncovered_oracle_dumps: Vec::new(),
    };
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

// --- The full dispatch: "not ported" and "missing dump" (fixture tree + a stand-in port binary)

fn run_dir(name: &str) -> PathBuf {
    scratch_dir(name)
}

/// The committed stand-in for `grampus-cli` (see this file's own doc comment): writes nothing
/// unless `port_env` (`StagesConfig::port_env`) points it, through `STAGE_DUMP_SOURCE`, at a
/// directory holding the `.dump`/`plate_1.gcode` files to copy.
fn stand_in_port_bin() -> PathBuf {
    data_dir().join("fake_port.sh")
}

#[test]
fn not_ported_stage_is_reported_without_touching_any_dump() {
    // A manifest holding only the one not-ported row -- not the shared `manifest.tsv`, whose
    // other rows derive `ported` and would need the stand-in port binary to have actually
    // written their dumps, which it does not unless `port_env` tells it to.
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
        port_env: &[],
    };

    let report = stages::run(&cfg)
        .expect("run succeeds: nothing fully ported but the one row this stage needs");
    let line = report
        .lines
        .iter()
        .find(|l| l.stage == "myNotPortedStage")
        .expect("myNotPortedStage has a line");
    assert!(matches!(line.result, StageResult::NotPorted));
    assert!(report.first_differing_stage.is_none());
}

#[test]
fn fully_ported_stage_with_no_committed_oracle_dump_fails_naming_the_fix() {
    let manifest = data_dir().join("fixture_tree/manifest.tsv");
    let tree = data_dir().join("fixture_tree");
    let scratch = run_dir("missing-oracle-dump");

    // `oracle_dumps_dir` here deliberately holds nothing at all, so `myDumpStage.dump` (the
    // first stage this run reaches) is absent.
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
        port_env: &[],
    };

    let err = stages::run(&cfg).expect_err("a fully-ported stage with no oracle dump must fail");
    assert!(
        err.contains("just oracle-dump 3DBenchy base-fff"),
        "error should name the fix: {err}"
    );
}

#[test]
fn run_routes_a_fully_ported_fallback_stage_to_the_gcode_path_not_the_dump_path() {
    // Proves `run()`'s own dispatch decision (`is_fallback_stage`), not just `evaluate_fallback`
    // in isolation: a manifest naming only `psGCodeExport`/`psConflictCheck`, both fully ported,
    // against an *empty* oracle-dumps directory. The dump-record branch would fail naming
    // `psGCodeExport.dump`; the fallback branch fails naming `plate_1.gcode` instead -- the
    // error text alone tells us which branch actually ran.
    let manifest = data_dir().join("fixture_tree/manifest_fallback_only.tsv");
    let tree = data_dir().join("fixture_tree");
    let scratch = run_dir("fallback-routing");
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
        port_env: &[],
    };

    let err = stages::run(&cfg)
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

#[test]
fn fully_ported_stage_whose_port_run_wrote_no_dump_is_an_internal_inconsistency_not_a_pass() {
    let manifest = data_dir().join("fixture_tree/manifest.tsv");
    let tree = data_dir().join("fixture_tree");
    let scratch = run_dir("no-port-dump");

    // All four of this manifest's stages but `myNotPortedStage` derive `ported` (same
    // `ported_seam.rs` module). The oracle side carries all three of the ported dump stages'
    // own committed dumps, each with a `; seam:` header matching its own stage (so the new
    // `dump_seam_header` cross-check never fires here) -- `write_dump` generates these rather
    // than reusing one file's bytes under three names, which would carry the wrong seam value
    // for two of the three.
    let dumps_dir = scratch.join("oracle-dumps");
    std::fs::create_dir_all(&dumps_dir).unwrap();
    write_dump(&dumps_dir.join("myDumpStage.dump"), "myDumpStage", "0x1p0");
    write_dump(
        &dumps_dir.join("myMissingDumpStage.dump"),
        "myMissingDumpStage",
        "0x1p0",
    );
    write_dump(
        &dumps_dir.join("myNoPortDumpStage.dump"),
        "myNoPortDumpStage",
        "0x1p0",
    );

    // The port side (driven through `fake_port.sh` + `port_env`) writes dumps for
    // `myDumpStage` and `myMissingDumpStage` but genuinely nothing for `myNoPortDumpStage` --
    // this is what makes the error this test asserts a *real* inconsistency (the port wrote
    // some stage dumps, just not this one) rather than the "the port writes no stage dumps at
    // all" structural-gap case `port_wrote_no_dumps_at_all` reports instead.
    let port_source_dir = scratch.join("port-dump-source");
    std::fs::create_dir_all(&port_source_dir).unwrap();
    write_dump(
        &port_source_dir.join("myDumpStage.dump"),
        "myDumpStage",
        "0x1p0",
    );
    write_dump(
        &port_source_dir.join("myMissingDumpStage.dump"),
        "myMissingDumpStage",
        "0x1p0",
    );
    let source_dir_str = port_source_dir
        .to_str()
        .expect("utf8 scratch path")
        .to_string();

    let cfg = stages::StagesConfig {
        manifest_path: &manifest,
        tree: &tree,
        oracle_dumps_dir: &dumps_dir,
        port_bin: &stand_in_port_bin(),
        fixture: &repo_root().join("fixtures/20mm_cube.stl"),
        config: &repo_root().join("port/goldens/configs/base-fff.json"),
        scratch_dir: &scratch,
        port_env: &[("STAGE_DUMP_SOURCE", &source_dir_str)],
    };

    let err = stages::run(&cfg)
        .expect_err("a fully-ported stage the port's own run produced no dump for must fail");
    assert!(
        err.contains("internal inconsistency"),
        "error should name it as an internal inconsistency, not silently pass: {err}"
    );
    assert!(
        err.contains("myNoPortDumpStage"),
        "error should name the stage the port wrote nothing for: {err}"
    );
    assert!(
        err.contains("port exited exit status: 0"),
        "fake_port.sh always exits 0; the error should carry that diagnostic tail: {err}"
    );
}

#[test]
fn run_reports_a_matching_dump_stage_through_the_real_dispatch() {
    // Exercises `run()`'s own `StageResult::Dump`/`DumpOutcome::Match` arm end to end, not just
    // `compare_dump_records` called directly -- the previous suite's stand-in (`/usr/bin/true`)
    // could never reach this branch at all (see this file's own doc comment).
    let manifest = data_dir().join("fixture_tree/manifest_dump_only.tsv");
    let tree = data_dir().join("fixture_tree");
    let scratch = run_dir("dump-match");

    let dumps_dir = scratch.join("oracle-dumps");
    std::fs::create_dir_all(&dumps_dir).unwrap();
    write_dump(&dumps_dir.join("myDumpStage.dump"), "myDumpStage", "0x1p0");

    let port_source_dir = scratch.join("port-dump-source");
    std::fs::create_dir_all(&port_source_dir).unwrap();
    write_dump(
        &port_source_dir.join("myDumpStage.dump"),
        "myDumpStage",
        "0x1p0",
    );
    let source_dir_str = port_source_dir
        .to_str()
        .expect("utf8 scratch path")
        .to_string();

    let cfg = stages::StagesConfig {
        manifest_path: &manifest,
        tree: &tree,
        oracle_dumps_dir: &dumps_dir,
        port_bin: &stand_in_port_bin(),
        fixture: &repo_root().join("fixtures/20mm_cube.stl"),
        config: &repo_root().join("port/goldens/configs/base-fff.json"),
        scratch_dir: &scratch,
        port_env: &[("STAGE_DUMP_SOURCE", &source_dir_str)],
    };

    let report = stages::run(&cfg).expect("a byte-identical dump pair must not error");
    assert_eq!(report.lines.len(), 1);
    match &report.lines[0].result {
        StageResult::Dump { outcome, .. } => assert_eq!(*outcome, DumpOutcome::Match),
        other => panic!("expected a Dump result, got {other:?}"),
    }
    assert!(report.first_differing_stage.is_none());
    let rendered = stages::render_text(&report);
    assert!(
        rendered.contains("myDumpStage: exact -- match"),
        "{rendered}"
    );
}

#[test]
fn run_names_the_first_differing_dump_stage_through_the_real_dispatch() {
    let manifest = data_dir().join("fixture_tree/manifest_dump_only.tsv");
    let tree = data_dir().join("fixture_tree");
    let scratch = run_dir("dump-differ");

    let dumps_dir = scratch.join("oracle-dumps");
    std::fs::create_dir_all(&dumps_dir).unwrap();
    write_dump(&dumps_dir.join("myDumpStage.dump"), "myDumpStage", "0x1p0");

    let port_source_dir = scratch.join("port-dump-source");
    std::fs::create_dir_all(&port_source_dir).unwrap();
    // A deliberately differing value -- 0x1p1 (2.0) against the oracle's 0x1p0 (1.0).
    write_dump(
        &port_source_dir.join("myDumpStage.dump"),
        "myDumpStage",
        "0x1p1",
    );
    let source_dir_str = port_source_dir
        .to_str()
        .expect("utf8 scratch path")
        .to_string();

    let cfg = stages::StagesConfig {
        manifest_path: &manifest,
        tree: &tree,
        oracle_dumps_dir: &dumps_dir,
        port_bin: &stand_in_port_bin(),
        fixture: &repo_root().join("fixtures/20mm_cube.stl"),
        config: &repo_root().join("port/goldens/configs/base-fff.json"),
        scratch_dir: &scratch,
        port_env: &[("STAGE_DUMP_SOURCE", &source_dir_str)],
    };

    let report = stages::run(&cfg).expect("a differing dump pair is reported, not an error");
    assert_eq!(report.first_differing_stage.as_deref(), Some("myDumpStage"));
    match &report.lines[0].result {
        StageResult::Dump {
            outcome: DumpOutcome::Differ { record_index, .. },
            ..
        } => assert_eq!(*record_index, 0),
        other => panic!("expected a differing Dump result, got {other:?}"),
    }
}

#[test]
fn uncovered_oracle_dump_is_printed_as_a_warning_not_silently_dropped() {
    // Four of R5's eleven seams (06-manifest-scope's own gap) have committed oracle dumps but
    // no manifest row at all. This command cannot fix the manifest (out of this task's scope),
    // but it must not stay silent about the gap either.
    let manifest = data_dir().join("fixture_tree/manifest_dump_only.tsv");
    let tree = data_dir().join("fixture_tree");
    let scratch = run_dir("uncovered-dump");

    let dumps_dir = scratch.join("oracle-dumps");
    std::fs::create_dir_all(&dumps_dir).unwrap();
    write_dump(&dumps_dir.join("myDumpStage.dump"), "myDumpStage", "0x1p0");
    write_dump(&dumps_dir.join("posIroning.dump"), "posIroning", "0x1p0");

    let port_source_dir = scratch.join("port-dump-source");
    std::fs::create_dir_all(&port_source_dir).unwrap();
    write_dump(
        &port_source_dir.join("myDumpStage.dump"),
        "myDumpStage",
        "0x1p0",
    );
    let source_dir_str = port_source_dir
        .to_str()
        .expect("utf8 scratch path")
        .to_string();

    let cfg = stages::StagesConfig {
        manifest_path: &manifest,
        tree: &tree,
        oracle_dumps_dir: &dumps_dir,
        port_bin: &stand_in_port_bin(),
        fixture: &repo_root().join("fixtures/20mm_cube.stl"),
        config: &repo_root().join("port/goldens/configs/base-fff.json"),
        scratch_dir: &scratch,
        port_env: &[("STAGE_DUMP_SOURCE", &source_dir_str)],
    };

    let report = stages::run(&cfg).expect("an uncovered dump is a warning, not a failure");
    assert_eq!(
        report.uncovered_oracle_dumps,
        vec!["posIroning".to_string()]
    );
    let rendered = stages::render_text(&report);
    assert!(
        rendered.contains("posIroning: no manifest row names this stage"),
        "{rendered}"
    );
}

// --- The CLI binary's own exit code (task 39's "Done when": a divergence must fail the run) --

fn oracle_compare_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_oracle-compare"))
}

#[test]
fn cli_stages_exits_non_zero_on_a_differing_stage_and_zero_when_nothing_differs() {
    let tree = data_dir().join("fixture_tree");
    let manifest = data_dir().join("fixture_tree/manifest_dump_only.tsv");
    let fixture = repo_root().join("fixtures/20mm_cube.stl");
    let config = repo_root().join("port/goldens/configs/base-fff.json");
    let port_bin = stand_in_port_bin();

    // A differing run: exit code must be non-zero.
    let scratch = run_dir("cli-exit-differ");
    let dumps_dir = scratch.join("oracle-dumps");
    std::fs::create_dir_all(&dumps_dir).unwrap();
    write_dump(&dumps_dir.join("myDumpStage.dump"), "myDumpStage", "0x1p0");
    let source_dir = scratch.join("port-dump-source");
    std::fs::create_dir_all(&source_dir).unwrap();
    write_dump(&source_dir.join("myDumpStage.dump"), "myDumpStage", "0x1p1");

    let differ_output = std::process::Command::new(oracle_compare_bin())
        .arg("--stages")
        .arg("--manifest")
        .arg(&manifest)
        .arg("--tree")
        .arg(&tree)
        .arg("--oracle-dumps")
        .arg(&dumps_dir)
        .arg("--port-bin")
        .arg(&port_bin)
        .arg("--fixture")
        .arg(&fixture)
        .arg("--config")
        .arg(&config)
        .arg("--scratch")
        .arg(scratch.join("run-scratch"))
        .env("STAGE_DUMP_SOURCE", &source_dir)
        .output()
        .expect("spawning oracle-compare --stages");
    assert!(
        !differ_output.status.success(),
        "a differing stage must exit non-zero: stdout={} stderr={}",
        String::from_utf8_lossy(&differ_output.stdout),
        String::from_utf8_lossy(&differ_output.stderr),
    );
    let stdout = String::from_utf8_lossy(&differ_output.stdout);
    assert!(
        stdout.contains("first differing stage: myDumpStage"),
        "{stdout}"
    );

    // A matching run on the same shape: exit code must be zero.
    let scratch2 = run_dir("cli-exit-match");
    let dumps_dir2 = scratch2.join("oracle-dumps");
    std::fs::create_dir_all(&dumps_dir2).unwrap();
    write_dump(&dumps_dir2.join("myDumpStage.dump"), "myDumpStage", "0x1p0");
    let source_dir2 = scratch2.join("port-dump-source");
    std::fs::create_dir_all(&source_dir2).unwrap();
    write_dump(
        &source_dir2.join("myDumpStage.dump"),
        "myDumpStage",
        "0x1p0",
    );

    let match_output = std::process::Command::new(oracle_compare_bin())
        .arg("--stages")
        .arg("--manifest")
        .arg(&manifest)
        .arg("--tree")
        .arg(&tree)
        .arg("--oracle-dumps")
        .arg(&dumps_dir2)
        .arg("--port-bin")
        .arg(&port_bin)
        .arg("--fixture")
        .arg(&fixture)
        .arg("--config")
        .arg(&config)
        .arg("--scratch")
        .arg(scratch2.join("run-scratch"))
        .env("STAGE_DUMP_SOURCE", &source_dir2)
        .output()
        .expect("spawning oracle-compare --stages");
    assert!(
        match_output.status.success(),
        "a matching run must exit zero: stdout={} stderr={}",
        String::from_utf8_lossy(&match_output.stdout),
        String::from_utf8_lossy(&match_output.stderr),
    );

    // A not-ported-only run: exit code must also be zero (task.md's own "Done when").
    let scratch3 = run_dir("cli-exit-not-ported");
    let not_ported_manifest = data_dir().join("fixture_tree/manifest_not_ported_only.tsv");
    let empty_dumps = scratch3.join("empty-oracle-dumps");
    std::fs::create_dir_all(&empty_dumps).unwrap();
    let not_ported_output = std::process::Command::new(oracle_compare_bin())
        .arg("--stages")
        .arg("--manifest")
        .arg(&not_ported_manifest)
        .arg("--tree")
        .arg(&tree)
        .arg("--oracle-dumps")
        .arg(&empty_dumps)
        .arg("--port-bin")
        .arg(&port_bin)
        .arg("--fixture")
        .arg(&fixture)
        .arg("--config")
        .arg(&config)
        .arg("--scratch")
        .arg(scratch3.join("run-scratch"))
        .output()
        .expect("spawning oracle-compare --stages");
    assert!(
        not_ported_output.status.success(),
        "a tree with nothing fully ported must exit zero: stdout={} stderr={}",
        String::from_utf8_lossy(&not_ported_output.stdout),
        String::from_utf8_lossy(&not_ported_output.stderr),
    );
}
