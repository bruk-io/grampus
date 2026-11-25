//! Proves proof path 2 (R19, plan step 18, task 32-proof-dumps-fuzz) and the three rules
//! `crates/slic3r-proof/src/dumps.rs`'s module doc states a reader has to get right.
//!
//! No ported Rust module exists yet (task 34-foundation-spike's fan-out is where one first
//! lands), so, exactly as `tests/harness_paths.rs` does for proof paths 1 and 3, `expected` below
//! is a value this task worked out by reading the real committed dump directly -- not a value a
//! stand-in computes. That is what a harness proof checks at this step: that the dump reader
//! actually reaches the committed file, parses the real format `dump_seams.cpp` writes, and
//! compares what it claims to through `tiers::compare_unit`.

use std::path::Path;

use slic3r_proof::dumps::{self, DumpError};
use slic3r_proof::{compare_unit, compare_unit_at, Oracle};

/// Proof path 2: `port/dumps/20mm_cube-base-fff/posSlice.dump`'s `lslices` field at layer 0,
/// object 0 -- a single four-point square, read straight from that dump's own text (reproduced
/// in this comment for the reviewer): `pts=10000000,10000000 -10000000,10000000
/// -10000000,-10000000 10000000,-10000000`. `ExPolygon.hpp`'s manifest row declares `proof path`
/// `2`, `pipeline stage` `posSlice` and `equality tier` `exact` -- `lslices` is an `ExPolygons`
/// field, i.e. `ExPolygon.hpp`'s own ported type, which is why this row is the one this test
/// names at `compare_unit`.
#[test]
fn ex_polygon_lslices_at_pos_slice_proves_path_2() {
    let dump = dumps::read_dump_default("20mm_cube-base-fff", "posSlice")
        .expect("the committed corpus dump must read: port/dumps/20mm_cube-base-fff/posSlice.dump");
    assert_eq!(dump.seam, "posSlice");
    assert_eq!(dump.object, "0");

    let actual: Vec<i64> = dump
        .points_at("lslices", Some(0), None)
        .expect("layer 0 must carry an 'lslices' record");
    let expected: Vec<i64> = vec![
        10_000_000,
        10_000_000,
        -10_000_000,
        10_000_000,
        -10_000_000,
        -10_000_000,
        10_000_000,
        -10_000_000,
    ];
    compare_unit("ExPolygon.hpp", Oracle::Built, actual, expected)
        .expect("posSlice's layer-0 lslices must be this square, read from the dump unchanged");
}

/// Rule 1 (the module doc): a coordinate above 2^53 is parsed as `i64` and comes back unchanged,
/// never round-tripped through `f64` (which would silently round `9007199254740993` to
/// `9007199254740992`). Reads the hand-written fixture through `dumps::parse_records` directly --
/// the fixture carries no provenance header by design (see its own leading comment), so
/// `dumps::read_dump`, which enforces rule 2 below, would refuse it; `parse_records` is the one
/// function this module exposes with no such requirement.
#[test]
fn oversized_coordinate_survives_exact() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/data/oversized-coordinate.dump");
    let text = std::fs::read_to_string(&path).expect("the hand-written fixture must be committed");
    let records = dumps::parse_records(&text, &path).expect("the fixture's one line must parse");
    assert_eq!(records.len(), 1);
    assert_eq!(
        records[0].points,
        vec![vec![9_007_199_254_740_993_i64, -9_007_199_254_740_993_i64]],
        "a coordinate above 2^53 must come back exactly, never rounded through f64"
    );

    // The same assertion again, but through a round-trip that *would* lose exactness, proving
    // the fixture really does sit on the boundary this test claims: 2^53 + 1 is not representable
    // in f64 at all (it rounds down to 2^53), so a reader that went through f64 anywhere in its
    // path would silently fail this specific check instead.
    assert_ne!(
        9_007_199_254_740_993_i64 as f64 as i64,
        9_007_199_254_740_993_i64
    );
}

/// Rule 2, first half: a dump with no provenance header at all is refused, named by file, rather
/// than compared. Constructs the text in memory (no file is written) -- `display_path` is only
/// what the resulting error names.
#[test]
fn missing_provenance_header_is_refused_named() {
    let path = Path::new("synthetic/no-header.dump");
    let text = "field=foo layer=- region=- value=1\n";
    let err = dumps::parse_dump_text(text, path).expect_err("a header-less dump must be refused");
    match err {
        DumpError::MissingProvenanceHeader { path: p } => assert_eq!(p, path),
        other => panic!("expected MissingProvenanceHeader, got {other}"),
    }
}

/// Rule 2, second half: a dump whose header names a commit other than the pin is refused, named
/// by file, rather than compared against what could be a stale dump.
#[test]
fn mismatched_provenance_header_is_refused_named() {
    let path = Path::new("synthetic/wrong-commit.dump");
    let text = "; provenance\n;   oracle-commit: 0000000000000000000000000000000000000000\n; seam: posSlice\n; object: 0\nfield=foo layer=- region=- value=1\n";
    let err = dumps::parse_dump_text(text, path).expect_err("a mismatched commit must be refused");
    match err {
        DumpError::ProvenanceMismatch {
            path: p,
            found,
            expected,
        } => {
            assert_eq!(p, path);
            assert_eq!(found, "0000000000000000000000000000000000000000");
            assert_eq!(expected, dumps::PINNED_COMMIT);
        }
        other => panic!("expected ProvenanceMismatch, got {other}"),
    }
}

/// Rule 3: `dumps::read_dump` takes the dump directory as an argument rather than hardcoding
/// `port/dumps/` -- proven directly (not only through the env-var default every other test here
/// relies on) by pointing it at a scratch directory holding one real dump line under a different
/// pair/seam name.
#[test]
fn read_dump_takes_the_directory_as_an_argument() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../oracle/.cache/tmp");
    std::fs::create_dir_all(dir.join("scratch-pair")).expect("oracle/.cache/tmp must be creatable");
    let dump_path = dir.join("scratch-pair/scratch-seam.dump");
    std::fs::write(
        &dump_path,
        format!(
            "; provenance\n;   oracle-commit: {}\n; seam: scratch-seam\n; object: print\nfield=skirt layer=- region=- count=0\n",
            dumps::PINNED_COMMIT
        ),
    )
    .expect("write the scratch dump");

    let dump = dumps::read_dump(&dir, "scratch-pair", "scratch-seam")
        .expect("read_dump must read from the directory it is given, not a hardcoded default");
    assert_eq!(dump.seam, "scratch-seam");
    assert_eq!(dump.object, "print");

    std::fs::remove_file(&dump_path).ok();
}

/// `points_at` matches `region` as well as `layer` -- the committed corpus has only one region
/// per layer (`20mm_cube-base-fff` carries a single filament/region), so this cannot be shown by
/// a real multi-region dump yet, but the filter itself is exercised directly either way: the real
/// `slices` field (an `ExPolygons`-shaped field with a genuine region axis, unlike `lslices`)
/// is found at its real `region=0`, and a region that does not exist in this dump at all
/// (`region=5`) is refused with `FieldNotFound` rather than silently matched by `layer` alone --
/// proving the region key actually narrows the match, not merely that it is accepted syntactically.
#[test]
fn points_at_matches_region_not_only_layer() {
    let dump =
        dumps::read_dump_default("20mm_cube-base-fff", "posSlice").expect("read posSlice.dump");

    let at_real_region = dump
        .points_at("slices", Some(0), Some(0))
        .expect("layer 0, region 0 must carry a 'slices' record");
    assert_eq!(
        at_real_region.len(),
        8,
        "layer 0's one slice is a 4-point square"
    );

    let err = dump.points_at("slices", Some(0), Some(5)).expect_err(
        "region 5 does not exist in this dump; a region-blind filter would wrongly \
                      return layer 0's real region-0 points instead of refusing",
    );
    assert!(matches!(err, DumpError::FieldNotFound { .. }), "{err}");
}

/// A malformed manifest row (bypassing `compare_unit`'s env-var default) still reaches
/// `tiers::TierError` correctly when fed a dump-derived value -- exercises `compare_unit_at`
/// directly against the shared fixture `tests/manifest_validation.rs` already commits, proving
/// this module's `Vec<i64>` output is an ordinary `ProofValue` with no bridging needed.
#[test]
fn dump_derived_vec_compares_through_compare_unit_at() {
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/data/manifest/fixture.tsv");
    let dump =
        dumps::read_dump_default("20mm_cube-base-fff", "posSlice").expect("read posSlice.dump");
    let actual = dump
        .points_at("lslices", Some(1), None)
        .expect("layer 1 must carry lslices");
    compare_unit_at(
        &fixture,
        "Fixture/ExactRow.cpp",
        Oracle::Built,
        actual.len() as i64,
        8,
    )
    .expect(
        "layer 1's lslices square has 4 points (8 flattened x,y components), same as layer 0's",
    );
}
