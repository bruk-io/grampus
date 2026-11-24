//! Pins the text and JSON rendering contract that steps 6, 10 and 25 to 27's standalone Python
//! callers parse: the key names in `render_json`, the `none of L0, L1` wording in `render_text`,
//! and that `render_json` always emits valid JSON.

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
    let report = compare(b"G28\n", b"G28\n");
    let json = render_json(&report);
    assert_valid_json_object(&json);
    assert!(json.contains("\"levels\":"));
    assert!(json.contains("\"strongest_evaluated_level\":\"L1\""));
    assert!(json.contains("\"evaluated_levels\":[\"L0\",\"L1\"]"));
    assert!(json.contains("\"stricter_failure\":null"));
    assert!(json.contains("\"first_differing_byte\":null"));
    assert!(json.contains("\"tolerance\":\"exact\""));
}

#[test]
fn passing_report_text_names_the_strongest_evaluated_level() {
    let report = compare(b"G28\n", b"G28\n");
    let text = render_text(&report);
    assert!(text.contains("L0: pass\n"));
    assert!(text.contains("L1: pass\n"));
    assert!(text.contains("strongest evaluated level: L1\n"));
}

#[test]
fn no_pass_report_text_says_none_of_l0_l1_not_a_bare_none() {
    let report = compare(b"abc", b"xyz");
    let text = render_text(&report);
    assert!(text.contains("strongest evaluated level: none of L0, L1\n"));
    assert!(text.contains("first differing byte: 0\n"));
}

#[test]
fn no_pass_report_json_has_null_strongest_level_and_the_byte_offset() {
    let report = compare(b"abc", b"xyz");
    let json = render_json(&report);
    assert_valid_json_object(&json);
    assert!(json.contains("\"strongest_evaluated_level\":null"));
    assert!(json.contains("\"first_differing_byte\":0"));
}

#[test]
fn line_failure_marks_an_absent_side_as_null_not_an_empty_string() {
    // `a` normalises to ["G28", ""], `b` to ["G28"]: line 2 is present-but-empty on `a`, wholly
    // absent on `b`. `null` must be distinguishable from `""` in both renderings.
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
