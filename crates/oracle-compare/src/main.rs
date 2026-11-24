//! `oracle-compare <a> <b> [--require <level>] [--format text|json]`
//! `               [--tolerance-from-manifest <tsv> --row <path>]`
//! `               [--tolerance-from-config <md> --configuration <name>]`
//!
//! Evaluates all five levels between two files, read as raw bytes. With neither tolerance flag
//! the comparator is exact; `--tolerance-from-manifest`/`--tolerance-from-config` read a looser
//! one from a manifest or configuration row (`tolerance.rs`).
//!
//! Exit status: 0 whenever the comparison ran, including when no evaluated level passed, unless
//! `--require <level>` names a level that did not pass, or there was a usage error, an unreadable
//! file, or a tolerance error (a missing file, no matching row, a missing column, a tolerance
//! value with no reason, or an unimplemented comparison function) — all of these exit non-zero
//! with a named error on stderr, and a tolerance error is checked before either G-code file is
//! read, so the run compares nothing.

use std::process::ExitCode;

use oracle_compare::{format, tolerance, Level, Tolerance};

struct Args {
    a: String,
    b: String,
    require: Option<Level>,
    format: OutputFormat,
    tolerance: ToleranceArgs,
}

enum ToleranceArgs {
    Exact,
    Manifest {
        tsv: String,
        row: Option<String>,
    },
    Config {
        md: String,
        configuration: Option<String>,
    },
}

#[derive(Clone, Copy)]
enum OutputFormat {
    Text,
    Json,
}

fn usage() -> &'static str {
    "usage: oracle-compare <a> <b> [--require <level>] [--format text|json]\n       \
     [--tolerance-from-manifest <tsv> --row <path>]\n       \
     [--tolerance-from-config <md> --configuration <name>]"
}

fn parse_args(raw: &[String]) -> Result<Args, String> {
    let mut positional: Vec<String> = Vec::new();
    let mut require: Option<Level> = None;
    let mut format = OutputFormat::Text;
    let mut manifest: Option<String> = None;
    let mut row: Option<String> = None;
    let mut config: Option<String> = None;
    let mut configuration: Option<String> = None;

    let mut i = 0;
    while i < raw.len() {
        match raw[i].as_str() {
            "--require" => {
                let value = raw
                    .get(i + 1)
                    .ok_or_else(|| "--require needs a level argument (L0..L4)".to_string())?;
                let level = Level::parse(value)
                    .ok_or_else(|| format!("--require: not a level: {value}"))?;
                require = Some(level);
                i += 2;
            }
            "--format" => {
                let value = raw
                    .get(i + 1)
                    .ok_or_else(|| "--format needs an argument (text or json)".to_string())?;
                format = match value.as_str() {
                    "text" => OutputFormat::Text,
                    "json" => OutputFormat::Json,
                    other => return Err(format!("--format: not text or json: {other}")),
                };
                i += 2;
            }
            "--tolerance-from-manifest" => {
                let value = raw
                    .get(i + 1)
                    .ok_or_else(|| "--tolerance-from-manifest needs a path".to_string())?;
                manifest = Some(value.clone());
                i += 2;
            }
            "--row" => {
                let value = raw
                    .get(i + 1)
                    .ok_or_else(|| "--row needs a path".to_string())?;
                row = Some(value.clone());
                i += 2;
            }
            "--tolerance-from-config" => {
                let value = raw
                    .get(i + 1)
                    .ok_or_else(|| "--tolerance-from-config needs a path".to_string())?;
                config = Some(value.clone());
                i += 2;
            }
            "--configuration" => {
                let value = raw
                    .get(i + 1)
                    .ok_or_else(|| "--configuration needs a name".to_string())?;
                configuration = Some(value.clone());
                i += 2;
            }
            other if other.starts_with("--") => {
                return Err(format!("unknown flag: {other}"));
            }
            other => {
                positional.push(other.to_string());
                i += 1;
            }
        }
    }

    if positional.len() != 2 {
        return Err(format!(
            "expected 2 positional arguments (a, b), got {}",
            positional.len()
        ));
    }

    let tolerance_args = match (manifest, config) {
        (Some(_), Some(_)) => {
            return Err(
                "--tolerance-from-manifest and --tolerance-from-config are mutually exclusive"
                    .to_string(),
            );
        }
        (Some(tsv), None) => {
            if configuration.is_some() {
                return Err(
                    "--configuration needs --tolerance-from-config, not --tolerance-from-manifest"
                        .to_string(),
                );
            }
            ToleranceArgs::Manifest { tsv, row }
        }
        (None, Some(md)) => {
            if row.is_some() {
                return Err(
                    "--row needs --tolerance-from-manifest, not --tolerance-from-config"
                        .to_string(),
                );
            }
            ToleranceArgs::Config { md, configuration }
        }
        (None, None) => {
            if row.is_some() {
                return Err("--row needs --tolerance-from-manifest".to_string());
            }
            if configuration.is_some() {
                return Err("--configuration needs --tolerance-from-config".to_string());
            }
            ToleranceArgs::Exact
        }
    };

    let b = positional.pop().expect("checked len == 2 above");
    let a = positional.pop().expect("checked len == 2 above");
    Ok(Args {
        a,
        b,
        require,
        format,
        tolerance: tolerance_args,
    })
}

fn resolve_tolerance(args: &ToleranceArgs) -> Result<Tolerance, String> {
    match args {
        ToleranceArgs::Exact => Ok(Tolerance::Exact),
        ToleranceArgs::Manifest { tsv, row } => {
            let row = row
                .as_deref()
                .ok_or_else(|| "--tolerance-from-manifest needs --row".to_string())?;
            tolerance::from_manifest_row(tsv, row)
        }
        ToleranceArgs::Config { md, configuration } => {
            let configuration = configuration
                .as_deref()
                .ok_or_else(|| "--tolerance-from-config needs --configuration".to_string())?;
            tolerance::from_config_row(md, configuration)
        }
    }
}

fn run() -> Result<ExitCode, String> {
    let raw: Vec<String> = std::env::args().skip(1).collect();
    let args = parse_args(&raw).map_err(|e| format!("{e}\n{}", usage()))?;

    // Resolved before either G-code file is read: a bad tolerance cell exits non-zero having
    // compared nothing, never a report built on a silent default.
    let tolerance = resolve_tolerance(&args.tolerance)?;

    let a = std::fs::read(&args.a).map_err(|e| format!("cannot read file '{}': {e}", args.a))?;
    let b = std::fs::read(&args.b).map_err(|e| format!("cannot read file '{}': {e}", args.b))?;

    let report = oracle_compare::compare_with_tolerance(&a, &b, tolerance);

    let rendered = match args.format {
        OutputFormat::Text => format::render_text(&report),
        OutputFormat::Json => format::render_json(&report),
    };
    print!("{rendered}");

    let ok = match args.require {
        Some(level) => report.passed(level),
        None => true,
    };

    Ok(if ok {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    })
}

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(message) => {
            eprintln!("error: {message}");
            ExitCode::FAILURE
        }
    }
}
