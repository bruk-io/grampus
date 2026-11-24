//! The tolerance input every level above L1 needs. Every tolerance in this repository defaults
//! to exact (zero per-axis deviation, zero aggregate deviation); a looser one is read only from a
//! `port/manifest.tsv` row (`--tolerance-from-manifest <tsv> --row <path>`) or a `docs/ORACLE.md`
//! configuration row (`--tolerance-from-config <md> --configuration <name>`), and only with a
//! reason in that row's tolerance cell. Both readers find their columns by header name, so a
//! reordered or renamed column does not silently break them, and both fail named and loud rather
//! than falling back to exact on a malformed row.
//!
//! **Decision (task silent on the comparison function's name space):** the only comparison
//! function this crate implements is absolute deviation, named `abs` in a cell (an empty
//! comparison-function cell on a row that does declare a tolerance also means `abs`, since it is
//! the only one there is). `tests/parity.md` ("`|Δ| ≤ tol`" per axis) is the research note this
//! reads as confirming the shape; any other name in a `comparison function` cell is refused as
//! not implemented, which is the hook task `14-manifest-columns` or a future comparison function
//! can widen later.

use std::fmt;

/// The comparison functions this crate implements. `Abs` (absolute deviation, `|a - b| <=
/// tolerance`) is the only one; a `comparison function` cell naming anything else is refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComparisonFunction {
    Abs,
}

impl ComparisonFunction {
    fn parse(name: &str) -> Option<ComparisonFunction> {
        match name.trim().to_ascii_lowercase().as_str() {
            "abs" | "absolute" => Some(ComparisonFunction::Abs),
            _ => None,
        }
    }

    fn name(self) -> &'static str {
        match self {
            ComparisonFunction::Abs => "abs",
        }
    }
}

/// Where a declared tolerance came from: a manifest row or a configuration row, each identified
/// by the file it was read from and the row key the caller asked for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToleranceSource {
    Manifest {
        tsv_path: String,
        row: String,
    },
    Config {
        md_path: String,
        configuration: String,
    },
}

/// The tolerance a run uses: `Exact` (the repository-wide default, zero deviation) or a value
/// declared on a manifest or configuration row, which always carries the reason text from that
/// row's tolerance cell.
#[derive(Debug, Clone, PartialEq)]
pub enum Tolerance {
    Exact,
    Declared {
        value: f64,
        unit: Option<String>,
        reason: String,
        comparison_function: ComparisonFunction,
        source: ToleranceSource,
    },
}

impl Tolerance {
    /// The absolute deviation allowed, `0.0` for `Exact`. L3 and L4 read this; neither reads
    /// `unit` or `comparison_function` today, since `Abs` on a plain `f64` is the only function
    /// implemented and every fixture's unit is already the move model's own (millimetres).
    pub fn value(&self) -> f64 {
        match self {
            Tolerance::Exact => 0.0,
            Tolerance::Declared { value, .. } => *value,
        }
    }
}

impl fmt::Display for Tolerance {
    /// Renders as one string so `tolerance` stays a JSON string key (not an object) across this
    /// task's change — `format.rs` and the scripts in steps 6, 10 and 25-27 that parse this
    /// binary's JSON output read `tolerance` as a string today, and this task does not change
    /// that shape.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Tolerance::Exact => f.write_str("exact"),
            Tolerance::Declared {
                value,
                unit,
                reason,
                comparison_function,
                source,
            } => {
                let source_desc = match source {
                    ToleranceSource::Manifest { tsv_path, row } => {
                        format!("manifest row '{row}' in {tsv_path}")
                    }
                    ToleranceSource::Config {
                        md_path,
                        configuration,
                    } => format!("configuration '{configuration}' in {md_path}"),
                };
                let unit_suffix = unit.as_deref().map(|u| format!(" {u}")).unwrap_or_default();
                write!(
                    f,
                    "{value}{unit_suffix} ({}) from {source_desc}; reason: {reason}",
                    comparison_function.name()
                )
            }
        }
    }
}

/// Parses a tolerance cell of the shape `<value>[ <unit>]; reason: <text>`. `Ok(None)` for an
/// empty cell — no tolerance is declared for this row, which defaults to exact, silently, since
/// nothing invalid was written there. `Err` for a cell that carries a value with no `reason:`
/// clause, or with an empty one.
fn parse_tolerance_cell(cell: &str) -> Result<Option<(f64, Option<String>, String)>, String> {
    let cell = cell.trim();
    if cell.is_empty() {
        return Ok(None);
    }

    let semi = cell
        .find(';')
        .ok_or_else(|| format!("tolerance cell '{cell}' has a value with no reason"))?;
    let value_part = cell[..semi].trim();
    let after_semi = cell[semi + 1..].trim();
    let reason = after_semi
        .strip_prefix("reason:")
        .map(|r| r.trim())
        .filter(|r| !r.is_empty())
        .ok_or_else(|| format!("tolerance cell '{cell}' has a value with no reason"))?
        .to_string();

    let mut parts = value_part.split_whitespace();
    let value_str = parts
        .next()
        .ok_or_else(|| format!("tolerance cell '{cell}' has no value before the reason"))?;
    let value = value_str
        .parse::<f64>()
        .map_err(|e| format!("tolerance cell '{cell}': cannot parse value '{value_str}': {e}"))?;
    let unit = parts.next().map(|s| s.to_string());

    Ok(Some((value, unit, reason)))
}

/// Resolves the `comparison function` cell on its own, independent of whether the row's
/// `tolerance` cell carries a value. An empty cell means `abs` (the only function implemented),
/// whether or not a tolerance value is declared; a non-empty, unimplemented name is refused
/// unconditionally — the task asks to refuse "a comparison function the comparator does not
/// implement" without exempting the case where the row otherwise defaults to exact, and reading
/// it only inside the `Some` arm below (as an earlier version of this function did) let a row
/// with an empty tolerance cell and an unimplemented function name resolve to `Exact` silently.
fn parse_comparison_function(
    function_cell: &str,
    row_desc: &str,
) -> Result<ComparisonFunction, String> {
    let function_name = function_cell.trim();
    if function_name.is_empty() {
        return Ok(ComparisonFunction::Abs);
    }
    ComparisonFunction::parse(function_name).ok_or_else(|| {
        format!("comparison function '{function_name}' is not implemented (row '{row_desc}')")
    })
}

fn build_tolerance(
    tolerance_cell: &str,
    function_cell: &str,
    row_desc: &str,
    source: ToleranceSource,
) -> Result<Tolerance, String> {
    // Checked before branching on the tolerance cell's own shape, so a row with no declared
    // tolerance but an unimplemented comparison function is still refused rather than silently
    // resolving to `Exact`.
    let comparison_function = parse_comparison_function(function_cell, row_desc)?;

    match parse_tolerance_cell(tolerance_cell).map_err(|e| format!("{e} (row '{row_desc}')"))? {
        None => Ok(Tolerance::Exact),
        Some((value, unit, reason)) => Ok(Tolerance::Declared {
            value,
            unit,
            reason,
            comparison_function,
            source,
        }),
    }
}

/// Reads the tolerance for `row_path` (the libslic3r path a manifest row names) from a
/// tab-separated manifest at `tsv_path`, finding the `path`, `tolerance` and `comparison
/// function` columns by header name. A missing file, a manifest with no `path`, `tolerance` or
/// `comparison function` column, or no row whose `path` cell equals `row_path` each return a
/// named `Err` — the caller compares nothing on any of them.
pub fn from_manifest_row(tsv_path: &str, row_path: &str) -> Result<Tolerance, String> {
    let content = std::fs::read_to_string(tsv_path)
        .map_err(|e| format!("cannot read tolerance manifest '{tsv_path}': {e}"))?;
    let mut lines = content.lines();
    let header_line = lines
        .next()
        .ok_or_else(|| format!("tolerance manifest '{tsv_path}' is empty"))?;
    let headers: Vec<&str> = header_line.split('\t').collect();
    let find_col = |name: &str| headers.iter().position(|h| h.eq_ignore_ascii_case(name));

    let path_col = find_col("path")
        .ok_or_else(|| format!("tolerance manifest '{tsv_path}' has no 'path' column"))?;
    let (tolerance_col, function_col) =
        match (find_col("tolerance"), find_col("comparison function")) {
            (Some(t), Some(c)) => (t, c),
            _ => {
                return Err(format!(
                "tolerance manifest '{tsv_path}' has no 'tolerance' or 'comparison function' column"
            ))
            }
        };

    for line in lines {
        if line.is_empty() {
            continue;
        }
        let fields: Vec<&str> = line.split('\t').collect();
        if fields.get(path_col).copied().unwrap_or("") != row_path {
            continue;
        }
        let tolerance_cell = fields.get(tolerance_col).copied().unwrap_or("");
        let function_cell = fields.get(function_col).copied().unwrap_or("");
        return build_tolerance(
            tolerance_cell,
            function_cell,
            row_path,
            ToleranceSource::Manifest {
                tsv_path: tsv_path.to_string(),
                row: row_path.to_string(),
            },
        );
    }

    Err(format!(
        "no row for path '{row_path}' in tolerance manifest '{tsv_path}'"
    ))
}

/// One parsed Markdown pipe table: its header cells and its data rows, both trimmed.
struct MdTable {
    headers: Vec<String>,
    rows: Vec<Vec<String>>,
}

fn split_md_row(line: &str) -> Vec<String> {
    let trimmed = line.trim().trim_start_matches('|').trim_end_matches('|');
    trimmed.split('|').map(|c| c.trim().to_string()).collect()
}

/// A GFM table separator row: only `|`, `-`, `:` and spaces.
fn is_separator_row(line: &str) -> bool {
    let line = line.trim();
    line.starts_with('|') && line.chars().all(|c| matches!(c, '|' | '-' | ':' | ' '))
}

/// Finds every Markdown pipe table in `content`: a header row, a GFM separator row, then data
/// rows until a line that is not a table row.
fn parse_md_tables(content: &str) -> Vec<MdTable> {
    let lines: Vec<&str> = content.lines().collect();
    let mut tables = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        let header_line = lines[i].trim();
        if header_line.starts_with('|') && i + 1 < lines.len() && is_separator_row(lines[i + 1]) {
            let headers = split_md_row(header_line);
            let mut rows = Vec::new();
            let mut j = i + 2;
            while j < lines.len() && lines[j].trim().starts_with('|') {
                rows.push(split_md_row(lines[j]));
                j += 1;
            }
            tables.push(MdTable { headers, rows });
            i = j;
        } else {
            i += 1;
        }
    }
    tables
}

/// Reads the tolerance for `configuration` from the `docs/ORACLE.md`-shaped configuration table
/// at `md_path`, finding the table with both a `tolerance` and a `comparison function` column
/// (by header name — there may be other tables in the same document with neither), then that
/// table's `configuration` column by header name. A missing file, a document with no table
/// carrying both columns — true of the real `docs/ORACLE.md` until `04-oracle-slice` adds them —
/// or no row whose `configuration` cell equals `configuration` each return a named `Err`.
pub fn from_config_row(md_path: &str, configuration: &str) -> Result<Tolerance, String> {
    let content = std::fs::read_to_string(md_path)
        .map_err(|e| format!("cannot read tolerance configuration table '{md_path}': {e}"))?;
    let tables = parse_md_tables(&content);

    let find_col =
        |headers: &[String], name: &str| headers.iter().position(|h| h.eq_ignore_ascii_case(name));

    let table = tables.iter().find(|t| {
        find_col(&t.headers, "tolerance").is_some()
            && find_col(&t.headers, "comparison function").is_some()
    });
    let table = table.ok_or_else(|| {
        format!(
            "tolerance configuration table '{md_path}' has no 'tolerance' or 'comparison function' column"
        )
    })?;

    let config_col = find_col(&table.headers, "configuration").ok_or_else(|| {
        format!("tolerance configuration table '{md_path}' has no 'configuration' column")
    })?;
    let tolerance_col = find_col(&table.headers, "tolerance").expect("checked above");
    let function_col = find_col(&table.headers, "comparison function").expect("checked above");

    for row in &table.rows {
        if row.get(config_col).map(|s| s.as_str()) != Some(configuration) {
            continue;
        }
        let tolerance_cell = row.get(tolerance_col).map(|s| s.as_str()).unwrap_or("");
        let function_cell = row.get(function_col).map(|s| s.as_str()).unwrap_or("");
        return build_tolerance(
            tolerance_cell,
            function_cell,
            configuration,
            ToleranceSource::Config {
                md_path: md_path.to_string(),
                configuration: configuration.to_string(),
            },
        );
    }

    Err(format!(
        "no configuration '{configuration}' in tolerance configuration table '{md_path}'"
    ))
}
