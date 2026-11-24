//! Exercises all five levels against the committed fragments under `tests/data/`. L0 and L1 are
//! task 08's; L2, L3 and L4 are this task's (`09-comparator-gcode-levels`), and every pair here
//! is evaluated at all five levels at once — none of them are "not evaluated" from this task
//! onward.

use oracle_compare::{
    compare, compare_with_tolerance, tolerance, FailLocation, Level, LevelOutcome, Side,
};

fn data_dir() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/data")
}

fn read_pair(name: &str) -> (Vec<u8>, Vec<u8>) {
    let dir = data_dir().join(name);
    let a = std::fs::read(dir.join("a.gcode"))
        .unwrap_or_else(|e| panic!("reading {}/a.gcode: {e}", dir.display()));
    let b = std::fs::read(dir.join("b.gcode"))
        .unwrap_or_else(|e| panic!("reading {}/b.gcode: {e}", dir.display()));
    (a, b)
}

fn read_file(relative: &str) -> Vec<u8> {
    let path = data_dir().join(relative);
    std::fs::read(&path).unwrap_or_else(|e| panic!("reading {}: {e}", path.display()))
}

#[test]
fn identical_pair_passes_every_level() {
    // Byte-identical, so every level passes: L2's tokens, L3's moves and L4's aggregates are all
    // trivially equal once L0 already is.
    let (a, b) = read_pair("identical");
    let report = compare(&a, &b);
    for level in Level::ALL {
        assert_eq!(
            *report.outcome(level),
            LevelOutcome::Pass,
            "{level} should pass"
        );
    }
    assert_eq!(report.strongest_evaluated_pass, Some(Level::L4));
    assert_eq!(report.stricter_failure, None);
    assert_eq!(report.first_differing_byte, None);
}

#[test]
fn generated_by_pair_fails_l0_only_and_names_l4_strongest() {
    // The only difference is the attribution comment line, which L1 strips and which is
    // comment-only to begin with, so it never reaches L2's token stream or L3/L4's move model:
    // every level but L0 passes.
    let (a, b) = read_pair("generated_by");
    let report = compare(&a, &b);
    match report.outcome(Level::L0) {
        LevelOutcome::Fail(FailLocation::ByteOffset(_)) => {}
        other => panic!("expected L0 byte-offset failure, got {other:?}"),
    }
    for level in [Level::L1, Level::L2, Level::L3, Level::L4] {
        assert_eq!(
            *report.outcome(level),
            LevelOutcome::Pass,
            "{level} should pass"
        );
    }
    assert_eq!(report.strongest_evaluated_pass, Some(Level::L4));
    // `stricter_failure` is anchored at the strictest *passing* level (L1 here, since L0 is the
    // only failure), not at `strongest_evaluated_pass` (L4): L0's byte-offset failure sits
    // immediately beneath L1, and that is what a reader needs to see, not nothing just because
    // L3 (immediately beneath L4) happened to pass too. See `Report`'s doc comment.
    assert_eq!(
        report.stricter_failure,
        Some((
            Level::L0,
            FailLocation::ByteOffset(match report.outcome(Level::L0) {
                LevelOutcome::Fail(FailLocation::ByteOffset(off)) => *off,
                _ => unreachable!(),
            })
        ))
    );
}

#[test]
fn plugin_armed_pair_passes_every_level_but_l0() {
    let (a, b) = read_pair("plugin_armed");
    let report = compare(&a, &b);
    assert!(
        !report.passed(Level::L0),
        "the two files differ by a whole line, L0 must fail"
    );
    for level in [Level::L1, Level::L2, Level::L3, Level::L4] {
        assert_eq!(
            *report.outcome(level),
            LevelOutcome::Pass,
            "{level} should pass"
        );
    }
    assert_eq!(report.strongest_evaluated_pass, Some(Level::L4));
}

#[test]
fn model_label_pair_passes_every_level_but_l0() {
    // The object-label text differs (id 7 vs 9) but object labels are not an L4 grouping
    // dimension and not part of L3's per-move comparison, so this is still an L0-only failure.
    let (a, b) = read_pair("model_label");
    let report = compare(&a, &b);
    assert!(!report.passed(Level::L0));
    for level in [Level::L1, Level::L2, Level::L3, Level::L4] {
        assert_eq!(
            *report.outcome(level),
            LevelOutcome::Pass,
            "{level} should pass"
        );
    }
}

#[test]
fn one_coordinate_pair_fails_every_level() {
    let (a, b) = read_pair("one_coordinate");
    let report = compare(&a, &b);
    assert!(!report.passed(Level::L0));
    match report.outcome(Level::L1) {
        LevelOutcome::Fail(FailLocation::Line { number, .. }) => {
            assert!(*number > 0);
        }
        other => panic!("expected an L1 line failure, got {other:?}"),
    }
    assert!(
        !report.passed(Level::L2),
        "the Y literal differs, L2 must fail"
    );
    assert!(
        !report.passed(Level::L3),
        "the Y value differs by 1.0, L3 must fail at exact tolerance"
    );
    assert!(
        !report.passed(Level::L4),
        "the bounding box differs, L4 must fail"
    );
    assert_eq!(report.strongest_evaluated_pass, None);
    assert!(report.first_differing_byte.is_some());
}

#[test]
fn unrelated_pair_fails_every_level_from_the_first_byte() {
    let (a, b) = read_pair("unrelated");
    let report = compare(&a, &b);
    for level in Level::ALL {
        assert!(
            !report.passed(level),
            "{level} should fail on two unrelated files"
        );
    }
    assert_eq!(report.strongest_evaluated_pass, None);
    assert_eq!(report.first_differing_byte, Some(0));
}

#[test]
fn empty_input_normalises_to_empty() {
    assert_eq!(oracle_compare::normalise(b""), Vec::<u8>::new());
}

#[test]
fn missing_final_newline_gains_one() {
    assert_eq!(oracle_compare::normalise(b"G28"), b"G28\n".to_vec());
}

#[test]
fn model_label_id_is_a_prefix_match_not_a_substring_match() {
    let kept = oracle_compare::normalise(b"G1 X1 ; model label id: 7\n");
    assert_eq!(kept, b"G1 X1 ; model label id: 7\n".to_vec());
}

#[test]
fn generated_by_is_a_prefix_match_not_a_substring_match() {
    let kept = oracle_compare::normalise(b"G1 X1 ; generated by x\n");
    assert_eq!(kept, b"G1 X1 ; generated by x\n".to_vec());
}

#[test]
fn carriage_return_before_newline_is_kept() {
    let kept = oracle_compare::normalise(b"G28\r\n");
    assert_eq!(kept, b"G28\r\n".to_vec());
}

#[test]
fn invalid_utf8_byte_in_a_kept_line_survives() {
    let input: &[u8] = b"; comment \xff end\n";
    assert_eq!(oracle_compare::normalise(input), input.to_vec());
}

#[test]
fn trailing_blank_line_difference_reports_the_real_divergence() {
    let a = b"G28\n\n";
    let b = b"G28\n";
    let report = oracle_compare::compare(a, b);
    match report.outcome(Level::L1) {
        LevelOutcome::Fail(FailLocation::Line { number, a, b }) => {
            assert_eq!(*number, 2);
            assert_eq!(a.as_deref(), Some(&b""[..]));
            assert_eq!(b.as_deref(), None);
        }
        other => panic!("expected an L1 line failure at line 2, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------------------------
// L2: identical token stream
// ---------------------------------------------------------------------------------------------

#[test]
fn comment_text_pair_l2_passes_while_l0_fails() {
    // The only difference between the two files is the trailing comment text; comments
    // contribute no tokens at all, so L2 passes even though L0 (and L1 — this is not one of the
    // four declared normalisations) fail.
    let (a, b) = read_pair("comment_text");
    let report = compare(&a, &b);
    assert!(!report.passed(Level::L0));
    assert_eq!(*report.outcome(Level::L2), LevelOutcome::Pass);
}

#[test]
fn coordinate_literal_pair_l2_fails_while_l3_passes() {
    // `X10.0` vs `X10`: different literal text (L2 fails) but the same parsed value (L3 passes
    // at exact tolerance) — L2 is a text comparison, L3 a numeric one.
    let (a, b) = read_pair("coordinate_literal");
    let report = compare(&a, &b);
    match report.outcome(Level::L2) {
        LevelOutcome::Fail(FailLocation::Token { .. }) => {}
        other => panic!("expected an L2 token failure, got {other:?}"),
    }
    assert_eq!(*report.outcome(Level::L3), LevelOutcome::Pass);
}

#[test]
fn unrelated_pair_l2_failure_names_both_lines() {
    let (a, b) = read_pair("unrelated");
    let report = compare(&a, &b);
    match report.outcome(Level::L2) {
        LevelOutcome::Fail(FailLocation::Token {
            line_a,
            line_b,
            text_a,
            text_b,
            ..
        }) => {
            assert!(line_a.is_some() && line_b.is_some());
            assert!(text_a.is_some() && text_b.is_some());
        }
        other => panic!("expected an L2 token failure, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------------------------
// L3: move count/order and per-axis tolerance
// ---------------------------------------------------------------------------------------------

#[test]
fn small_drift_fails_l3_at_exact_tolerance() {
    let (a, b) = read_pair("small_drift");
    let report = compare(&a, &b);
    match report.outcome(Level::L3) {
        LevelOutcome::Fail(FailLocation::Move { .. }) => {}
        other => panic!("expected an L3 move failure, got {other:?}"),
    }
    let dev = report
        .l3_axis_deviations
        .get(&'X')
        .expect("X was compared on both sides");
    assert!(
        (dev.max - 0.001).abs() < 1e-9,
        "expected max X deviation 0.001, got {}",
        dev.max
    );
}

#[test]
fn small_drift_passes_l3_against_a_fixture_manifest_row_tolerance() {
    let (a, b) = read_pair("small_drift");
    let manifest = data_dir().join("manifest/tolerance.tsv");
    let tol = tolerance::from_manifest_row(manifest.to_str().unwrap(), "Foo/Bar.cpp")
        .expect("Foo/Bar.cpp has a valid tolerance cell");
    let report = compare_with_tolerance(&a, &b, tol);
    assert_eq!(*report.outcome(Level::L3), LevelOutcome::Pass);
    assert!(report.tolerance.to_string().contains("reason"));
}

#[test]
fn tolerance_cell_with_no_reason_is_refused_and_names_the_row() {
    let manifest = data_dir().join("manifest/tolerance.tsv");
    let err = tolerance::from_manifest_row(manifest.to_str().unwrap(), "Foo/Baz.cpp")
        .expect_err("a tolerance value with no reason must be refused");
    assert!(
        err.contains("Foo/Baz.cpp"),
        "error should name the row: {err}"
    );
    assert!(err.contains("reason"), "error should say why: {err}");
}

#[test]
fn empty_tolerance_cell_defaults_to_exact_silently() {
    let manifest = data_dir().join("manifest/tolerance.tsv");
    let tol = tolerance::from_manifest_row(manifest.to_str().unwrap(), "Foo/Qux.cpp")
        .expect("an empty tolerance cell is not an error");
    assert_eq!(tol, oracle_compare::Tolerance::Exact);
}

#[test]
fn manifest_with_no_matching_row_is_refused_and_names_it() {
    let manifest = data_dir().join("manifest/tolerance.tsv");
    let err = tolerance::from_manifest_row(manifest.to_str().unwrap(), "No/Such.cpp")
        .expect_err("no row has this path");
    assert!(
        err.contains("No/Such.cpp"),
        "error should name the row: {err}"
    );
}

#[test]
fn manifest_missing_tolerance_columns_is_refused() {
    let manifest = data_dir().join("manifest/no_tolerance_column.tsv");
    let err = tolerance::from_manifest_row(manifest.to_str().unwrap(), "Foo/Bar.cpp")
        .expect_err("this manifest has no tolerance or comparison function column");
    assert!(
        err.contains("tolerance"),
        "error should name the missing column: {err}"
    );
}

#[test]
fn missing_manifest_file_is_refused() {
    let err = tolerance::from_manifest_row("/no/such/manifest.tsv", "Foo/Bar.cpp")
        .expect_err("the file does not exist");
    assert!(err.contains("/no/such/manifest.tsv"));
}

#[test]
fn inserted_move_fails_l3_with_count_and_localisation() {
    let (a, b) = read_pair("inserted_move");
    let report = compare(&a, &b);
    match report.outcome(Level::L3) {
        LevelOutcome::Fail(FailLocation::Move {
            index,
            layer,
            role,
            object,
            count_a,
            count_b,
        }) => {
            assert_eq!(*count_a, 2);
            assert_eq!(*count_b, 3);
            assert_eq!(
                *index, 2,
                "the shorter side has no move at the first extra index"
            );
            assert_eq!(*layer, Some(0));
            assert_eq!(role.as_deref(), Some("Perimeter"));
            assert_eq!(*object, None);
        }
        other => panic!("expected an L3 move-count failure, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------------------------
// L4: per-layer and per-role aggregates
// ---------------------------------------------------------------------------------------------

#[test]
fn reordered_moves_pass_l4_while_failing_l3() {
    // Same two moves, opposite order, chosen (see tests/data/README.md) so every L4 aggregate
    // — move count, extruded length, E delta, bounding box, travel distance, layer count —
    // comes out byte-for-byte equal despite the per-move X values disagreeing outright. This is
    // the proof that L3 and L4 are evaluated independently rather than one nested inside the
    // other.
    let (a, b) = read_pair("reordered_moves");
    let report = compare(&a, &b);
    assert!(
        !report.passed(Level::L3),
        "the per-move X values differ, L3 must fail"
    );
    assert_eq!(
        *report.outcome(Level::L4),
        LevelOutcome::Pass,
        "the aggregates agree exactly"
    );
}

#[test]
fn layer_count_mismatch_fails_l4_naming_the_layer() {
    let (a, b) = read_pair("layer_count_mismatch");
    let report = compare(&a, &b);
    match report.outcome(Level::L4) {
        LevelOutcome::Fail(FailLocation::LayerCount {
            count_a,
            count_b,
            differing_layer,
            present_in,
            ..
        }) => {
            assert_eq!(*count_a, 1);
            assert_eq!(*count_b, 2);
            assert_eq!(*differing_layer, 1);
            assert_eq!(*present_in, Side::B);
        }
        LevelOutcome::Fail(other) => {
            panic!(
                "expected the mismatch to be reported as a layer count difference, got {other:?}"
            )
        }
        other => panic!("expected an L4 failure, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------------------------
// The localisation and the two tag spellings
// ---------------------------------------------------------------------------------------------

#[test]
fn named_move_reports_its_layer_role_and_object() {
    let data = read_file("localisation/fragment.gcode");
    let parsed = oracle_compare::gcode::parse(&data);
    assert_eq!(parsed.moves.len(), 3);

    // "G1 X10 Y0 E1", the second move: inside the object label, still on layer 0, role
    // "Perimeter".
    let named = &parsed.moves[1];
    assert_eq!(named.localisation.layer, Some(0));
    assert_eq!(named.localisation.role.as_deref(), Some("Perimeter"));
    assert_eq!(
        named.localisation.object.as_deref(),
        Some("Box id:3 copy 0")
    );

    // The third move, after the object's stop line and the second layer tag: object cleared,
    // layer advanced, role changed.
    let after = &parsed.moves[2];
    assert_eq!(after.localisation.layer, Some(1));
    assert_eq!(after.localisation.role.as_deref(), Some("Internal infill"));
    assert_eq!(after.localisation.object, None);
}

#[test]
fn bambu_and_compatible_tag_spellings_parse_to_the_same_model_shape() {
    let bambu = read_file("tag_spellings/bambu.gcode");
    let compatible = read_file("tag_spellings/compatible.gcode");

    let parsed_bambu = oracle_compare::gcode::parse(&bambu);
    let parsed_compatible = oracle_compare::gcode::parse(&compatible);

    assert_eq!(parsed_bambu.moves.len(), 1);
    assert_eq!(parsed_compatible.moves.len(), 1);

    let a = &parsed_bambu.moves[0];
    let b = &parsed_compatible.moves[0];
    assert_eq!(a.localisation.layer, b.localisation.layer);
    assert_eq!(a.localisation.z, b.localisation.z);
    assert_eq!(a.localisation.role, b.localisation.role);
    assert_eq!(a.x, b.x);
    assert_eq!(a.y, b.y);

    assert!(parsed_bambu.tag_spellings.layer_bambu);
    assert!(parsed_bambu.tag_spellings.z_bambu);
    assert!(parsed_bambu.tag_spellings.role_bambu);
    assert!(parsed_compatible.tag_spellings.layer_compatible);
    assert!(parsed_compatible.tag_spellings.z_compatible);
    assert!(parsed_compatible.tag_spellings.role_compatible);
}

// ---------------------------------------------------------------------------------------------
// L4's six quantities, pinned against the hand computation in levels.rs's module doc comment
// ---------------------------------------------------------------------------------------------

#[test]
fn reordered_moves_l4_aggregates_match_the_hand_computed_values() {
    // The numbers below are computed by hand in `levels.rs`'s module doc comment, walking
    // `a.gcode`'s two moves (`M83` relative mode, `G1 X10 Y0 E1` then `G1 X-10 Y0 E1`) from the
    // implicit origin. A mutation that swaps `extruded_length` and `travel_distance` in
    // `fold_group`, or that subtracts a retraction from `e_delta`, must make this test fail —
    // confirmed by running that mutation against this test before committing it.
    let (a, b) = read_pair("reordered_moves");
    let report = compare(&a, &b);
    let whole_a = &report.l4_aggregates_a.whole;
    let whole_b = &report.l4_aggregates_b.whole;

    for (label, agg) in [("a", whole_a), ("b", whole_b)] {
        assert_eq!(agg.move_count, 2, "{label}: move count");
        assert!(
            (agg.extruded_length - 30.0).abs() < 1e-9,
            "{label}: extruded length, got {}",
            agg.extruded_length
        );
        assert!(
            (agg.travel_distance - 0.0).abs() < 1e-9,
            "{label}: travel distance, got {}",
            agg.travel_distance
        );
        assert!(
            (agg.e_delta - 2.0).abs() < 1e-9,
            "{label}: E delta, got {}",
            agg.e_delta
        );
        assert_eq!(
            agg.bbox,
            Some((-10.0, 10.0, 0.0, 0.0)),
            "{label}: bounding box"
        );
        assert_eq!(agg.layer_count(), 1, "{label}: layer count");
    }
}

#[test]
fn a_retraction_does_not_subtract_from_e_delta() {
    // `G1 E1` advances (positive delta +1), `G1 E-1` retracts (negative delta, must contribute
    // nothing — not subtract 1, which would read E delta as 0). Absolute E mode (the default),
    // so the literal E values are the running position, not a delta: 0 -> 1 is +1, 1 -> 0 is -1.
    let gcode = b"G1 X1 E1\nG1 X2 E0\n";
    let parsed = oracle_compare::gcode::parse(gcode);
    let agg = oracle_compare::compare(gcode, gcode).l4_aggregates_a.whole;
    assert_eq!(
        parsed.moves[1].e_delta,
        Some(-1.0),
        "the retraction's own delta"
    );
    assert!(
        (agg.e_delta - 1.0).abs() < 1e-9,
        "only the +1 advance should count, got {}",
        agg.e_delta
    );
}

// ---------------------------------------------------------------------------------------------
// The whole-file layer count the report prints must be the number L4 actually judged
// ---------------------------------------------------------------------------------------------

#[test]
fn whole_file_layer_count_printed_matches_the_outcome_l4_judged() {
    // `layer_count_mismatch/b.gcode` has a trailing `; CHANGE_LAYER` with no move after it, so
    // the parser's raw boundary count (2) disagrees with the moves-derived count (1, since no
    // move ever touches the second layer). `render_text`/`render_json` must print the boundary
    // count for the whole-file group — the number the `L4: fail` line actually names — not the
    // moves-derived one, or the same report would state two different "layer count"s for the
    // same group.
    let (a, b) = read_pair("layer_count_mismatch");
    let report = compare(&a, &b);
    assert_eq!(report.layer_boundaries_a, 1);
    assert_eq!(report.layer_boundaries_b, 2);
    // The moves-derived count is 1 on both sides (confirming the discrepancy exists to catch).
    assert_eq!(report.l4_aggregates_a.whole.layer_count(), 1);
    assert_eq!(report.l4_aggregates_b.whole.layer_count(), 1);

    let text = oracle_compare::format::render_text(&report);
    assert!(
        text.contains("layer count: a=1 b=2\n"),
        "whole-file layer count in text output should be the boundary count: {text}"
    );

    let json = oracle_compare::format::render_json(&report);
    assert!(
        json.contains("\"whole_file\":{\"a\":{") && json.contains("\"layer_count\":1"),
        "{json}"
    );
    // The JSON L4 outcome (count_a=1, count_b=2) and the whole-file aggregate's own
    // `layer_count` must agree, which they would not if the aggregate printed its
    // moves-derived count (1) instead of the boundary count (2) for side b.
    assert!(json.contains("\"L4\":{\"outcome\":\"fail\""), "{json}");
}

// ---------------------------------------------------------------------------------------------
// A pair with no G0-G3 moves on either side: documented vacuous pass (levels.rs's doc comment)
// ---------------------------------------------------------------------------------------------

#[test]
fn moves_free_pair_passes_l3_and_l4_vacuously() {
    // Two unrelated, non-G-code texts with no move command at all: L0-L2 all fail outright, but
    // L3 and L4 compare two empty move lists and pass. This is the documented decision in
    // `levels.rs`'s doc comment on `evaluate_l3`, pinned here so it reads as an intended outcome
    // rather than something a later task discovers by surprise.
    let a = b"totally unrelated text\nwith no g-code in it at all\n";
    let b = b"a different unrelated text\nalso with nothing that parses as a move\n";
    let report = compare(a, b);
    assert!(!report.passed(Level::L0));
    assert!(!report.passed(Level::L1));
    assert!(!report.passed(Level::L2));
    assert_eq!(
        *report.outcome(Level::L3),
        LevelOutcome::Pass,
        "two empty move lists are vacuously equal"
    );
    assert_eq!(
        *report.outcome(Level::L4),
        LevelOutcome::Pass,
        "two empty aggregates are vacuously equal"
    );
}

// ---------------------------------------------------------------------------------------------
// CRLF vs LF: an undeclared-but-decided normalisation above L1 (gcode.rs's doc comment)
// ---------------------------------------------------------------------------------------------

#[test]
fn crlf_and_lf_pair_agree_from_l2_up() {
    let lf: &[u8] = b"; CHANGE_LAYER\n; FEATURE: Perimeter\nG1 X10 Y0 E1\nG1 X20 Y0 E1\n";
    let crlf: Vec<u8> = lf
        .split(|&b| b == b'\n')
        .filter(|l| !l.is_empty())
        .flat_map(|l| l.iter().copied().chain([b'\r', b'\n']))
        .collect();
    let report = compare(lf, &crlf);
    // L0 and L1 keep `\r` (R4 normalises nothing else), so they still disagree.
    assert!(!report.passed(Level::L0), "bytes differ by every \\r");
    assert!(
        !report.passed(Level::L1),
        "R4's declared normalisation does not touch \\r"
    );
    assert_eq!(*report.outcome(Level::L2), LevelOutcome::Pass);
    assert_eq!(*report.outcome(Level::L3), LevelOutcome::Pass);
    assert_eq!(*report.outcome(Level::L4), LevelOutcome::Pass);
}

// ---------------------------------------------------------------------------------------------
// evaluate_within: the public API for "the level reached within one layer, role or object"
// ---------------------------------------------------------------------------------------------

#[test]
fn evaluate_within_carries_forward_the_real_previous_position_across_the_scope_boundary() {
    // Two layers; the move being scoped to (layer 1) is not the first move in the file. If the
    // scope's aggregate were computed by filtering the move slice first and only then measuring
    // consecutive deltas (the recipe an earlier version of `gcode.rs`'s doc comment described),
    // this move's length would be measured from the origin (distance ≈ 20.6) instead of from
    // where layer 0's move actually left off (distance = 5).
    let gcode =
        b"; CHANGE_LAYER\n; FEATURE: Perimeter\nG1 X20 Y0 E1\n; CHANGE_LAYER\nG1 X20 Y5 E2\n";
    let parsed = oracle_compare::gcode::parse(gcode);
    assert_eq!(parsed.moves.len(), 2);
    assert_eq!(parsed.moves[1].localisation.layer, Some(1));

    let (l3, l4, agg_a, agg_b) = oracle_compare::levels::evaluate_within(
        &parsed.moves,
        &parsed.moves,
        &oracle_compare::levels::Scope::Layer(1),
        0.0,
    );
    assert_eq!(l3, LevelOutcome::Pass);
    assert_eq!(l4, LevelOutcome::Pass);
    assert_eq!(agg_a.move_count, 1);
    assert!(
        (agg_a.extruded_length - 5.0).abs() < 1e-9,
        "expected the real carried-forward length 5, got {} (20.6 would mean it was \
         measured from the origin instead of from layer 0's last move)",
        agg_a.extruded_length
    );
    assert_eq!(agg_a, agg_b, "same moves on both sides");
}
