//! `--dump-stages <dir>` (task `39-compare-stages`, R20): the flag this binary gained so
//! `crates/oracle-compare/src/stages.rs` can drive it as a subprocess. `tests/surface.rs`
//! already proves `Surface::process` itself is unaffected by a dump directory being set (its
//! five-calls test calls `process()` with no arguments either way, since the field lives on
//! `Surface`, not on that method's signature); this file proves the binary's own command-line
//! behaviour: the flag is accepted, and — honestly, on the tree this task lands on, where
//! nothing under `crates/slic3r-*/` is ported — running with it changes nothing observable,
//! because `build_model` still fails before any of the eleven seams in `surface.rs` are ever
//! reached. `Surface::process`'s own doc comment is where "the exported G-code is byte-identical
//! with and without the flag" actually needs to hold once a real implementation exists; this
//! test is the vacuous-but-real version of that claim available today (see this task's pull
//! request body for why it is vacuous).

use std::path::PathBuf;
use std::process::Command;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../")
}

fn scratch_dir(name: &str) -> PathBuf {
    let dir = repo_root()
        .join("oracle/.cache/tmp/dump-stages-test")
        .join(format!("{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("can create scratch dir");
    dir
}

/// Runs the compiled binary with `extra_args` appended after the fixed `<model> --config
/// <golden> --out <out_dir>` arguments, returning its exit status, stdout and stderr.
fn run_cli(out_dir: &std::path::Path, extra_args: &[&str]) -> std::process::Output {
    let bin = PathBuf::from(env!("CARGO_BIN_EXE_grampus-cli"));
    let model = repo_root().join("fixtures/20mm_cube.stl");
    let golden = repo_root().join("port/goldens/configs/base-fff.json");
    Command::new(bin)
        .arg(&model)
        .arg("--config")
        .arg(&golden)
        .arg("--out")
        .arg(out_dir)
        .args(extra_args)
        .output()
        .expect("grampus-cli runs")
}

#[test]
fn dump_stages_flag_is_accepted_and_writes_no_files_today() {
    let out_dir = scratch_dir("accepted");
    let dump_dir = out_dir.join("dump-stages");
    let output = run_cli(&out_dir, &["--dump-stages", dump_dir.to_str().unwrap()]);

    assert!(
        !output.status.success(),
        "build_model still reports not-yet-ported, so this exits non-zero"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("not yet ported: Surface::build_model"),
        "stderr: {stderr}"
    );

    // `dump_dir` itself is created (see `run_port_binary`'s own convention in `stages.rs`, which
    // this test does not call, and `main.rs`'s own flag handling, which only ever passes the
    // path through to `Surface::set_dump_stages_dir`), but no seam dump exists in it: every
    // `emit_stage_dump` call site is a documented no-op until a module behind that seam is
    // ported (see `surface.rs`).
    if dump_dir.is_dir() {
        let entries: Vec<_> = std::fs::read_dir(&dump_dir)
            .expect("dump dir is readable")
            .collect();
        assert!(
            entries.is_empty(),
            "expected no files under {dump_dir:?}, found {entries:?}"
        );
    }
}

#[test]
fn running_with_and_without_dump_stages_is_identical_today() {
    let without_dir = scratch_dir("without");
    let with_dir = scratch_dir("with");
    let dump_dir = with_dir.join("dump-stages");

    let without = run_cli(&without_dir, &[]);
    let with = run_cli(&with_dir, &["--dump-stages", dump_dir.to_str().unwrap()]);

    assert_eq!(without.status.code(), with.status.code());
    assert_eq!(without.stderr, with.stderr);
    assert_eq!(without.stdout, with.stdout);

    // Neither run ever reaches `export_gcode` (it fails at `build_model`, the first of the five
    // calls), so neither writes `plate_1.gcode` -- the claim "the port's G-code with
    // `--dump-stages` is byte-identical to its G-code without it" holds today only because
    // both sides are absent, not because a real comparison ran. See this task's pull request
    // body: this is the honestly-vacuous state of that proof until Phase B ports `export_gcode`.
    assert!(!without_dir.join("plate_1.gcode").is_file());
    assert!(!with_dir.join("plate_1.gcode").is_file());
}
