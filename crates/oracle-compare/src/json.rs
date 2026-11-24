//! JSON artifact comparison: part 3 of plan step 5 (R4) — the oracle's `--export-settings` dump
//! (`settings.json`) and its `--export-statistics` dump (`plate_N.statistics.json`, named by
//! replacing the G-code's extension with `.statistics.json`,
//! `vendor/OrcaSlicer/src/libslic3r/GCode/GCodeStatistics.cpp:185-188`). Parts 1 and 2 of this
//! step (`levels.rs`/`gcode.rs`) cover the G-code artifact's five levels; this module adds
//! neither a G-code level nor a numeric allowance of any kind — both rules below are exact, each
//! with exactly one *structural* exception named by R4 itself, never a value within some margin.
//!
//! A JSON artifact is reported only as [`crate::Level::L0`] (identical bytes), [`crate::Level::L1`]
//! (identical after that artifact's own declared exception), or `none` (represented as
//! [`JsonReport::level`] being `None`) — L2, L3 and L4 are G-code-only and are never produced
//! here. `none` carries the first JSON Pointer (RFC 6901) at which the two parsed documents
//! disagree and both values there, or, when a side cannot be parsed as JSON at all (or, in
//! principle, when the raw bytes differ with no allowance in effect yet no structural difference
//! can be found — whitespace or key-order only — which this oracle's deterministic writer has
//! never been observed to produce), the first raw differing byte. A malformed, empty or
//! unreadable *file* is this binary's named-error exit (handled in `main.rs`, before this module
//! ever sees the bytes); a file that is readable but is not JSON is a measurement about the two
//! artifacts and is reported here as `none`, not as an error.

use serde_json::Value;

use crate::Level;

/// Where a JSON artifact's comparison landed when it is neither [`Level::L0`] nor [`Level::L1`].
#[derive(Debug, Clone, PartialEq)]
pub enum JsonDifference {
    /// The first JSON Pointer (RFC 6901) at which the two parsed documents disagree, with that
    /// pointer's value on each side. `None` on a side means the pointer exists on the other side
    /// only — a key present in one document and absent in the other, or an array one element
    /// shorter than the other's.
    Pointer {
        pointer: String,
        a: Option<Value>,
        b: Option<Value>,
    },
    /// No JSON Pointer could be named: either a side is not parseable JSON at all, or the parsed
    /// documents agree everywhere the artifact's own allowance does not already excuse, yet the
    /// raw bytes still differ (whitespace or key order — this oracle's writer has never been
    /// observed to do this, but the path still must not panic or invent a pointer). Either way,
    /// this is the first byte at which the two raw inputs differ — the same computation
    /// [`crate::Level::L0`] uses — `None` only when the two byte strings are themselves
    /// identical, a case [`compare_settings`]/[`compare_statistics`] never reach this far for,
    /// since that is `Level::L0` and both return before parsing.
    NoPointer { first_differing_byte: Option<usize> },
}

/// One JSON artifact's comparison result: [`Level::L0`]/[`Level::L1`] reached, or R4's `none`
/// with the first difference. An enum rather than an `Option<Level>` paired with an
/// `Option<JsonDifference>`, so there is no state where both or neither are present for a renderer
/// to fall over on.
#[derive(Debug, Clone, PartialEq)]
pub enum JsonOutcome {
    Level(Level),
    None(JsonDifference),
}

/// One JSON artifact's comparison result.
#[derive(Debug, Clone, PartialEq)]
pub struct JsonReport {
    pub outcome: JsonOutcome,
}

impl JsonReport {
    fn l0() -> Self {
        JsonReport {
            outcome: JsonOutcome::Level(Level::L0),
        }
    }

    fn l1() -> Self {
        JsonReport {
            outcome: JsonOutcome::Level(Level::L1),
        }
    }

    fn pointer(pointer: String, a: Option<Value>, b: Option<Value>) -> Self {
        JsonReport {
            outcome: JsonOutcome::None(JsonDifference::Pointer { pointer, a, b }),
        }
    }

    fn no_pointer(a: &[u8], b: &[u8]) -> Self {
        JsonReport {
            outcome: JsonOutcome::None(JsonDifference::NoPointer {
                first_differing_byte: crate::levels::first_difference(a, b),
            }),
        }
    }

    /// Whether this report meets `level`, for `--require`: [`Level::L0`] only when the pair was
    /// byte-identical, [`Level::L1`] when it was either that or identical after the artifact's
    /// own allowance (`L0` implies `L1` here — unlike the five independently evaluated G-code
    /// levels, a JSON artifact has exactly one outcome, not five), and never for `L2`/`L3`/`L4`,
    /// which this module never produces.
    pub fn passed(&self, level: Level) -> bool {
        match (level, &self.outcome) {
            (Level::L0, JsonOutcome::Level(Level::L0)) => true,
            (Level::L0, _) => false,
            (Level::L1, JsonOutcome::Level(Level::L0 | Level::L1)) => true,
            (Level::L1, _) => false,
            (Level::L2 | Level::L3 | Level::L4, _) => false,
        }
    }
}

/// Parses both sides as JSON. `None` when either side fails to parse — malformed, empty, or
/// truncated input never panics here and is never treated as this binary's named-error exit (an
/// *unreadable* file is; see this module's doc comment): `serde_json::from_slice` reports a parse
/// failure as `Err`, which this function only ever turns into `None`, and never `.unwrap()`s or
/// `.expect()`s either `Result`.
fn parse_both(a: &[u8], b: &[u8]) -> Option<(Value, Value)> {
    let va = serde_json::from_slice(a).ok()?;
    let vb = serde_json::from_slice(b).ok()?;
    Some((va, vb))
}

/// Escapes one JSON Pointer reference token per RFC 6901 §3: `~` to `~0` first, then `/` to
/// `~1` — reversing the order would double-escape a literal `~1` that was already in the key.
fn escape_pointer_token(token: &str) -> String {
    token.replace('~', "~0").replace('/', "~1")
}

/// Walks `a` and `b` together in a stable order — object keys sorted lexically (`serde_json`'s
/// default `Map` is a `BTreeMap`; this crate never enables the `preserve_order` feature, so this
/// is the type's own iteration order, not an extra sort this function imposes) and array indices
/// ascending — and returns the first JSON Pointer (RFC 6901) at which they disagree, together
/// with that pointer's value on each side (`None` on a side when the pointer exists only on the
/// other). `skip` names pointers this walk must treat as always equal without recursing into them
/// at all — the one declared allowance each artifact gets — so a value present only inside a
/// skipped pointer, or of a different shape there, is never reported. `None` means every value
/// compared equal except what `skip` excluded.
fn first_difference<'v>(
    a: &'v Value,
    b: &'v Value,
    pointer: &str,
    skip: &dyn Fn(&str) -> bool,
) -> Option<(String, Option<&'v Value>, Option<&'v Value>)> {
    if skip(pointer) {
        return None;
    }
    match (a, b) {
        (Value::Object(ma), Value::Object(mb)) => {
            let mut keys: Vec<&String> = ma.keys().chain(mb.keys()).collect();
            keys.sort();
            keys.dedup();
            for key in keys {
                let child = format!("{pointer}/{}", escape_pointer_token(key));
                if skip(&child) {
                    continue;
                }
                match (ma.get(key), mb.get(key)) {
                    (Some(va), Some(vb)) => {
                        if let Some(diff) = first_difference(va, vb, &child, skip) {
                            return Some(diff);
                        }
                    }
                    (va, vb) => return Some((child, va, vb)),
                }
            }
            None
        }
        (Value::Array(aa), Value::Array(ab)) => {
            for i in 0..aa.len().max(ab.len()) {
                let child = format!("{pointer}/{i}");
                if skip(&child) {
                    continue;
                }
                match (aa.get(i), ab.get(i)) {
                    (Some(va), Some(vb)) => {
                        if let Some(diff) = first_difference(va, vb, &child, skip) {
                            return Some(diff);
                        }
                    }
                    (va, vb) => return Some((child, va, vb)),
                }
            }
            None
        }
        (va, vb) if va == vb => None,
        (va, vb) => Some((pointer.to_string(), Some(va), Some(vb))),
    }
}

/// Compares two `--export-settings` dumps (`settings.json`): byte for byte with no normalisation.
/// The one declared allowance — the `slicing_pipeline_plugin` key may differ — applies only when
/// `allow_pipeline_plugin_diff` is `true`, because arming the slicing-pipeline hook requires that
/// option to be a non-empty `ConfigOptionStrings`
/// (`vendor/OrcaSlicer/src/libslic3r/Print.cpp:3306-3310`). The allowance is off by default
/// (`false`): a caller compares an armed run against an unarmed one by passing `true` only for
/// that pair, never as a standing default.
pub fn compare_settings(a: &[u8], b: &[u8], allow_pipeline_plugin_diff: bool) -> JsonReport {
    if a == b {
        return JsonReport::l0();
    }
    let Some((va, vb)) = parse_both(a, b) else {
        return JsonReport::no_pointer(a, b);
    };
    let skip = |pointer: &str| allow_pipeline_plugin_diff && pointer == "/slicing_pipeline_plugin";
    match first_difference(&va, &vb, "", &skip) {
        None if allow_pipeline_plugin_diff => JsonReport::l1(),
        // The allowance is off, so there is no normalisation step declared for this artifact at
        // all: a structurally-equal pair whose raw bytes still differ has no L1 to report and no
        // pointer to name it by, so it falls back to the first raw differing byte, like
        // unparseable input.
        None => JsonReport::no_pointer(a, b),
        Some((pointer, found_a, found_b)) => {
            JsonReport::pointer(pointer, found_a.cloned(), found_b.cloned())
        }
    }
}

/// Compares two `--export-statistics` dumps (`plate_N.statistics.json`): every key compared,
/// with exactly one field normalised, `generator`, which the oracle writes as the static
/// `std::string(SLIC3R_APP_NAME) + " " + SoftFever_VERSION`
/// (`vendor/OrcaSlicer/src/libslic3r/GCode/GCodeStatistics.cpp:177`) and which the port cannot
/// reproduce without claiming to be OrcaSlicer. `version` and the `summary`, `units`, `layers`
/// and `objects` objects are all compared like every other key — only `generator` is excused.
pub fn compare_statistics(a: &[u8], b: &[u8]) -> JsonReport {
    if a == b {
        return JsonReport::l0();
    }
    let Some((va, vb)) = parse_both(a, b) else {
        return JsonReport::no_pointer(a, b);
    };
    let skip = |pointer: &str| pointer == "/generator";
    match first_difference(&va, &vb, "", &skip) {
        None => JsonReport::l1(),
        Some((pointer, found_a, found_b)) => {
            JsonReport::pointer(pointer, found_a.cloned(), found_b.cloned())
        }
    }
}

fn value_text(value: &Option<Value>) -> String {
    match value {
        Some(v) => v.to_string(),
        None => "<absent>".to_string(),
    }
}

fn value_json(value: &Option<Value>) -> String {
    match value {
        Some(v) => v.to_string(),
        None => "null".to_string(),
    }
}

/// One line naming the level reached and, at `none`, the first difference — the text `--format
/// text` (the default) prints for `--settings`/`--statistics`.
pub fn render_text(report: &JsonReport) -> String {
    match &report.outcome {
        JsonOutcome::Level(level) => format!("{level}: pass\n"),
        JsonOutcome::None(JsonDifference::Pointer { pointer, a, b }) => {
            format!(
                "none\n  first difference at {pointer}\n    a: {}\n    b: {}\n",
                value_text(a),
                value_text(b)
            )
        }
        JsonOutcome::None(JsonDifference::NoPointer {
            first_differing_byte,
        }) => match first_differing_byte {
            Some(off) => format!("none\n  not parseable as JSON; first differing byte: {off}\n"),
            None => "none\n  not parseable as JSON\n".to_string(),
        },
    }
}

/// The same report as one stable-keyed JSON object, for the scripts of steps 6, 10, 25-27 that
/// parse this binary's output rather than its prose (see `format.rs`'s own doc comment, which
/// states the same rule for the G-code report).
pub fn render_json(report: &JsonReport) -> String {
    let (level, difference) = match &report.outcome {
        JsonOutcome::Level(level) => (format!("\"{level}\""), "null".to_string()),
        JsonOutcome::None(JsonDifference::Pointer { pointer, a, b }) => (
            "null".to_string(),
            format!(
                "{{\"kind\":\"pointer\",\"pointer\":{},\"a\":{},\"b\":{}}}",
                serde_json::to_string(pointer).unwrap_or_else(|_| "\"\"".to_string()),
                value_json(a),
                value_json(b)
            ),
        ),
        JsonOutcome::None(JsonDifference::NoPointer {
            first_differing_byte,
        }) => (
            "null".to_string(),
            format!(
                "{{\"kind\":\"no_pointer\",\"first_differing_byte\":{}}}",
                first_differing_byte
                    .map(|n| n.to_string())
                    .unwrap_or_else(|| "null".to_string())
            ),
        ),
    };
    format!("{{\"level\":{level},\"difference\":{difference}}}")
}
