//! Pins the text and JSON rendering contract that steps 6, 10 and 25 to 27's standalone Python
//! callers parse: the key names in `render_json`, that `render_text` never says "not evaluated",
//! "strongest evaluated" or "none of L0, L1" (task `09-comparator-gcode-levels` fills in L2-L4,
//! which removes those task-08 qualifiers for good), and that `render_json` always emits valid
//! JSON.

use oracle_compare::compare;
use oracle_compare::format::{render_json, render_text};

/// A tiny, dependency-free JSON reader good enough to assert this contract: it only needs to
/// confirm the top-level keys exist with the expected shape, not parse arbitrary JSON.
fn assert_valid_json_object(s: &str) {
    assert!(s.starts_with('{'), "expected a JSON object: {s}");
    assert!(s.ends_with('}'), "expected a JSON object: {s}");
    // Braces and brackets must balance; a crude but real check that nothing is truncated.
    let mut depth = 0i32;
    for c in s.chars() {
        match c {
            '{' | '[' => depth += 1,
            '}' | ']' => depth -= 1,
            _ => {}
        }
        assert!(depth >= 0, "unbalanced JSON in {s}");
    }
    assert_eq!(depth, 0, "unbalanced JSON in {s}");
}

#[test]
fn passing_report_json_has_the_documented_keys() {
    // No moves in either file, so L2 (comments dropped, no tokens from a comment-free "G28"
    // line's single command word, identical both sides), L3 and L4 (both move lists are empty,
    // vacuously equal) all pass too: every level passes on a byte-identical pair.
    let report = compare(b"G28\n", b"G28\n");
    let json = render_json(&report);
    assert_valid_json_object(&json);
    assert!(json.contains("\"levels\":"));
    assert!(json.contains("\"strongest_level\":\"L4\""));
    assert!(json.contains("\"evaluated_levels\":[\"L0\",\"L1\",\"L2\",\"L3\",\"L4\"]"));
    assert!(json.contains("\"stricter_failure\":null"));
    assert!(json.contains("\"first_differing_byte\":null"));
    assert!(json.contains("\"l3_axis_deviations\":"));
    assert!(json.contains("\"l4_aggregates\":"));
    assert!(json.contains("\"tolerance\":\"exact\""));
    assert!(
        !json.contains("not_evaluated"),
        "every level is evaluated from this task onward: {json}"
    );
}

#[test]
fn passing_report_text_names_the_strongest_level() {
    let report = compare(b"G28\n", b"G28\n");
    let text = render_text(&report);
    assert!(text.contains("L0: pass\n"));
    assert!(text.contains("L1: pass\n"));
    assert!(text.contains("L2: pass\n"));
    assert!(text.contains("L3: pass\n"));
    assert!(text.contains("L4: pass\n"));
    assert!(text.contains("strongest level: L4\n"));
    assert!(
        !text.contains("strongest evaluated"),
        "the phrase 'strongest evaluated' must not appear in any output line: {text}"
    );
}

#[test]
fn no_pass_report_text_says_a_bare_none_not_none_of_l0_l1() {
    // `a` opens with a move, `b` with an unrelated command: every level disagrees, so nothing
    // passes at all, not just L0 and L1.
    let report = compare(b"G1 X1 Y1 E1\n", b"M117 nope\n");
    let text = render_text(&report);
    assert!(text.contains("strongest level: none\n"));
    assert!(
        !text.contains("none of"),
        "the phrase 'none of ...' must not appear in any output line: {text}"
    );
    assert!(
        !text.contains("not evaluated"),
        "every level is evaluated from this task onward: {text}"
    );
    assert!(text.contains("first differing byte: 0\n"));
}

#[test]
fn no_pass_report_json_has_null_strongest_level_and_the_byte_offset() {
    let report = compare(b"G1 X1 Y1 E1\n", b"M117 nope\n");
    let json = render_json(&report);
    assert_valid_json_object(&json);
    assert!(json.contains("\"strongest_level\":null"));
    assert!(json.contains("\"first_differing_byte\":0"));
}

#[test]
fn line_failure_marks_an_absent_side_as_null_not_an_empty_string() {
    // `a` normalises to ["G28", ""], `b` to ["G28"]: line 2 is present-but-empty on `a`, wholly
    // absent on `b`. `null` must be distinguishable from `""` in both renderings. Neither file
    // has a move, so L1 is the one that actually fails here; L3/L4 pass vacuously.
    let report = compare(b"G28\n\n", b"G28\n");
    let json = render_json(&report);
    assert_valid_json_object(&json);
    assert!(
        json.contains("\"line\":2,\"a\":\"\",\"b\":null"),
        "expected a present-empty and b absent at line 2, got {json}"
    );

    let text = render_text(&report);
    assert!(text.contains("fail at line 2\n    a: \n    b: <no line>\n"));
}
