//! Text and JSON rendering of a [`crate::Report`]. JSON uses stable key names because the oracle
//! scripts of steps 6, 10 and 25 to 27 are standalone Python executables that call this binary
//! and parse its output, not its prose.

use crate::{FailLocation, Level, LevelOutcome, Report};

fn lossy(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

/// Renders a `FailLocation::Line` side for text output: the line's text when the normalised
/// stream has a line at that number, or a marker distinct from an actual empty line when it has
/// none (one side's normalised stream is a strict prefix of the other's).
fn side_text(side: &Option<Vec<u8>>) -> String {
    match side {
        Some(bytes) => lossy(bytes),
        None => "<no line>".to_string(),
    }
}

/// Renders a `FailLocation::Line` side for JSON: the line's text, or `null` when the normalised
/// stream has no line at that number at all.
fn side_json(side: &Option<Vec<u8>>) -> String {
    match side {
        Some(bytes) => json_escape(&lossy(bytes)),
        None => "null".to_string(),
    }
}

fn outcome_text(outcome: &LevelOutcome) -> String {
    match outcome {
        LevelOutcome::Pass => "pass".to_string(),
        LevelOutcome::NotEvaluated => "not evaluated".to_string(),
        LevelOutcome::Fail(FailLocation::ByteOffset(off)) => {
            format!("fail at byte offset {off}")
        }
        LevelOutcome::Fail(FailLocation::Line { number, a, b }) => {
            format!(
                "fail at line {number}\n    a: {}\n    b: {}",
                side_text(a),
                side_text(b)
            )
        }
    }
}

fn evaluated_list_text(report: &Report) -> String {
    report
        .evaluated_levels()
        .iter()
        .map(|l| l.name())
        .collect::<Vec<_>>()
        .join(", ")
}

/// One line per level with its outcome, the strongest evaluated level, the first failing
/// location at the stricter level immediately beyond it, the tolerance used, and, when no
/// evaluated level passed, the first differing byte.
pub fn render_text(report: &Report) -> String {
    let mut out = String::new();
    for level in Level::ALL {
        out.push_str(level.name());
        out.push_str(": ");
        out.push_str(&outcome_text(report.outcome(level)));
        out.push('\n');
    }

    match report.strongest_evaluated_pass {
        Some(level) => {
            out.push_str(&format!("strongest evaluated level: {level}\n"));
        }
        None => {
            out.push_str(&format!(
                "strongest evaluated level: none of {}\n",
                evaluated_list_text(report)
            ));
        }
    }

    if let Some((level, loc)) = &report.stricter_failure {
        out.push_str(&format!(
            "stricter level {level} failed: {}\n",
            outcome_text(&LevelOutcome::Fail(loc.clone()))
        ));
    }

    if let Some(off) = report.first_differing_byte {
        out.push_str(&format!("first differing byte: {off}\n"));
    }

    out.push_str(&format!("tolerance: {}\n", report.tolerance));
    out
}

fn json_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

fn outcome_json(outcome: &LevelOutcome) -> String {
    match outcome {
        LevelOutcome::Pass => "{\"outcome\":\"pass\"}".to_string(),
        LevelOutcome::NotEvaluated => "{\"outcome\":\"not_evaluated\"}".to_string(),
        LevelOutcome::Fail(FailLocation::ByteOffset(off)) => {
            format!("{{\"outcome\":\"fail\",\"byte_offset\":{off}}}")
        }
        LevelOutcome::Fail(FailLocation::Line { number, a, b }) => {
            format!(
                "{{\"outcome\":\"fail\",\"line\":{number},\"a\":{},\"b\":{}}}",
                side_json(a),
                side_json(b)
            )
        }
    }
}

/// One JSON object with the same items `render_text` prints, under stable key names:
/// `levels` (an object keyed `"L0"`..`"L4"`), `strongest_evaluated_level` (a level name or
/// `null`), `evaluated_levels` (array of level names), `stricter_failure` (`null` or
/// `{"level": ..., ...the failing level's own shape}`), `first_differing_byte` (`null` or a
/// number), and `tolerance`. An L1 line failure's `"a"`/`"b"` are each a string, or `null` when
/// that side's normalised stream has no line at that number at all (one side is a strict prefix
/// of the other) — distinct from `""`, which means the line is present and empty.
pub fn render_json(report: &Report) -> String {
    let mut levels_json = String::from("{");
    for (i, level) in Level::ALL.into_iter().enumerate() {
        if i > 0 {
            levels_json.push(',');
        }
        levels_json.push_str(&json_escape(level.name()));
        levels_json.push(':');
        levels_json.push_str(&outcome_json(report.outcome(level)));
    }
    levels_json.push('}');

    let strongest = match report.strongest_evaluated_pass {
        Some(level) => json_escape(level.name()),
        None => "null".to_string(),
    };

    let evaluated = format!(
        "[{}]",
        report
            .evaluated_levels()
            .iter()
            .map(|l| json_escape(l.name()))
            .collect::<Vec<_>>()
            .join(",")
    );

    let stricter_failure = match &report.stricter_failure {
        Some((level, loc)) => format!(
            "{{\"level\":{},\"failure\":{}}}",
            json_escape(level.name()),
            outcome_json(&LevelOutcome::Fail(loc.clone()))
        ),
        None => "null".to_string(),
    };

    let first_differing_byte = match report.first_differing_byte {
        Some(off) => off.to_string(),
        None => "null".to_string(),
    };

    format!(
        "{{\"levels\":{levels_json},\"strongest_evaluated_level\":{strongest},\"evaluated_levels\":{evaluated},\"stricter_failure\":{stricter_failure},\"first_differing_byte\":{first_differing_byte},\"tolerance\":{}}}",
        json_escape(&report.tolerance.to_string())
    )
}
