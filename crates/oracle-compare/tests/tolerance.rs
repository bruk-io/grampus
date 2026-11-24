//! Exercises `--tolerance-from-config` against both a committed fixture table (the shape
//! `docs/ORACLE.md`'s configuration table will have once `04-oracle-slice` adds it) and the real
//! `docs/ORACLE.md`, which has no `tolerance` or `comparison function` column yet — task
//! `09-comparator-gcode-levels` is held to the fixture meanwhile, and the real document must
//! fail with the named missing-column error rather than compare at a silent default (its own
//! "Done when" line). Also exercises the `oracle-compare` binary directly for the pieces of R4
//! that are about its command-line behaviour: the tolerance and its source are printed on every
//! run, and a bad tolerance cell exits non-zero having compared nothing.

use oracle_compare::tolerance;

fn data_dir() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/data")
}

/// The repository root, three levels up from this crate's manifest dir
/// (`crates/oracle-compare` -> `crates` -> repo root).
fn repo_root() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repo root exists")
}

#[test]
fn config_fixture_row_with_reason_resolves() {
    let path = data_dir().join("config/tolerance.md");
    let tol = tolerance::from_config_row(path.to_str().unwrap(), "basic_pla")
        .expect("basic_pla has a valid tolerance cell");
    assert!(tol.to_string().contains("reason"));
    assert!((tol.value() - 0.01).abs() < 1e-9);
}

#[test]
fn config_fixture_row_with_no_reason_is_refused_and_names_the_row() {
    let path = data_dir().join("config/tolerance.md");
    let err = tolerance::from_config_row(path.to_str().unwrap(), "no_reason")
        .expect_err("a tolerance value with no reason must be refused");
    assert!(
        err.contains("no_reason"),
        "error should name the row: {err}"
    );
}

#[test]
fn config_fixture_missing_configuration_is_refused_and_names_it() {
    let path = data_dir().join("config/tolerance.md");
    let err = tolerance::from_config_row(path.to_str().unwrap(), "no_such_configuration")
        .expect_err("no row has this configuration");
    assert!(
        err.contains("no_such_configuration"),
        "error should name it: {err}"
    );
}

#[test]
fn real_oracle_doc_has_no_tolerance_column_yet() {
    // The honest, committed proof that this task does not silently compare at a default tolerance
    // against the real document: `docs/ORACLE.md` has no configuration table with `tolerance` and
    // `comparison function` columns until `04-oracle-slice` lands one, so this must fail named,
    // not default to exact.
    let path = repo_root().join("docs/ORACLE.md");
    assert!(path.is_file(), "expected {} to exist", path.display());
    let err = tolerance::from_config_row(path.to_str().unwrap(), "any")
        .expect_err("docs/ORACLE.md has no tolerance/comparison function column yet");
    assert!(
        err.contains("tolerance") && err.contains("comparison function"),
        "error should name the missing columns: {err}"
    );
}

fn binary_path() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_BIN_EXE_oracle-compare"))
}

fn run(args: &[&str]) -> (bool, String, String) {
    let output = std::process::Command::new(binary_path())
        .args(args)
        .output()
        .expect("spawning oracle-compare");
    (
        output.status.success(),
        String::from_utf8_lossy(&output.stdout).into_owned(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

#[test]
fn cli_prints_exact_tolerance_with_no_flag() {
    let a = data_dir().join("identical/a.gcode");
    let b = data_dir().join("identical/b.gcode");
    let (ok, stdout, _) = run(&[a.to_str().unwrap(), b.to_str().unwrap()]);
    assert!(ok);
    assert!(stdout.contains("tolerance: exact\n"), "{stdout}");
}

#[test]
fn cli_prints_the_declared_tolerance_and_its_source() {
    let a = data_dir().join("small_drift/a.gcode");
    let b = data_dir().join("small_drift/b.gcode");
    let manifest = data_dir().join("manifest/tolerance.tsv");
    let (ok, stdout, _) = run(&[
        a.to_str().unwrap(),
        b.to_str().unwrap(),
        "--tolerance-from-manifest",
        manifest.to_str().unwrap(),
        "--row",
        "Foo/Bar.cpp",
    ]);
    assert!(ok);
    assert!(stdout.contains("Foo/Bar.cpp"), "{stdout}");
    assert!(stdout.contains("reason"), "{stdout}");
    assert!(stdout.contains("L3: pass"), "{stdout}");
}

#[test]
fn cli_exits_non_zero_and_names_the_row_with_no_reason() {
    let a = data_dir().join("small_drift/a.gcode");
    let b = data_dir().join("small_drift/b.gcode");
    let manifest = data_dir().join("manifest/tolerance.tsv");
    let (ok, stdout, stderr) = run(&[
        a.to_str().unwrap(),
        b.to_str().unwrap(),
        "--tolerance-from-manifest",
        manifest.to_str().unwrap(),
        "--row",
        "Foo/Baz.cpp",
    ]);
    assert!(!ok, "a tolerance cell with no reason must exit non-zero");
    assert!(
        stdout.is_empty(),
        "nothing should be compared: stdout was {stdout}"
    );
    assert!(stderr.contains("Foo/Baz.cpp"), "{stderr}");
}
