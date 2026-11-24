//! `oracle-compare <a> <b> [--require <level>] [--format text|json]`
//!
//! Evaluates L0 and L1 between two files, read as raw bytes. L2, L3 and L4 are not evaluated by
//! this binary yet (task `09-comparator-gcode-levels` fills them in) and always report
//! "not evaluated".
//!
//! Exit status: 0 whenever the comparison ran, including when no evaluated level passed, unless
//! `--require <level>` names a level that did not pass, or there was a usage error or an
//! unreadable file (both exit non-zero with a named error on stderr).

use std::process::ExitCode;

use oracle_compare::{format, Level};

struct Args {
    a: String,
    b: String,
    require: Option<Level>,
    format: OutputFormat,
}

#[derive(Clone, Copy)]
enum OutputFormat {
    Text,
    Json,
}

fn usage() -> &'static str {
    "usage: oracle-compare <a> <b> [--require <level>] [--format text|json]"
}

fn parse_args(raw: &[String]) -> Result<Args, String> {
    let mut positional: Vec<String> = Vec::new();
    let mut require: Option<Level> = None;
    let mut format = OutputFormat::Text;

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

    let b = positional.pop().expect("checked len == 2 above");
    let a = positional.pop().expect("checked len == 2 above");
    Ok(Args {
        a,
        b,
        require,
        format,
    })
}

fn run() -> Result<ExitCode, String> {
    let raw: Vec<String> = std::env::args().skip(1).collect();
    let args = parse_args(&raw).map_err(|e| format!("{e}\n{}", usage()))?;

    let a = std::fs::read(&args.a).map_err(|e| format!("cannot read file '{}': {e}", args.a))?;
    let b = std::fs::read(&args.b).map_err(|e| format!("cannot read file '{}': {e}", args.b))?;

    let report = oracle_compare::compare(&a, &b);

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
