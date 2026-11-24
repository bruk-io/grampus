//! Text and JSON rendering of a [`crate::Report`]. JSON uses stable key names because the oracle
//! scripts of steps 6, 10 and 25 to 27 are standalone Python executables that call this binary
//! and parse its output, not its prose.
//!
//! Every level is evaluated by task `09-comparator-gcode-levels` onward, so no line here ever
//! renders [`LevelOutcome::NotEvaluated`], the text `"strongest evaluated"` or `"none of ..."` —
//! the variant and the `evaluated_levels` helper stay on [`Report`] for API compatibility (a
//! future level could still decline to run), but `compare`/`compare_with_tolerance` never
//! produce them today.

use crate::{AggregateGroup, FailLocation, Level, LevelOutcome, Report, Side};

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

fn opt_text(side: &Option<String>) -> String {
    match side {
        Some(s) => s.clone(),
        None => "<no line>".to_string(),
    }
}

fn opt_json(side: &Option<String>) -> String {
    match side {
        Some(s) => json_escape(s),
        None => "null".to_string(),
    }
}

fn opt_usize_json(v: Option<usize>) -> String {
    match v {
        Some(n) => n.to_string(),
        None => "null".to_string(),
    }
}

fn localisation_text(
    layer: &Option<usize>,
    role: &Option<String>,
    object: &Option<String>,
) -> String {
    format!(
        "layer={}, role={}, object={}",
        layer
            .map(|n| n.to_string())
            .unwrap_or_else(|| "none".to_string()),
        role.as_deref().unwrap_or("none"),
        object.as_deref().unwrap_or("none"),
    )
}

fn side_name(side: Side) -> &'static str {
    match side {
        Side::A => "a",
        Side::B => "b",
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
        LevelOutcome::Fail(FailLocation::Token {
            index,
            line_a,
            line_b,
            text_a,
            text_b,
        }) => {
            format!(
                "fail at token {index}\n    a: line {} {}\n    b: line {} {}",
                line_a
                    .map(|n| n.to_string())
                    .unwrap_or_else(|| "<none>".to_string()),
                opt_text(text_a),
                line_b
                    .map(|n| n.to_string())
                    .unwrap_or_else(|| "<none>".to_string()),
                opt_text(text_b),
            )
        }
        LevelOutcome::Fail(FailLocation::Move {
            index,
            layer,
            role,
            object,
            count_a,
            count_b,
        }) => {
            format!(
                "fail at move {index} ({})\n    move count: a={count_a} b={count_b}",
                localisation_text(layer, role, object)
            )
        }
        LevelOutcome::Fail(FailLocation::Aggregate {
            group,
            quantity,
            a,
            b,
            tolerance,
        }) => {
            format!("fail: {group}'s {quantity} differs (a={a} b={b}, tolerance={tolerance})")
        }
        LevelOutcome::Fail(FailLocation::LayerCount {
            group,
            count_a,
            count_b,
            differing_layer,
            present_in,
        }) => {
            format!(
                "fail: {group}'s layer count differs (a={count_a} b={count_b}); layer {differing_layer} is present only in {}",
                side_name(*present_in)
            )
        }
    }
}

/// One line per level with its outcome, the strongest evaluated level, the first failing
/// location at the stricter level immediately beyond it, the L3 per-axis deviations, the L4
/// aggregates, the tolerance used, and, when no evaluated level passed, the first differing byte.
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
            out.push_str(&format!("strongest level: {level}\n"));
        }
        None => {
            out.push_str("strongest level: none\n");
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

    out.push_str("L3 axis deviations:\n");
    if report.l3_axis_deviations.is_empty() {
        out.push_str("  (no axis compared on both sides)\n");
    }
    for (axis, dev) in &report.l3_axis_deviations {
        out.push_str(&format!("  {axis}: max={} rms={}\n", dev.max, dev.rms));
    }

    out.push_str("L4 aggregates:\n");
    out.push_str("  whole file:\n");
    push_aggregate_pair_text(
        &mut out,
        "    ",
        &report.l4_aggregates_a.whole,
        &report.l4_aggregates_b.whole,
        // The whole-file group's layer count is the parser's raw layer-boundary count — the
        // number `evaluate_l4` actually compares for this group — not the moves-derived
        // `Aggregate::layer_count()`, which can under-count a trailing boundary with no move
        // after it (see `tests/data/layer_count_mismatch/`).
        Some((report.layer_boundaries_a, report.layer_boundaries_b)),
    );
    for layer in report
        .l4_aggregates_a
        .by_layer
        .keys()
        .chain(report.l4_aggregates_b.by_layer.keys())
        .collect::<std::collections::BTreeSet<_>>()
    {
        out.push_str(&format!("  layer {layer}:\n"));
        let empty = crate::Aggregate::default();
        let a = report.l4_aggregates_a.by_layer.get(layer).unwrap_or(&empty);
        let b = report.l4_aggregates_b.by_layer.get(layer).unwrap_or(&empty);
        push_aggregate_pair_text(&mut out, "    ", a, b, None);
    }
    for role in report
        .l4_aggregates_a
        .by_role
        .keys()
        .chain(report.l4_aggregates_b.by_role.keys())
        .collect::<std::collections::BTreeSet<_>>()
    {
        out.push_str(&format!("  role {role}:\n"));
        let empty = crate::Aggregate::default();
        let a = report.l4_aggregates_a.by_role.get(role).unwrap_or(&empty);
        let b = report.l4_aggregates_b.by_role.get(role).unwrap_or(&empty);
        push_aggregate_pair_text(&mut out, "    ", a, b, None);
    }
    for key in report
        .l4_aggregates_a
        .by_layer_role
        .keys()
        .chain(report.l4_aggregates_b.by_layer_role.keys())
        .collect::<std::collections::BTreeSet<_>>()
    {
        out.push_str(&format!("  layer {}, role {}:\n", key.0, key.1));
        let empty = crate::Aggregate::default();
        let a = report
            .l4_aggregates_a
            .by_layer_role
            .get(key)
            .unwrap_or(&empty);
        let b = report
            .l4_aggregates_b
            .by_layer_role
            .get(key)
            .unwrap_or(&empty);
        push_aggregate_pair_text(&mut out, "    ", a, b, None);
    }

    out.push_str(&format!("tolerance: {}\n", report.tolerance));
    out
}

/// `layer_count_override`, when given, is `(a, b)` printed in place of `a.layer_count()` /
/// `b.layer_count()` — used only for the whole-file group, whose layer count `evaluate_l4` reads
/// from [`crate::Report::layer_boundaries_a`]/`_b` rather than from the aggregate (see the call
/// site).
fn push_aggregate_pair_text(
    out: &mut String,
    indent: &str,
    a: &crate::Aggregate,
    b: &crate::Aggregate,
    layer_count_override: Option<(usize, usize)>,
) {
    out.push_str(&format!(
        "{indent}move count: a={} b={}\n",
        a.move_count, b.move_count
    ));
    out.push_str(&format!(
        "{indent}extruded length: a={} b={}\n",
        a.extruded_length, b.extruded_length
    ));
    out.push_str(&format!(
        "{indent}E delta: a={} b={}\n",
        a.e_delta, b.e_delta
    ));
    out.push_str(&format!(
        "{indent}travel distance: a={} b={}\n",
        a.travel_distance, b.travel_distance
    ));
    out.push_str(&format!(
        "{indent}bounding box: a={} b={}\n",
        bbox_text(a.bbox),
        bbox_text(b.bbox)
    ));
    let (layer_count_a, layer_count_b) =
        layer_count_override.unwrap_or((a.layer_count(), b.layer_count()));
    out.push_str(&format!(
        "{indent}layer count: a={layer_count_a} b={layer_count_b}\n"
    ));
}

fn bbox_text(bbox: Option<(f64, f64, f64, f64)>) -> String {
    match bbox {
        Some((min_x, max_x, min_y, max_y)) => format!("({min_x}..{max_x}, {min_y}..{max_y})"),
        None => "(none)".to_string(),
    }
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

fn group_json(group: &AggregateGroup) -> String {
    match group {
        AggregateGroup::WholeFile => "{\"kind\":\"whole_file\"}".to_string(),
        AggregateGroup::Layer(n) => format!("{{\"kind\":\"layer\",\"layer\":{n}}}"),
        AggregateGroup::Role(role) => {
            format!("{{\"kind\":\"role\",\"role\":{}}}", json_escape(role))
        }
        AggregateGroup::LayerRole(n, role) => {
            format!(
                "{{\"kind\":\"layer_role\",\"layer\":{n},\"role\":{}}}",
                json_escape(role)
            )
        }
        AggregateGroup::Object(name) => {
            format!("{{\"kind\":\"object\",\"object\":{}}}", json_escape(name))
        }
    }
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
        LevelOutcome::Fail(FailLocation::Token {
            index,
            line_a,
            line_b,
            text_a,
            text_b,
        }) => {
            format!(
                "{{\"outcome\":\"fail\",\"token_index\":{index},\"line_a\":{},\"line_b\":{},\"a\":{},\"b\":{}}}",
                opt_usize_json(*line_a),
                opt_usize_json(*line_b),
                opt_json(text_a),
                opt_json(text_b),
            )
        }
        LevelOutcome::Fail(FailLocation::Move {
            index,
            layer,
            role,
            object,
            count_a,
            count_b,
        }) => {
            format!(
                "{{\"outcome\":\"fail\",\"move_index\":{index},\"layer\":{},\"role\":{},\"object\":{},\"count_a\":{count_a},\"count_b\":{count_b}}}",
                opt_usize_json(*layer),
                opt_json(role),
                opt_json(object),
            )
        }
        LevelOutcome::Fail(FailLocation::Aggregate {
            group,
            quantity,
            a,
            b,
            tolerance,
        }) => {
            format!(
                "{{\"outcome\":\"fail\",\"group\":{},\"quantity\":{},\"a\":{a},\"b\":{b},\"tolerance\":{tolerance}}}",
                group_json(group),
                json_escape(quantity),
            )
        }
        LevelOutcome::Fail(FailLocation::LayerCount {
            group,
            count_a,
            count_b,
            differing_layer,
            present_in,
        }) => {
            format!(
                "{{\"outcome\":\"fail\",\"group\":{},\"count_a\":{count_a},\"count_b\":{count_b},\"differing_layer\":{differing_layer},\"present_in\":{}}}",
                group_json(group),
                json_escape(side_name(*present_in)),
            )
        }
    }
}

/// `layer_count_override`, when given, replaces `agg.layer_count()` — see `push_aggregate_pair_text`'s
/// doc comment; the same whole-file exception applies here.
fn aggregate_json(agg: &crate::Aggregate, layer_count_override: Option<usize>) -> String {
    let bbox = match agg.bbox {
        Some((min_x, max_x, min_y, max_y)) => {
            format!("{{\"min_x\":{min_x},\"max_x\":{max_x},\"min_y\":{min_y},\"max_y\":{max_y}}}")
        }
        None => "null".to_string(),
    };
    let layer_count = layer_count_override.unwrap_or_else(|| agg.layer_count());
    format!(
        "{{\"move_count\":{},\"extruded_length\":{},\"e_delta\":{},\"travel_distance\":{},\"bbox\":{bbox},\"layer_count\":{layer_count}}}",
        agg.move_count,
        agg.extruded_length,
        agg.e_delta,
        agg.travel_distance,
    )
}

fn aggregate_pair_json(
    a: &crate::Aggregate,
    b: &crate::Aggregate,
    layer_count_override: Option<(usize, usize)>,
) -> String {
    let (a_override, b_override) = match layer_count_override {
        Some((ac, bc)) => (Some(ac), Some(bc)),
        None => (None, None),
    };
    format!(
        "{{\"a\":{},\"b\":{}}}",
        aggregate_json(a, a_override),
        aggregate_json(b, b_override)
    )
}

fn l4_json(report: &Report) -> String {
    let mut by_layer = String::from("{");
    for (i, layer) in report
        .l4_aggregates_a
        .by_layer
        .keys()
        .chain(report.l4_aggregates_b.by_layer.keys())
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .enumerate()
    {
        if i > 0 {
            by_layer.push(',');
        }
        let empty = crate::Aggregate::default();
        let a = report.l4_aggregates_a.by_layer.get(layer).unwrap_or(&empty);
        let b = report.l4_aggregates_b.by_layer.get(layer).unwrap_or(&empty);
        by_layer.push_str(&format!("\"{layer}\":{}", aggregate_pair_json(a, b, None)));
    }
    by_layer.push('}');

    let mut by_role = String::from("{");
    for (i, role) in report
        .l4_aggregates_a
        .by_role
        .keys()
        .chain(report.l4_aggregates_b.by_role.keys())
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .enumerate()
    {
        if i > 0 {
            by_role.push(',');
        }
        let empty = crate::Aggregate::default();
        let a = report.l4_aggregates_a.by_role.get(role).unwrap_or(&empty);
        let b = report.l4_aggregates_b.by_role.get(role).unwrap_or(&empty);
        by_role.push_str(&format!(
            "{}:{}",
            json_escape(role),
            aggregate_pair_json(a, b, None)
        ));
    }
    by_role.push('}');

    let mut by_layer_role = String::from("{");
    for (i, key) in report
        .l4_aggregates_a
        .by_layer_role
        .keys()
        .chain(report.l4_aggregates_b.by_layer_role.keys())
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .enumerate()
    {
        if i > 0 {
            by_layer_role.push(',');
        }
        let empty = crate::Aggregate::default();
        let a = report
            .l4_aggregates_a
            .by_layer_role
            .get(key)
            .unwrap_or(&empty);
        let b = report
            .l4_aggregates_b
            .by_layer_role
            .get(key)
            .unwrap_or(&empty);
        // The key is `"<layer>:<role>"` as a single JSON string (object keys must be strings;
        // `group_json`'s `{"kind":"layer_role",...}` shape is for `stricter_failure`'s own
        // nested object, not usable as a map key here).
        by_layer_role.push_str(&format!(
            "{}:{}",
            json_escape(&format!("{}:{}", key.0, key.1)),
            aggregate_pair_json(a, b, None)
        ));
    }
    by_layer_role.push('}');

    format!(
        "{{\"whole_file\":{},\"by_layer\":{by_layer},\"by_role\":{by_role},\"by_layer_role\":{by_layer_role}}}",
        aggregate_pair_json(
            &report.l4_aggregates_a.whole,
            &report.l4_aggregates_b.whole,
            Some((report.layer_boundaries_a, report.layer_boundaries_b)),
        )
    )
}

fn l3_json(report: &Report) -> String {
    let mut out = String::from("{");
    for (i, (axis, dev)) in report.l3_axis_deviations.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        out.push_str(&format!(
            "\"{axis}\":{{\"max\":{},\"rms\":{}}}",
            dev.max, dev.rms
        ));
    }
    out.push('}');
    out
}

/// One JSON object with the same items `render_text` prints, under stable key names:
/// `levels` (an object keyed `"L0"`..`"L4"`), `strongest_level` (a level name or `null`),
/// `evaluated_levels` (array of level names), `stricter_failure` (`null` or `{"level": ...,
/// ...the failing level's own shape}`), `first_differing_byte` (`null` or a number),
/// `l3_axis_deviations` (an object keyed by axis letter), `l4_aggregates` (whole file, by layer,
/// by role and by `"<layer>:<role>"`, each `{"a": ..., "b": ...}`; the whole-file pair's
/// `layer_count` is `Report::layer_boundaries_a`/`_b`, not the aggregate's own
/// `Aggregate::layer_count()` — see `aggregate_json`), and `tolerance` (a string — see
/// [`crate::Tolerance`]'s `Display`, which this never diverges from).
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
        "{{\"levels\":{levels_json},\"strongest_level\":{strongest},\"evaluated_levels\":{evaluated},\"stricter_failure\":{stricter_failure},\"first_differing_byte\":{first_differing_byte},\"l3_axis_deviations\":{},\"l4_aggregates\":{},\"tolerance\":{}}}",
        l3_json(report),
        l4_json(report),
        json_escape(&report.tolerance.to_string())
    )
}
