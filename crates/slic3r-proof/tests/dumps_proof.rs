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
/// `9007199254740992`). Reads the hand-written fixture through
/// `dumps::parse_hand_written_fixture_records` directly -- the fixture carries no provenance
/// header by design (see its own leading comment), so `dumps::read_dump`, which enforces rule 2
/// below, would refuse it; this is the one function this module exposes with no such requirement.
#[test]
fn oversized_coordinate_survives_exact() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/data/oversized-coordinate.dump");
    let text = std::fs::read_to_string(&path).expect("the hand-written fixture must be committed");
    let records = dumps::parse_hand_written_fixture_records(&text, &path)
        .expect("the fixture's one line must parse");
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

/// `points_at` refuses (rather than silently returning) a field whose records carry an `object=`
/// key -- `object_skirt` is `psSkirtBrim`'s print-wide, per-`Print::objects()` record, so a call
/// with no way to pin that axis must not be answered by merging every object's coordinates into
/// one list. Proven against the real committed `port/dumps/20mm_cube-base-fff/psSkirtBrim.dump`,
/// whose one `object_skirt` record already carries `object=0`.
#[test]
fn points_at_refuses_a_field_with_an_object_axis() {
    let dump = dumps::read_dump_default("20mm_cube-base-fff", "psSkirtBrim")
        .expect("read psSkirtBrim.dump");
    let err = dump
        .points_at("object_skirt", None, None)
        .expect_err("object_skirt carries an 'object=' key this call does not constrain");
    match err {
        DumpError::AmbiguousAxis { key, .. } => assert_eq!(key, "object"),
        other => panic!("expected AmbiguousAxis, got {other}"),
    }
}

/// The same refusal for the `group=`/`brim=` axes `skirt_brim_group_skirt`/
/// `skirt_brim_group_brim` carry -- proven against the real committed
/// `port/dumps/3DBenchy-base-fff/psSkirtBrim.dump` (`20mm_cube-base-fff`'s own copy has no skirt
/// or brim groups: `base-fff`'s `skirt_brim_groups count=0`), whose `skirt_brim_group_brim`
/// records carry `group=0 brim=0`.
#[test]
fn points_at_refuses_a_field_with_a_group_axis() {
    let dump = dumps::read_dump_default("3DBenchy-base-fff", "psSkirtBrim")
        .expect("read 3DBenchy-base-fff/psSkirtBrim.dump");
    let err = dump
        .points_at("skirt_brim_group_brim", None, None)
        .expect_err("skirt_brim_group_brim carries a 'group=' key this call does not constrain");
    match err {
        DumpError::AmbiguousAxis { key, .. } => assert_eq!(key, "group"),
        other => panic!("expected AmbiguousAxis, got {other}"),
    }
}

/// The false-pass this refusal replaces, shown directly: two objects' `object_skirt` records
/// (synthetic, built in memory through `parse_dump_text` with the real pinned header -- not a
/// second committed `.dump` file, since `oversized-coordinate.dump` is this branch's one
/// hand-written exception) would, without the fix, silently concatenate into one list;
/// `points_at` now refuses the call outright instead, and a caller that actually needs
/// per-object access reads `Record::key_i64(path, "object")` on `dump.field("object_skirt")`
/// directly (the header-level `seam`/`object` keys plus this record key are what the "keyed by
/// seam, object, layer and region" Done-when line is satisfied through).
#[test]
fn points_at_never_merges_two_objects_coordinates() {
    let text = format!(
        "; provenance\n;   oracle-commit: {}\n; seam: psSkirtBrim\n; object: print\n\
         field=object_skirt layer=- region=- object=0 count=1\n\
         field=object_skirt layer=- region=- object=0 path=0 role=Skirt loop=- n=2 pts=1,1,0 2,2,0\n\
         field=object_skirt layer=- region=- object=1 count=1\n\
         field=object_skirt layer=- region=- object=1 path=0 role=Skirt loop=- n=2 pts=7,7,0 8,8,0\n",
        dumps::PINNED_COMMIT
    );
    let dump = dumps::parse_dump_text(&text, Path::new("synthetic/two-object-skirt.dump"))
        .expect("synthetic two-object text must parse");
    let err = dump
        .points_at("object_skirt", None, None)
        .expect_err("two objects' records must never be silently merged into one list");
    assert!(matches!(err, DumpError::AmbiguousAxis { .. }), "{err}");

    // The per-record axis a caller falls back to still discriminates correctly.
    let object_0: Vec<i64> = dump
        .field("object_skirt")
        .filter(|r| r.key_i64(&dump.path, "object").unwrap() == Some(0))
        .flat_map(|r| r.points.iter().flatten().copied())
        .collect();
    assert_eq!(object_0, vec![1, 1, 0, 2, 2, 0]);
}

/// `points_at_support_layer` discriminates by `support_layer`, not merging every support layer
/// into one list (synthetic text, same reasoning as above: `20mm_cube-base-fff`'s `base-fff`
/// configuration has no support material at all, so no committed dump carries a `support_layer=`
/// record to prove this against).
#[test]
fn points_at_support_layer_discriminates_by_support_layer() {
    let text = format!(
        "; provenance\n;   oracle-commit: {}\n; seam: posSupportMaterial\n; object: 0\n\
         field=lslices layer=0 region=- poly=0 ring=contour n=2 pts=9,9 10,10\n\
         field=support_islands layer=- region=- support_layer=0 poly=0 ring=contour n=2 pts=1,1 2,2\n\
         field=support_islands layer=- region=- support_layer=1 poly=0 ring=contour n=2 pts=3,3 4,4\n",
        dumps::PINNED_COMMIT
    );
    let dump = dumps::parse_dump_text(&text, Path::new("synthetic/two-support-layers.dump"))
        .expect("synthetic two-support-layer text must parse");

    assert_eq!(
        dump.points_at_support_layer("support_islands", Some(0))
            .expect("support_layer 0 must be found"),
        vec![1, 1, 2, 2]
    );
    assert_eq!(
        dump.points_at_support_layer("support_islands", Some(1))
            .expect("support_layer 1 must be found"),
        vec![3, 3, 4, 4]
    );

    // A field with no `support_layer=` key at all (e.g. `lslices`) is never a candidate match,
    // regardless of the `support_layer` argument -- before the fix, `None` matched it by treating
    // its absent key as equal to `None`, silently merging every layer and region together.
    let err = dump
        .points_at_support_layer("lslices", None)
        .expect_err("a field with no support_layer key must never match points_at_support_layer");
    assert!(matches!(err, DumpError::FieldNotFound { .. }), "{err}");
}

/// `Record::value_f64`/`scalar_f64`/`pair_f64` over the real committed
/// `port/dumps/20mm_cube-base-fff/psWipeTower.dump`: a plain scalar (`wipe_tower_depth`), a plain
/// decimal integer scalar (`wipe_tower_number_of_toolchanges`), a `min=`/`max=` pair
/// (`wipe_tower_bbx`), and a comma-paired `value=` (`wipe_tower_rib_offset`) -- the two shapes
/// `oracle/stage-dump/README.md`'s "Dump format" section names for these fields, all actually
/// readable through a typed accessor rather than only through `Record::raw`.
#[test]
fn scalar_and_pair_accessors_read_the_real_wipe_tower_dump() {
    let dump = dumps::read_dump_default("20mm_cube-base-fff", "psWipeTower")
        .expect("read psWipeTower.dump");

    let depth = dump
        .field("wipe_tower_depth")
        .next()
        .expect("wipe_tower_depth record must exist");
    assert_eq!(depth.value_f64(&dump.path).unwrap(), Some(0.0));

    let toolchanges = dump
        .field("wipe_tower_number_of_toolchanges")
        .next()
        .expect("wipe_tower_number_of_toolchanges record must exist");
    assert_eq!(toolchanges.value_f64(&dump.path).unwrap(), Some(-1.0));

    let bbx = dump
        .field("wipe_tower_bbx")
        .next()
        .expect("wipe_tower_bbx record must exist");
    assert_eq!(
        bbx.value_f64(&dump.path).unwrap(),
        None,
        "no 'value=' key on this record"
    );
    assert_eq!(bbx.pair_f64(&dump.path, "min").unwrap(), Some((0.0, 0.0)));
    assert_eq!(bbx.pair_f64(&dump.path, "max").unwrap(), Some((0.0, 0.0)));

    let rib_offset = dump
        .field("wipe_tower_rib_offset")
        .next()
        .expect("wipe_tower_rib_offset record must exist");
    rib_offset
        .value_f64(&dump.path)
        .expect_err("'value=' on this record is a comma-separated pair, not a single scalar");
    assert_eq!(
        rib_offset.pair_f64(&dump.path, "value").unwrap(),
        Some((0.0, 0.0))
    );
}

/// `Record::value_f64` over the real committed `port/dumps/20mm_cube-base-fff/posSlice.dump`'s
/// `print_z` field at layer 0 -- the one normal-range `parse_hexfloat` case no committed test
/// exercised before this task's fix pass, matching Python's own `float.fromhex` for the same
/// token (`0x1.999999999999ap-3` -> `0.2`).
#[test]
fn print_z_scalar_parses_the_real_hexfloat_token() {
    let dump =
        dumps::read_dump_default("20mm_cube-base-fff", "posSlice").expect("read posSlice.dump");
    let print_z_layer_0 = dump
        .field("print_z")
        .find(|r| r.layer(&dump.path).unwrap() == Some(0))
        .expect("print_z must carry a layer-0 record");
    assert_eq!(print_z_layer_0.value_f64(&dump.path).unwrap(), Some(0.2));
}

/// `parse_hexfloat`'s subnormal-exponent fix: `0x1p-1074` is the smallest positive subnormal
/// `f64` (`f64::from_bits(1)`), which the original single-`powi` scaling silently rounded to
/// `0.0` (`2f64.powi(-1074)` overflows its own intermediate to `0.0`). Synthetic text (no real
/// dump ever carries a coordinate or scalar anywhere near this magnitude -- see the module doc),
/// checked bit-for-bit against `f64::from_bits`, not a decimal literal, since `4.9e-324` would
/// itself round through parsing.
#[test]
fn subnormal_hexfloat_survives_exact() {
    let text = format!(
        "; provenance\n;   oracle-commit: {}\n; seam: posSlice\n; object: 0\n\
         field=subnormal_probe layer=- region=- value=0x1p-1074\n",
        dumps::PINNED_COMMIT
    );
    let dump = dumps::parse_dump_text(&text, Path::new("synthetic/subnormal.dump"))
        .expect("synthetic subnormal text must parse");
    let record = dump.field("subnormal_probe").next().unwrap();
    let value = record.value_f64(&dump.path).unwrap().unwrap();
    assert_eq!(value.to_bits(), f64::from_bits(1).to_bits());
    assert_ne!(
        value, 0.0,
        "the original scaling rounded this subnormal value to zero"
    );
}

/// A `pts=` line whose declared `n=` disagrees with the number of tuples actually present is
/// refused rather than silently accepted -- a truncated or garbled line, not merely an
/// unexpected one.
#[test]
fn declared_point_count_mismatch_is_refused() {
    let text = format!(
        "; provenance\n;   oracle-commit: {}\n; seam: posSlice\n; object: 0\n\
         field=lslices layer=0 region=- poly=0 ring=contour n=2 pts=1,2\n",
        dumps::PINNED_COMMIT
    );
    let err = dumps::parse_dump_text(&text, Path::new("synthetic/bad-n.dump"))
        .expect_err("n=2 with only one pts tuple present must be refused");
    assert!(matches!(err, DumpError::MalformedLine { .. }), "{err}");
}

/// A `pts=` tuple whose component count is neither 2 (`x,y`) nor 3 (`x,y,z`) is refused, matching
/// `Record::points`'s own documented invariant.
#[test]
fn malformed_point_arity_is_refused() {
    let text = format!(
        "; provenance\n;   oracle-commit: {}\n; seam: posSlice\n; object: 0\n\
         field=lslices layer=0 region=- poly=0 ring=contour n=1 pts=1,2,3,4,5\n",
        dumps::PINNED_COMMIT
    );
    let err = dumps::parse_dump_text(&text, Path::new("synthetic/bad-arity.dump"))
        .expect_err("a 5-component 'pts=' tuple must be refused");
    assert!(matches!(err, DumpError::MalformedLine { .. }), "{err}");
}
