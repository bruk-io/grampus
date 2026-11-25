//! `grampus-cli <model> --config <golden> --out <dir> [--export-statistics]`
//!
//! The binary tasks `39-compare-stages`, `41-compare-benchy`, `42-compare-fixtures` and
//! `43-compare-configurations` call (task `33-port-surface`, plan step 19). Its command line is
//! fixed here because those four tasks are written against it: a positional model path,
//! `--config <golden>` (one of the resolved-configuration goldens under
//! `port/goldens/configs/`), `--out <dir>` and `--export-statistics`, matching `oracle/slice`'s
//! own `--out` and `--export-statistics` (plan step 2). It writes its G-code as `plate_1.gcode`
//! under `--out` — the name the oracle writes, `plate_<N>.gcode` — and, with
//! `--export-statistics`, a statistics JSON under the oracle's own `plate_1.statistics.json`
//! basename, so `just compare` compares like-named files. `--config` is read and parsed before
//! the surface's five-call sequence starts, so a bad path is reported as its own error rather
//! than being masked by `build_model`'s not-yet-ported error.
//!
//! Until Phase B lands (plan step 23) every path below exits non-zero with a named "not yet
//! ported" error on stderr, never a panic: [`grampus_cli::surface::Surface`]'s five calls all
//! return [`SurfaceError::NotYetPorted`] once their ordering precondition holds, and this
//! binary calls them in order and reports whichever one fires first — currently always
//! `build_model`, the first of the five.

use std::path::PathBuf;
use std::process::ExitCode;

use grampus_cli::resolved_config::{ResolvedConfig, ResolvedConfigError};
use grampus_cli::surface::{Surface, SurfaceError};

struct Args {
    model: PathBuf,
    config: PathBuf,
    out: PathBuf,
    export_statistics: bool,
    dump_stages: Option<PathBuf>,
}

/// Printed for `--help` and on a usage error; names all five options this binary takes
/// (`--dump-stages` added by task `39-compare-stages`, R20).
fn usage() -> &'static str {
    "usage: grampus-cli <model> --config <golden> --out <dir> [--export-statistics]\n       \
     [--dump-stages <dir>]\n\n\
     <model>               positional: path to the mesh to slice\n\
     --config <golden>     a resolved-configuration golden (port/goldens/configs/<name>.json)\n\
     --out <dir>           directory the G-code and (with --export-statistics) the statistics\n\
     \x20                     JSON are written under\n\
     --export-statistics   also write <out>/plate_1.statistics.json\n\
     --dump-stages <dir>   write the R5 per-seam dump format to <dir> at each ported seam\n\
     \x20                     (`Surface::process`); a seam whose modules are not ported yet\n\
     \x20                     writes nothing"
}

enum ParsedArgs {
    Help,
    Run(Args),
}

fn parse_args(raw: &[String]) -> Result<ParsedArgs, String> {
    let mut model: Option<PathBuf> = None;
    let mut config: Option<PathBuf> = None;
    let mut out: Option<PathBuf> = None;
    let mut export_statistics = false;
    let mut dump_stages: Option<PathBuf> = None;

    let mut i = 0;
    while i < raw.len() {
        match raw[i].as_str() {
            "--help" | "-h" => return Ok(ParsedArgs::Help),
            "--config" => {
                let value = raw
                    .get(i + 1)
                    .ok_or_else(|| "--config needs a path".to_string())?;
                config = Some(PathBuf::from(value));
                i += 2;
            }
            "--out" => {
                let value = raw
                    .get(i + 1)
                    .ok_or_else(|| "--out needs a path".to_string())?;
                out = Some(PathBuf::from(value));
                i += 2;
            }
            "--export-statistics" => {
                export_statistics = true;
                i += 1;
            }
            "--dump-stages" => {
                let value = raw
                    .get(i + 1)
                    .ok_or_else(|| "--dump-stages needs a path".to_string())?;
                dump_stages = Some(PathBuf::from(value));
                i += 2;
            }
            other if other.starts_with("--") => {
                return Err(format!("unknown flag: {other}"));
            }
            other => {
                if model.is_some() {
                    return Err(format!("unexpected extra positional argument: {other}"));
                }
                model = Some(PathBuf::from(other));
                i += 1;
            }
        }
    }

    let model = model.ok_or_else(|| "missing positional argument: <model>".to_string())?;
    let config = config.ok_or_else(|| "missing required flag: --config <golden>".to_string())?;
    let out = out.ok_or_else(|| "missing required flag: --out <dir>".to_string())?;

    Ok(ParsedArgs::Run(Args {
        model,
        config,
        out,
        export_statistics,
        dump_stages,
    }))
}

/// This binary's one named error, never a panic: a usage problem, a resolved-configuration
/// read failure, the surface's own not-yet-ported/out-of-order error, or the statistics export
/// path's own not-yet-ported report (there is no `Surface::export_statistics` method to name —
/// statistics export has no entry point of its own on `Surface`, R10's five calls being fixed —
/// so this gets its own variant rather than borrowing `SurfaceError::NotYetPorted` with a method
/// name that doesn't exist).
enum CliError {
    Usage(String),
    ResolvedConfig(ResolvedConfigError),
    Surface(SurfaceError),
    StatisticsNotYetPorted,
}

impl std::fmt::Display for CliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CliError::Usage(message) => write!(f, "{message}\n\n{}", usage()),
            CliError::ResolvedConfig(e) => write!(f, "{e}"),
            CliError::Surface(e) => write!(f, "{e}"),
            CliError::StatisticsNotYetPorted => {
                write!(f, "not yet ported: plate_1.statistics.json export")
            }
        }
    }
}

/// Runs the five-call sequence in order, reporting whichever call's precondition or
/// not-yet-ported error fires first. `out` and `export_statistics` name where this binary
/// would write `plate_1.gcode` and `plate_1.statistics.json` once `export_gcode` is ported;
/// today the sequence never reaches either write, because `build_model` is first and always
/// returns `SurfaceError::NotYetPorted`.
fn run(args: &Args) -> Result<(), CliError> {
    // Read and parse `--config` before entering the surface's five-call sequence, so a bad
    // golden path is reported as its own error rather than being masked by `build_model`'s
    // not-yet-ported error (which fires regardless of whether `--config` is even readable).
    // This changes nothing about the *surface's* call order, which still mirrors the oracle's
    // build-model-then-apply-config sequence: reading a file off disk is not `print.apply`.
    let mut config = ResolvedConfig::from_golden(&args.config).map_err(CliError::ResolvedConfig)?;
    // The oracle derives this from the printer's vendor preset, never from the resolved
    // configuration's own JSON (see resolved_config's doc comment); this binary has no vendor
    // preset to read, so until Phase B defines where a caller states it, every run is BBL.
    config.set_bambu_printer(true);

    let mut surface = Surface::new();
    if let Some(dir) = &args.dump_stages {
        surface.set_dump_stages_dir(dir.clone());
    }

    surface
        .build_model(&args.model)
        .map_err(CliError::Surface)?;
    surface.apply_config(&config).map_err(CliError::Surface)?;
    surface.validate().map_err(CliError::Surface)?;
    surface.process().map_err(CliError::Surface)?;

    let gcode_path = args.out.join("plate_1.gcode");
    surface
        .export_gcode(&gcode_path)
        .map_err(CliError::Surface)?;

    if args.export_statistics {
        let statistics_path = args.out.join("plate_1.statistics.json");
        // Not yet ported: statistics export has no entry point of its own on `Surface` (R10's
        // five calls are fixed), so until Phase B's statistics writer exists, this path reports
        // the same named "not yet ported" error the other four CLI paths do, rather than
        // silently exiting 0 having written nothing.
        let _ = statistics_path;
        return Err(CliError::StatisticsNotYetPorted);
    }

    Ok(())
}

fn main() -> ExitCode {
    let raw: Vec<String> = std::env::args().skip(1).collect();

    match parse_args(&raw) {
        Ok(ParsedArgs::Help) => {
            println!("{}", usage());
            ExitCode::SUCCESS
        }
        Ok(ParsedArgs::Run(args)) => match run(&args) {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("error: {e}");
                ExitCode::FAILURE
            }
        },
        Err(message) => {
            eprintln!("error: {}", CliError::Usage(message));
            ExitCode::FAILURE
        }
    }
}
