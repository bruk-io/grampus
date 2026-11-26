//! Proof for the `Utils.hpp`/`utils.cpp` manifest row (task 57-foundation-utils): `Slic3r::
//! get_utf8_sequence_length(const char*, size_t)` (`utils.cpp:1390-1452`), called through
//! `crates/slic3r-ffi/cxx/proof_units/utils.cpp`'s shim, reached through `slic3r-proof`'s
//! re-export of the FFI crate's per-unit proof bridges -- the one sanctioned route
//! (`docs/PORTING.md`, "Decisions recorded on 2026-11-25").
//!
//! Marked `// PROOF: 3` because the manifest row's `proof path` cell is `3`
//! (`port/port-status`'s `proof_present` matches that row's own cell verbatim); the shim itself
//! is shaped like proof path 1, not path 3 -- see `cxx/proof_units/utils.cpp`'s header comment
//! for the symbol-count measurement that rules the literal `#include` of `utils.cpp` out, and
//! this task's pull request body for the same note.
//!
//! Every input is non-empty: both sides read byte `0` unconditionally with no length check
//! (the vendored function has no guard either, `utils.cpp:1393`), so an empty input is not a
//! meaningful comparison.

use slic3r_proof::proof_units::utils::path1_get_utf8_sequence_length;
use slic3r_utils::utils::get_utf8_sequence_length;

/// `Slic3r::get_utf8_sequence_length` against this port's own [`get_utf8_sequence_length`],
/// across every leading-byte range the function branches on: plain ASCII, a lone continuation
/// byte, each valid multi-byte header length (2 through 6 bytes), a header byte whose declared
/// length runs past the buffer's end (in range and truncated), and the two `0xFE`/`0xFF`
/// "not a UTF-8 sequence" bytes.
// PROOF: 3
#[test]
fn utils_get_utf8_sequence_length_matches_the_oracle() {
    let cases: &[&[u8]] = &[
        b"a",
        b"hello world",
        &[0x80],
        &[0x80, 0x81, 0x82, b'x'],
        &[0xC2, 0xA9],
        &[0xC2],
        &[0xE2, 0x82, 0xAC],
        &[0xE2, 0x82],
        &[0xF0, 0x9F, 0x98, 0x80],
        &[0xF0, 0x9F],
        &[0xF8, 0x80, 0x80, 0x80, 0x80],
        &[0xF8, 0x80],
        &[0xFC, 0x80, 0x80, 0x80, 0x80, 0x80],
        &[0xFC],
        &[0xFE],
        &[0xFF],
        &[0xFE, 0x00],
    ];

    for seq in cases {
        let oracle_len = path1_get_utf8_sequence_length(seq.to_vec());
        let ported_len = get_utf8_sequence_length(seq) as u64;
        assert_eq!(oracle_len, ported_len, "mismatch for {seq:?}");
    }
}

// =============================================================================================
// The rest of this file is the same proof, differential against the same oracle, for the
// declarations the first revision of this unit left unported: the eight `Utils.hpp` time
// formatters, the three canonicalization-based path-root checks, and the file-mutation family.
// Every assertion compares a value this port computed against a value the oracle computed in
// the same run; nothing is compared against a literal transcribed by hand.
//
// The filesystem tests work in a scratch tree under `oracle/.cache/tmp/` (git-ignored), named
// with this process's id and a per-call counter so concurrent agents and concurrent test
// threads never share one. Each one builds two identical sibling subtrees and runs the oracle on
// `a/` and this port on `b/`, then compares both the returned result and the resulting tree.
// =============================================================================================

use slic3r_proof::proof_units::utils as oracle;
use slic3r_utils::utils as port;
use std::path::{Path, PathBuf};

static SCRATCH_COUNTER: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

/// A fresh, empty scratch directory inside the worktree's git-ignored cache.
fn scratch(label: &str) -> PathBuf {
    let n = SCRATCH_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../oracle/.cache/tmp")
        .join(format!(
            "utils-proof-{}-{}-{}",
            std::process::id(),
            n,
            label
        ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("scratch dir");
    dir
}

fn write(path: &Path, content: &[u8]) {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("parent");
    }
    std::fs::write(path, content).expect("write");
}

/// A directory tree rendered as sorted `relpath|kind|payload` lines, so two trees compare as one
/// string. A symlink's payload is its target, a file's is its bytes, a directory's is empty.
fn render_tree(root: &Path) -> String {
    let mut lines = Vec::new();
    fn walk(root: &Path, dir: &Path, lines: &mut Vec<String>) {
        let mut entries: Vec<PathBuf> = match std::fs::read_dir(dir) {
            Ok(rd) => rd.filter_map(Result::ok).map(|e| e.path()).collect(),
            Err(e) => {
                lines.push(format!("<unreadable {}: {}>", dir.display(), e.kind()));
                return;
            }
        };
        entries.sort();
        for entry in entries {
            let rel = entry.strip_prefix(root).unwrap_or(&entry).to_path_buf();
            let meta = match entry.symlink_metadata() {
                Ok(m) => m,
                Err(e) => {
                    lines.push(format!("{}|error|{}", rel.display(), e.kind()));
                    continue;
                }
            };
            if meta.file_type().is_symlink() {
                let target = std::fs::read_link(&entry).unwrap_or_default();
                lines.push(format!("{}|link|{}", rel.display(), target.display()));
            } else if meta.is_dir() {
                lines.push(format!("{}|dir|", rel.display()));
                walk(root, &entry, lines);
            } else {
                let bytes = std::fs::read(&entry).unwrap_or_default();
                lines.push(format!("{}|file|{:?}", rel.display(), bytes));
            }
        }
    }
    walk(root, root, &mut lines);
    lines.join("\n")
}

/// The oracle's `std::error_code::value()` next to this port's `io::Result`, as one integer each.
fn errno_of(result: &std::io::Result<()>) -> i32 {
    match result {
        Ok(()) => 0,
        Err(e) => e.raw_os_error().unwrap_or(-1),
    }
}

// ---------------------------------------------------------------------------------------------
// The Utils.hpp time formatters
// ---------------------------------------------------------------------------------------------

/// Every input used below. Chosen to cross each branch of each formatter: zero, sub-second,
/// sub-minute, each `d`/`h`/`m` threshold, a value whose seconds remainder rounds, and a value
/// large enough for a multi-digit day count.
const TIME_INPUTS: &[f32] = &[
    0.0, 0.5, 0.999, 1.0, 1.5, 2.0, 29.0, 30.0, 59.0, 59.5, 60.0, 61.0, 90.0, 599.0, 600.0, 3599.0,
    3600.0, 3601.0, 3661.5, 7200.0, 86399.0, 86400.0, 86401.0, 90061.0, 123456.75, 8640000.0,
];

#[test]
fn utils_get_time_dhms_matches_the_oracle() {
    for &t in TIME_INPUTS {
        assert_eq!(
            oracle::path1_get_time_dhms(t),
            port::get_time_dhms(t),
            "get_time_dhms({t})"
        );
    }
}

#[test]
fn utils_get_bbl_time_dhms_matches_the_oracle() {
    for &t in TIME_INPUTS {
        assert_eq!(
            oracle::path1_get_bbl_time_dhms(t),
            port::get_bbl_time_dhms(t),
            "get_bbl_time_dhms({t})"
        );
    }
}

#[test]
fn utils_get_time_hms_matches_the_oracle() {
    for &t in TIME_INPUTS {
        assert_eq!(
            oracle::path1_get_time_hms(t),
            port::get_time_hms(t),
            "get_time_hms({t})"
        );
    }
}

#[test]
fn utils_get_bbl_monitor_time_dhm_matches_the_oracle() {
    for &t in TIME_INPUTS {
        assert_eq!(
            oracle::path1_get_bbl_monitor_time_dhm(t),
            port::get_bbl_monitor_time_dhm(t),
            "get_bbl_monitor_time_dhm({t})"
        );
    }
}

#[test]
fn utils_get_bbl_remain_time_dhms_matches_the_oracle() {
    for &t in TIME_INPUTS {
        assert_eq!(
            oracle::path1_get_bbl_remain_time_dhms(t),
            port::get_bbl_remain_time_dhms(t),
            "get_bbl_remain_time_dhms({t})"
        );
    }
}

#[test]
fn utils_get_time_dhm_matches_the_oracle() {
    for &t in TIME_INPUTS {
        assert_eq!(
            oracle::path1_get_time_dhm(t),
            port::get_time_dhm(t),
            "get_time_dhm({t})"
        );
    }
}

#[test]
fn utils_get_timezone_utc_hm_matches_the_oracle() {
    let seconds: &[i64] = &[
        0, 1, 59, 60, 1800, 3600, 3660, -3600, -3660, -1800, 19800, -12600, 50400, -39600, 86399,
        -86399,
    ];
    for &s in seconds {
        assert_eq!(
            oracle::path1_get_timezone_utc_hm(s),
            port::get_timezone_utc_hm(s),
            "get_timezone_utc_hm({s})"
        );
    }
}

/// `short_time` over the four format shapes it dispatches on, each complete and each truncated
/// part-way, plus the `sscanf`-failure shapes (no digits, a literal that does not match) and the
/// minute/hour/day rollovers at `Utils.hpp:593-600`. Every input leaves `seconds >= 0`, which is
/// the range `Utils.hpp:618` writes a buffer for.
#[test]
fn utils_short_time_matches_the_oracle() {
    let inputs: &[&str] = &[
        "1d 2h 3m 4s",
        "1d 2h 3m 30s",
        "1d 2h 59m 30s",
        "1d 23h 59m 30s",
        "2d",
        "2d 5h",
        "3h 4m 5s",
        "3h 59m 30s",
        "0h 0m 0s",
        "5m 29s",
        "5m 30s",
        "5m",
        "45s",
        "0.500000s",
        "0.999999s",
        "1.000000s",
        "12.500000s",
        "s",
        "d",
        "m",
        "h",
        "xd",
        "",
        "   7d 1h 2m 3s",
        "7d1h2m3s",
        "7d  1h   2m    3s",
        "+7d 1h 2m 3s",
        "1e3s",
    ];
    for input in inputs {
        assert_eq!(
            oracle::path1_short_time(input.to_string()),
            port::short_time(input),
            "short_time({input:?})"
        );
    }
}

// ---------------------------------------------------------------------------------------------
// The three canonicalization-based path-root checks
// ---------------------------------------------------------------------------------------------

/// The scratch tree every path-root test below shares: a `root/` with a file, a subdirectory and
/// a symlink pointing out of it, beside an `outside/` the symlink reaches.
fn path_root_tree() -> PathBuf {
    let base = scratch("within-root");
    write(&base.join("root/inside.txt"), b"in");
    write(&base.join("root/sub/deeper.txt"), b"deep");
    write(&base.join("outside/secret.txt"), b"out");
    std::os::unix::fs::symlink("../outside", base.join("root/link_out")).expect("symlink");
    std::os::unix::fs::symlink("sub", base.join("root/link_in")).expect("symlink");
    base
}

#[test]
fn utils_is_path_within_root_matches_the_oracle() {
    let base = path_root_tree();
    let root = base.join("root");
    let root_as_string = root.to_str().expect("utf8 scratch path").to_string();
    let root_with_slash = format!("{root_as_string}/");

    let cases: &[&str] = &[
        "inside.txt",
        "sub",
        "sub/deeper.txt",
        "missing.txt",
        "missing/deeper.txt",
        "",
        "/inside.txt",
        "\\inside.txt",
        "c:inside.txt",
        "..",
        "../outside/secret.txt",
        "sub/../inside.txt",
        "sub/..",
        "link_out",
        "link_out/secret.txt",
        "link_in/deeper.txt",
        "a\0b",
        "inside.txt/",
        "./inside.txt",
        "sub\\deeper.txt",
    ];
    for root_spelling in [&root_as_string, &root_with_slash] {
        for rel in cases {
            assert_eq!(
                oracle::path1_is_path_within_root(rel.to_string(), root_spelling.clone()),
                port::is_path_within_root(rel, Path::new(root_spelling)),
                "is_path_within_root({rel:?}, {root_spelling:?})"
            );
        }
    }
}

#[test]
fn utils_is_symlink_target_within_root_matches_the_oracle() {
    let base = path_root_tree();
    let root = base.join("root");
    let root_as_string = root.to_str().expect("utf8 scratch path").to_string();

    let cases: &[(&str, &str)] = &[
        ("sub/link", "deeper.txt"),
        ("sub/link", "../inside.txt"),
        ("sub/link", ".."),
        ("link", "inside.txt"),
        ("link", "sub/deeper.txt"),
        ("link", ""),
        ("link", "/etc/passwd"),
        ("link", "\\etc"),
        ("link", "c:x"),
        ("sub\\link", "deeper.txt"),
        ("link", "missing.txt"),
        ("link", "link_out/secret.txt"),
    ];
    for (link_rel, target) in cases {
        assert_eq!(
            oracle::path1_is_symlink_target_within_root(
                link_rel.to_string(),
                target.to_string(),
                root_as_string.clone()
            ),
            port::is_symlink_target_within_root(link_rel, target, Path::new(&root_as_string)),
            "is_symlink_target_within_root({link_rel:?}, {target:?})"
        );
    }
}

#[test]
fn utils_is_absolute_path_within_root_matches_the_oracle() {
    let base = path_root_tree();
    let root = base.join("root");
    let root_as_string = root.to_str().expect("utf8 scratch path").to_string();

    let cases: Vec<String> = vec![
        root.join("inside.txt").to_string_lossy().to_string(),
        root.join("sub").to_string_lossy().to_string(),
        root.join("sub/deeper.txt").to_string_lossy().to_string(),
        root.join("missing.txt").to_string_lossy().to_string(),
        root.join("link_out").to_string_lossy().to_string(),
        root.join("link_out/secret.txt")
            .to_string_lossy()
            .to_string(),
        root_as_string.clone(),
        base.join("outside/secret.txt")
            .to_string_lossy()
            .to_string(),
        base.to_string_lossy().to_string(),
    ];
    for path in &cases {
        assert_eq!(
            oracle::path1_is_absolute_path_within_root(path.clone(), root_as_string.clone()),
            port::is_absolute_path_within_root(Path::new(path), Path::new(&root_as_string)),
            "is_absolute_path_within_root({path:?})"
        );
    }
}

// ---------------------------------------------------------------------------------------------
// The file-mutation family
// ---------------------------------------------------------------------------------------------

#[test]
fn utils_makedir_matches_the_oracle() {
    let base = scratch("makedir");
    let existing = base.join("there");
    std::fs::create_dir_all(&existing).expect("mkdir");
    for candidate in [existing, base.join("missing"), base.join("a/b/c")] {
        let as_string = candidate.to_string_lossy().to_string();
        assert_eq!(
            oracle::path1_makedir(as_string.clone()),
            port::makedir(&candidate),
            "makedir({as_string:?})"
        );
        // Neither side creates anything on this platform (the vendored `#else` branch is empty).
        assert!(!candidate.join("x").exists());
    }
}

#[test]
fn utils_is_running_in_flatpak_matches_the_oracle() {
    assert_eq!(
        oracle::path1_is_running_in_flatpak(),
        port::is_running_in_flatpak()
    );
}

#[test]
fn utils_rename_file_matches_the_oracle() {
    let base = scratch("rename");
    // Four shapes: a plain rename, a rename over an existing file, a missing source, and a
    // target whose parent directory does not exist.
    let cases: &[(&str, &str, bool, bool)] = &[
        ("from.txt", "to.txt", true, false),
        ("from.txt", "to.txt", true, true),
        ("missing.txt", "to.txt", false, false),
        ("from.txt", "nodir/to.txt", true, false),
    ];
    for (i, (from, to, create_from, create_to)) in cases.iter().enumerate() {
        let mut errnos = Vec::new();
        let mut trees = Vec::new();
        for side in ["a", "b"] {
            let dir = base.join(format!("{i}/{side}"));
            std::fs::create_dir_all(&dir).expect("case dir");
            if *create_from {
                write(&dir.join(from), b"payload");
            }
            if *create_to {
                write(&dir.join(to), b"old");
            }
            let from_path = dir.join(from);
            let to_path = dir.join(to);
            let code = if side == "a" {
                oracle::path1_rename_file(
                    from_path.to_string_lossy().to_string(),
                    to_path.to_string_lossy().to_string(),
                )
            } else {
                errno_of(&port::rename_file(&from_path, &to_path))
            };
            errnos.push(code);
            trees.push(render_tree(&dir));
        }
        assert_eq!(errnos[0], errnos[1], "rename_file({from:?}, {to:?}) errno");
        assert_eq!(trees[0], trees[1], "rename_file({from:?}, {to:?}) tree");
    }
}

#[test]
fn utils_write_file_atomically_matches_the_oracle() {
    let base = scratch("atomic-write");
    // A new file, an overwrite, a write through a symlink to a regular file, a write to a
    // directory that does not exist, and a target that is itself a directory.
    let cases: &[(&str, &str)] = &[
        ("new.txt", "fresh content"),
        ("existing.txt", "replacement"),
        ("link.txt", "through the link"),
        ("nodir/new.txt", "no parent"),
        ("adir", "onto a directory"),
    ];
    for (i, (name, content)) in cases.iter().enumerate() {
        let mut errnos = Vec::new();
        let mut trees = Vec::new();
        for side in ["a", "b"] {
            let dir = base.join(format!("{i}/{side}"));
            std::fs::create_dir_all(&dir).expect("case dir");
            write(&dir.join("existing.txt"), b"old");
            write(&dir.join("target-of-link.txt"), b"old-target");
            std::os::unix::fs::symlink("target-of-link.txt", dir.join("link.txt"))
                .expect("symlink");
            std::fs::create_dir_all(dir.join("adir")).expect("adir");
            let path = dir.join(name);
            let code = if side == "a" {
                oracle::path1_write_file_atomically(
                    path.to_string_lossy().to_string(),
                    content.to_string(),
                    false,
                )
            } else {
                errno_of(&port::write_file_atomically_content(&path, content, false))
            };
            errnos.push(code);
            // The temporary's name carries a pid and a counter, so it is excluded from the tree
            // comparison; both sides remove it on every path that leaves one behind.
            trees.push(
                render_tree(&dir)
                    .lines()
                    .filter(|l| !l.contains(".tmp"))
                    .collect::<Vec<_>>()
                    .join("\n"),
            );
        }
        assert_eq!(
            errnos[0], errnos[1],
            "write_file_atomically({name:?}) errno"
        );
        assert_eq!(trees[0], trees[1], "write_file_atomically({name:?}) tree");
    }
}

#[test]
fn utils_check_copy_matches_the_oracle() {
    let base = scratch("check-copy");
    // Identical, different length, same length different bytes, both empty, a missing origin, a
    // missing target, and a pair exactly at the 8 MiB buffer boundary the vendored loop uses.
    let boundary = 8 * 1024 * 1024;
    let cases: Vec<(&str, Option<Vec<u8>>, Option<Vec<u8>>)> = vec![
        (
            "identical",
            Some(b"same bytes".to_vec()),
            Some(b"same bytes".to_vec()),
        ),
        ("shorter", Some(b"abcdef".to_vec()), Some(b"abc".to_vec())),
        (
            "same-length",
            Some(b"abcdef".to_vec()),
            Some(b"abcdeg".to_vec()),
        ),
        ("both-empty", Some(Vec::new()), Some(Vec::new())),
        ("no-origin", None, Some(b"x".to_vec())),
        ("no-target", Some(b"x".to_vec()), None),
        (
            "boundary",
            Some(vec![7u8; boundary]),
            Some(vec![7u8; boundary]),
        ),
        (
            "boundary-plus-one",
            Some(vec![7u8; boundary + 1]),
            Some(vec![7u8; boundary + 1]),
        ),
    ];
    for (label, origin, copy) in &cases {
        let dir = base.join(label);
        std::fs::create_dir_all(&dir).expect("case dir");
        let origin_path = dir.join("origin.bin");
        let copy_path = dir.join("copy.bin");
        if let Some(bytes) = origin {
            write(&origin_path, bytes);
        }
        if let Some(bytes) = copy {
            write(&copy_path, bytes);
        }
        let oracle_code = oracle::path1_check_copy(
            origin_path.to_string_lossy().to_string(),
            copy_path.to_string_lossy().to_string(),
        );
        let ported_code = port::check_copy(&origin_path, &copy_path) as i32;
        assert_eq!(oracle_code, ported_code, "check_copy({label})");
    }
}

#[test]
fn utils_copy_file_inner_matches_the_oracle() {
    let base = scratch("copy-file-inner");
    let cases: &[(&str, &str, bool, bool)] = &[
        ("plain", "from.txt", false, false),
        ("over-existing", "from.txt", true, false),
        ("over-read-only", "from.txt", true, true),
        ("missing-source", "missing.txt", false, false),
        ("no-target-dir", "from.txt", false, false),
    ];
    for (label, from, create_target, read_only_target) in cases {
        let mut codes = Vec::new();
        let mut trees = Vec::new();
        for side in ["a", "b"] {
            let dir = base.join(format!("{label}/{side}"));
            std::fs::create_dir_all(&dir).expect("case dir");
            write(&dir.join("from.txt"), b"source payload");
            let to = if *label == "no-target-dir" {
                dir.join("nodir/to.txt")
            } else {
                dir.join("to.txt")
            };
            if *create_target {
                write(&to, b"older and longer payload");
                if *read_only_target {
                    use std::os::unix::fs::PermissionsExt;
                    std::fs::set_permissions(&to, std::fs::Permissions::from_mode(0o444))
                        .expect("chmod");
                }
            }
            let from_path = dir.join(from);
            let code = if side == "a" {
                oracle::path1_copy_file_inner(
                    from_path.to_string_lossy().to_string(),
                    to.to_string_lossy().to_string(),
                )
            } else {
                port::copy_file_inner(&from_path, &to).0 as i32
            };
            codes.push(code);
            trees.push(render_tree(&dir));
        }
        assert_eq!(codes[0], codes[1], "copy_file_inner({label}) result");
        assert_eq!(trees[0], trees[1], "copy_file_inner({label}) tree");
    }
}

#[test]
fn utils_copy_file_matches_the_oracle() {
    let base = scratch("copy-file");
    let cases: &[(&str, &str, bool)] = &[
        ("plain", "from.txt", false),
        ("plain-checked", "from.txt", true),
        ("missing-source", "missing.txt", false),
        ("missing-source-checked", "missing.txt", true),
        ("no-target-dir", "from.txt", false),
    ];
    for (label, from, with_check) in cases {
        let mut codes = Vec::new();
        let mut trees = Vec::new();
        for side in ["a", "b"] {
            let dir = base.join(format!("{label}/{side}"));
            std::fs::create_dir_all(&dir).expect("case dir");
            write(&dir.join("from.txt"), b"source payload");
            let to = if *label == "no-target-dir" {
                dir.join("nodir/to.txt")
            } else {
                dir.join("to.txt")
            };
            let from_path = dir.join(from);
            let code = if side == "a" {
                oracle::path1_copy_file(
                    from_path.to_string_lossy().to_string(),
                    to.to_string_lossy().to_string(),
                    *with_check,
                )
            } else {
                port::copy_file(&from_path, &to, *with_check).0 as i32
            };
            codes.push(code);
            trees.push(render_tree(&dir));
        }
        assert_eq!(codes[0], codes[1], "copy_file({label}) result");
        assert_eq!(trees[0], trees[1], "copy_file({label}) tree");
    }
}

/// The tree both directory-copying tests build as their source.
fn tree_to_copy(dir: &Path) {
    write(&dir.join("source/top.txt"), b"top");
    write(&dir.join("source/nested/one.txt"), b"one");
    write(&dir.join("source/nested/deeper/two.txt"), b"two");
    write(&dir.join("source/skip-me.log"), b"log");
    std::os::unix::fs::symlink("top.txt", dir.join("source/link.txt")).expect("symlink");
}

#[test]
fn utils_copy_framework_matches_the_oracle() {
    let base = scratch("copy-framework");
    for (label, make_source) in [("tree", true), ("source-is-a-file", false)] {
        let mut answers = Vec::new();
        let mut trees = Vec::new();
        for side in ["a", "b"] {
            let dir = base.join(format!("{label}/{side}"));
            std::fs::create_dir_all(&dir).expect("case dir");
            if make_source {
                tree_to_copy(&dir);
            } else {
                write(&dir.join("source"), b"not a directory");
            }
            let source = dir.join("source");
            let target = dir.join("target");
            let answer = if side == "a" {
                oracle::path1_copy_framework(
                    source.to_string_lossy().to_string(),
                    target.to_string_lossy().to_string(),
                )
            } else {
                port::copy_framework(&source, &target)
            };
            answers.push(answer);
            trees.push(render_tree(&dir));
        }
        assert_eq!(answers[0], answers[1], "copy_framework({label}) answer");
        assert_eq!(trees[0], trees[1], "copy_framework({label}) tree");
    }
}

#[test]
fn utils_copy_directory_recursively_matches_the_oracle() {
    let base = scratch("copy-dir");
    // merge_mode off with an existing target (the target is removed first), merge_mode on with
    // an existing target (it is kept), and a source that does not exist.
    let cases: &[(&str, bool, bool, bool)] = &[
        ("fresh", false, false, true),
        ("replace", false, true, true),
        ("merge", true, true, true),
        ("missing-source", false, false, false),
    ];
    for (label, merge_mode, pre_existing_target, make_source) in cases {
        let mut answers = Vec::new();
        let mut trees = Vec::new();
        for side in ["a", "b"] {
            let dir = base.join(format!("{label}/{side}"));
            std::fs::create_dir_all(&dir).expect("case dir");
            if *make_source {
                tree_to_copy(&dir);
            }
            if *pre_existing_target {
                write(&dir.join("target/already-here.txt"), b"kept or removed");
            }
            let source = dir.join("source");
            let target = dir.join("target");
            let answer = if side == "a" {
                oracle::path1_copy_directory_recursively(
                    source.to_string_lossy().to_string(),
                    target.to_string_lossy().to_string(),
                    *merge_mode,
                )
            } else {
                port::copy_directory_recursively(&source, &target, None, *merge_mode).is_ok()
            };
            answers.push(answer);
            trees.push(
                render_tree(&dir)
                    .lines()
                    .filter(|l| !l.contains(".tmp"))
                    .collect::<Vec<_>>()
                    .join("\n"),
            );
        }
        assert_eq!(
            answers[0], answers[1],
            "copy_directory_recursively({label}) answer"
        );
        assert_eq!(
            trees[0], trees[1],
            "copy_directory_recursively({label}) tree"
        );
    }
}

#[test]
fn utils_save_and_load_string_file_match_the_oracle() {
    let base = scratch("string-file");
    let payloads: &[&[u8]] = &[
        b"",
        b"plain text",
        b"with\na newline\r\n",
        &[0x00, 0x01, 0xfe],
    ];
    for (i, payload) in payloads.iter().enumerate() {
        // `save_string_file` is compared by the bytes it leaves behind as well as by its answer.
        let oracle_path = base.join(format!("{i}-a.bin"));
        let ported_path = base.join(format!("{i}-b.bin"));
        let oracle_saved = oracle::path1_save_string_file(
            oracle_path.to_string_lossy().to_string(),
            payload.to_vec(),
        );
        let ported_saved = port::save_string_file(&ported_path, payload).is_ok();
        assert_eq!(oracle_saved, ported_saved, "save_string_file({i}) answer");

        // Read both sides' own file back through both implementations, so the comparison does
        // not depend on the two having written the same bytes.
        for path in [&oracle_path, &ported_path] {
            let mut oracle_out: Vec<u8> = Vec::new();
            let oracle_loaded =
                oracle::path1_load_string_file(path.to_string_lossy().to_string(), &mut oracle_out);
            let ported_loaded = port::load_string_file(path);
            assert_eq!(
                oracle_loaded,
                ported_loaded.is_ok(),
                "load_string_file({path:?}) answer"
            );
            if oracle_loaded {
                assert_eq!(
                    oracle_out.as_slice(),
                    ported_loaded.expect("loaded").as_slice(),
                    "load_string_file({path:?}) bytes"
                );
            }
        }
    }

    // A path that is not there, and a path that is a directory.
    let missing = base.join("missing.bin");
    let mut sink: Vec<u8> = Vec::new();
    assert_eq!(
        oracle::path1_load_string_file(missing.to_string_lossy().to_string(), &mut sink),
        port::load_string_file(&missing).is_ok(),
        "load_string_file on a missing path"
    );
    let a_dir = base.join("a-dir");
    std::fs::create_dir_all(&a_dir).expect("mkdir");
    assert_eq!(
        oracle::path1_save_string_file(a_dir.to_string_lossy().to_string(), b"x".to_vec()),
        port::save_string_file(&a_dir, b"x").is_ok(),
        "save_string_file onto a directory"
    );
}
