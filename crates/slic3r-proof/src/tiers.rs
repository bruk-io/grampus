//! The tier dispatch every ported unit's test calls (task 31-proof-harness, plan step 18, R19).
//! [`compare_unit`] is the one API every ported unit's test uses, so its shape is fixed here
//! rather than left to the builder: the unit identifies itself by its vendored source path (the
//! same path its own completion trailer names -- this module never reads that trailer at run
//! time, the caller passes the same path), the manifest is located from an environment variable
//! defaulting to the workspace root, and the oracle is passed in by the caller because no
//! manifest column holds it.
//!
//! It reads a unit's row in `port/manifest.tsv` by that source path and applies the tier the row
//! declares: an `exact` row compares by exact equality of its proof's output, with no tolerance
//! path reachable; a row whose `equality tier` cell names anything else (there is no fixed
//! literal for this -- no row in the real manifest has needed one yet, see this task's pull
//! request body) compares by that row's declared `tolerance` and `comparison function`, and
//! nothing else. A row whose `tolerance` cell carries a value with no reason text, and a row
//! whose `proof path` cell is empty, are each refused with a named error naming the row, rather
//! than compared -- the same reason-text rule `crates/oracle-compare/src/tolerance.rs` applies
//! to the comparator (plan step 5), reimplemented here rather than taken as a dependency: this
//! crate reads four columns at once by one source path, not two independently by a tolerance or
//! a configuration key, and its `ComparisonFunction` only ever needs to agree with the
//! comparator's in spirit (the single `abs` function), not share its type.

use std::env;
use std::fmt;
use std::path::{Path, PathBuf};

/// Which real libslic3r this proof ran against. No manifest column holds this -- the caller
/// passes it, because the manifest's `proof path` column names the *mechanism* (FFI, a stage
/// dump, an included translation unit), not which binary that mechanism reached.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Oracle {
    /// The pinned `v2.5.2` release binary (`oracle/slice`).
    Release,
    /// The library `just oracle-build` compiles from the pin (`vendor/OrcaSlicer/build/arm64`).
    Built,
}

impl fmt::Display for Oracle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Oracle::Release => "release",
            Oracle::Built => "built",
        })
    }
}

/// The comparison functions this crate implements -- `Abs` (absolute deviation, `|actual -
/// expected| <= tolerance`) is the only one, mirroring `oracle-compare`'s own decision (plan
/// step 5) that it is the only one there is; an empty `comparison function` cell on a row that
/// declares a tolerance also means `Abs`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ComparisonFunction {
    Abs,
}

impl ComparisonFunction {
    fn parse(name: &str) -> Option<ComparisonFunction> {
        match name.trim().to_ascii_lowercase().as_str() {
            "abs" | "absolute" => Some(ComparisonFunction::Abs),
            _ => None,
        }
    }
}

impl fmt::Display for ComparisonFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            ComparisonFunction::Abs => "abs",
        })
    }
}

/// The tier a manifest row declares, already validated: an `exact` row carries nothing further
/// (no tolerance path is reachable from it, even when the row's `tolerance` cell is non-empty
/// and well-formed -- validated below regardless of tier, but never read again for this one);
/// a `Float` row carries its declared tolerance, unit, reason and comparison function, all of
/// which [`compare_unit`] uses and nothing else.
#[derive(Debug, Clone, PartialEq)]
enum Tier {
    Exact,
    Float {
        value: f64,
        unit: Option<String>,
        reason: String,
        comparison_function: ComparisonFunction,
    },
}

impl Tier {
    fn name(&self) -> &'static str {
        match self {
            Tier::Exact => "exact",
            Tier::Float { .. } => "float",
        }
    }
}

/// A validated manifest row: just the two cells [`compare_unit`] needs beyond the tier itself.
struct ValidatedRow {
    tier: Tier,
    proof_path: String,
}

/// Every error [`compare_unit`]/[`compare_unit_at`] can return, each naming the row it concerns
/// so a failure is legible without re-reading the manifest by hand.
#[derive(Debug)]
pub enum TierError {
    /// `port/manifest.tsv` (or the path `GRAMPUS_MANIFEST_PATH` names) could not be read at all.
    ManifestMissing {
        manifest_path: PathBuf,
        io_error: String,
    },
    /// The manifest file exists but is missing a column `compare_unit` needs, or has no header
    /// row at all.
    ManifestMalformed {
        manifest_path: PathBuf,
        reason: String,
    },
    /// No row's `path` cell equals `source_path`.
    RowNotFound {
        manifest_path: PathBuf,
        source_path: String,
    },
    /// The row's `proof path` cell is empty.
    EmptyProofPath { source_path: String },
    /// The row's `tolerance` cell carries a value with no `reason:` clause (or an empty one).
    ToleranceMissingReason { source_path: String, cell: String },
    /// The row's `tolerance` cell carries a `reason:` clause, but the value before it is not a
    /// number.
    ToleranceUnparsable {
        source_path: String,
        cell: String,
        detail: String,
    },
    /// The row's `equality tier` cell names anything but `exact`, but its `tolerance` cell is
    /// empty -- a float row must declare the tolerance it is compared at.
    FloatRowMissingTolerance { source_path: String },
    /// The row's `comparison function` cell names a function this crate does not implement.
    UnimplementedComparisonFunction { source_path: String, name: String },
    /// The row's `equality tier` cell is empty -- not yet seeded, so there is nothing to compare
    /// at rather than a default to fall back to.
    UnknownTier { source_path: String },
    /// An `exact`-tier comparison found `actual != expected`.
    Mismatch {
        source_path: String,
        proof_path: String,
        oracle: Oracle,
        actual: String,
        expected: String,
    },
    /// A float-tier comparison's deviation exceeded the row's declared tolerance.
    ToleranceExceeded {
        source_path: String,
        proof_path: String,
        oracle: Oracle,
        deviation: f64,
        tolerance: f64,
        reason: String,
    },
}

impl fmt::Display for TierError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TierError::ManifestMissing {
                manifest_path,
                io_error,
            } => write!(
                f,
                "slic3r-proof: cannot read manifest {}: {io_error}",
                manifest_path.display()
            ),
            TierError::ManifestMalformed {
                manifest_path,
                reason,
            } => write!(
                f,
                "slic3r-proof: manifest {} is malformed: {reason}",
                manifest_path.display()
            ),
            TierError::RowNotFound {
                manifest_path,
                source_path,
            } => write!(
                f,
                "slic3r-proof: no row for '{source_path}' in manifest {}",
                manifest_path.display()
            ),
            TierError::EmptyProofPath { source_path } => write!(
                f,
                "slic3r-proof: row '{source_path}' has an empty 'proof path' cell; refused \
                 rather than compared"
            ),
            TierError::ToleranceMissingReason { source_path, cell } => write!(
                f,
                "slic3r-proof: row '{source_path}' has a 'tolerance' cell with no reason \
                 ('{cell}'); refused rather than compared"
            ),
            TierError::ToleranceUnparsable {
                source_path,
                cell,
                detail,
            } => write!(
                f,
                "slic3r-proof: row '{source_path}' has an unparsable 'tolerance' cell \
                 ('{cell}'): {detail}"
            ),
            TierError::FloatRowMissingTolerance { source_path } => write!(
                f,
                "slic3r-proof: row '{source_path}' has a non-exact 'equality tier' but an empty \
                 'tolerance' cell; a float row must declare one"
            ),
            TierError::UnimplementedComparisonFunction { source_path, name } => write!(
                f,
                "slic3r-proof: row '{source_path}' names comparison function '{name}', which \
                 this crate does not implement"
            ),
            TierError::UnknownTier { source_path } => write!(
                f,
                "slic3r-proof: row '{source_path}' has an empty 'equality tier' cell; nothing \
                 to compare at"
            ),
            TierError::Mismatch {
                source_path,
                proof_path,
                oracle,
                actual,
                expected,
            } => write!(
                f,
                "slic3r-proof: {source_path} (tier=exact proof_path={proof_path} \
                 oracle={oracle}) FAILED: actual {actual} != expected {expected}"
            ),
            TierError::ToleranceExceeded {
                source_path,
                proof_path,
                oracle,
                deviation,
                tolerance,
                reason,
            } => write!(
                f,
                "slic3r-proof: {source_path} (tier=float proof_path={proof_path} \
                 oracle={oracle}) FAILED: deviation {deviation} exceeds declared tolerance \
                 {tolerance} ({reason})"
            ),
        }
    }
}

impl std::error::Error for TierError {}

/// What [`compare_unit`] needs from a proved value: exact equality for the `exact` tier, and the
/// maximum per-element absolute deviation from `other` for a float tier's declared tolerance.
/// Implemented here for the primitive types a proof shim returns plain integers or floats as,
/// and for `Vec<T>` component-wise, so a unit whose proof returns a list of coordinates never
/// needs its own impl.
///
/// No impl for a struct type is provided or intended: a bridge struct such as
/// `slic3r_ffi::proof::ProofZPoint` lives in a different crate, and implementing a trait this
/// crate owns for a type a dependency owns is fine by Rust's orphan rule either way the
/// dependency runs -- but adding that impl *here* (the only direction that does not also need a
/// reverse dependency back onto this crate) would make `tiers.rs` know about every bridge
/// struct any proof shim ever returns. A unit whose proof returns a composite value instead
/// flattens it to a `Vec<i64>` (or another primitive) at the call site before calling
/// `compare_unit` -- see `crates/slic3r-proof/tests/harness_paths.rs`'s path 3 test for the
/// pattern -- rather than this trait growing an impl per bridge type.
pub trait ProofValue: fmt::Debug {
    fn tiers_eq(&self, other: &Self) -> bool;
    fn abs_deviation(&self, other: &Self) -> f64;
}

macro_rules! impl_proof_value_int {
    ($($t:ty),+) => {
        $(
            impl ProofValue for $t {
                fn tiers_eq(&self, other: &Self) -> bool {
                    self == other
                }
                fn abs_deviation(&self, other: &Self) -> f64 {
                    (*self as f64 - *other as f64).abs()
                }
            }
        )+
    };
}
impl_proof_value_int!(i32, i64, u32, u64, isize, usize);

impl ProofValue for f64 {
    fn tiers_eq(&self, other: &Self) -> bool {
        self == other
    }
    fn abs_deviation(&self, other: &Self) -> f64 {
        (self - other).abs()
    }
}

impl<T: ProofValue> ProofValue for Vec<T> {
    fn tiers_eq(&self, other: &Self) -> bool {
        self.len() == other.len() && self.iter().zip(other).all(|(a, b)| a.tiers_eq(b))
    }
    fn abs_deviation(&self, other: &Self) -> f64 {
        if self.len() != other.len() {
            return f64::INFINITY;
        }
        self.iter()
            .zip(other)
            .map(|(a, b)| a.abs_deviation(b))
            .fold(0.0_f64, f64::max)
    }
}

/// The environment variable `compare_unit` locates `port/manifest.tsv` from; its default is the
/// workspace root (this crate's `CARGO_MANIFEST_DIR`, two directories up) plus
/// `port/manifest.tsv`. No path to the manifest is hardcoded beyond that default.
pub const MANIFEST_PATH_VAR: &str = "GRAMPUS_MANIFEST_PATH";

fn default_manifest_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../port/manifest.tsv")
}

/// Resolves where `compare_unit` reads the manifest from: `GRAMPUS_MANIFEST_PATH` if set, else
/// the workspace-root default.
pub fn resolve_manifest_path() -> PathBuf {
    match env::var_os(MANIFEST_PATH_VAR) {
        Some(p) => PathBuf::from(p),
        None => default_manifest_path(),
    }
}

/// Parses a `tolerance` cell of the shape `<value>[ <unit>]; reason: <text>` (the same shape
/// `crates/oracle-compare/src/tolerance.rs` parses). `Ok(None)` for an empty cell -- no
/// tolerance declared, which an `exact` row never needed and a float row must refuse. `Err` for
/// a cell that carries a value with no `reason:` clause, or an empty one.
fn parse_tolerance_cell(
    cell: &str,
    source_path: &str,
) -> Result<Option<(f64, Option<String>, String)>, TierError> {
    let cell = cell.trim();
    if cell.is_empty() {
        return Ok(None);
    }
    let missing_reason = || TierError::ToleranceMissingReason {
        source_path: source_path.to_string(),
        cell: cell.to_string(),
    };

    let semi = cell.find(';').ok_or_else(missing_reason)?;
    let value_part = cell[..semi].trim();
    let after_semi = cell[semi + 1..].trim();
    let reason = after_semi
        .strip_prefix("reason:")
        .map(|r| r.trim())
        .filter(|r| !r.is_empty())
        .ok_or_else(missing_reason)?
        .to_string();

    let mut parts = value_part.split_whitespace();
    let value_str = parts.next().ok_or_else(missing_reason)?;
    let value: f64 = value_str.parse().map_err(|e: std::num::ParseFloatError| {
        TierError::ToleranceUnparsable {
            source_path: source_path.to_string(),
            cell: cell.to_string(),
            detail: e.to_string(),
        }
    })?;
    let unit = parts.next().map(|s| s.to_string());
    Ok(Some((value, unit, reason)))
}

fn parse_comparison_function(
    cell: &str,
    source_path: &str,
) -> Result<ComparisonFunction, TierError> {
    let name = cell.trim();
    if name.is_empty() {
        return Ok(ComparisonFunction::Abs);
    }
    ComparisonFunction::parse(name).ok_or_else(|| TierError::UnimplementedComparisonFunction {
        source_path: source_path.to_string(),
        name: name.to_string(),
    })
}

/// Parses `content` (a `port/manifest.tsv`-shaped TSV, header row first) for `source_path`'s
/// row, by column name -- so a reordered or renamed column does not silently break this, the
/// same reason `crates/oracle-compare/src/tolerance.rs` finds its own columns by name. Returns a
/// validated [`ValidatedRow`] or a named [`TierError`]; never compares anything itself.
fn parse_row(
    content: &str,
    manifest_path: &Path,
    source_path: &str,
) -> Result<ValidatedRow, TierError> {
    let mut lines = content.lines();
    let header_line = lines.next().ok_or_else(|| TierError::ManifestMalformed {
        manifest_path: manifest_path.to_path_buf(),
        reason: "empty manifest".to_string(),
    })?;
    let headers: Vec<&str> = header_line.split('\t').collect();
    let find_col = |name: &str| headers.iter().position(|h| h.eq_ignore_ascii_case(name));

    let malformed = |reason: String| TierError::ManifestMalformed {
        manifest_path: manifest_path.to_path_buf(),
        reason,
    };
    let path_col = find_col("path").ok_or_else(|| malformed("no 'path' column".to_string()))?;
    let tier_col = find_col("equality tier")
        .ok_or_else(|| malformed("no 'equality tier' column".to_string()))?;
    let tolerance_col =
        find_col("tolerance").ok_or_else(|| malformed("no 'tolerance' column".to_string()))?;
    let function_col = find_col("comparison function")
        .ok_or_else(|| malformed("no 'comparison function' column".to_string()))?;
    let proof_path_col =
        find_col("proof path").ok_or_else(|| malformed("no 'proof path' column".to_string()))?;

    for line in lines {
        if line.is_empty() {
            continue;
        }
        let fields: Vec<&str> = line.split('\t').collect();
        if fields.get(path_col).copied().unwrap_or("") != source_path {
            continue;
        }

        let proof_path = fields
            .get(proof_path_col)
            .copied()
            .unwrap_or("")
            .trim()
            .to_string();
        if proof_path.is_empty() {
            return Err(TierError::EmptyProofPath {
                source_path: source_path.to_string(),
            });
        }

        let tolerance_cell = fields.get(tolerance_col).copied().unwrap_or("");
        let function_cell = fields.get(function_col).copied().unwrap_or("");
        // Both validated unconditionally, before branching on the tier -- a malformed tolerance
        // cell is refused whether or not the row's tier will go on to read it (an `exact` row has
        // "no tolerance path reachable", but a tolerance value with no reason is still refused,
        // never silently ignored), and the same now holds for the comparison function: reading it
        // only inside the float arm let an `exact` row naming an unimplemented function resolve
        // silently instead of being refused, the same shape `oracle-compare`'s `build_tolerance`
        // documents fixing. An empty cell still means `Abs` on every tier, so this changes nothing
        // for a well-formed `exact` row (whose comparison-function cell is empty, as every `exact`
        // row in the real manifest's is).
        let declared = parse_tolerance_cell(tolerance_cell, source_path)?;
        let comparison_function = parse_comparison_function(function_cell, source_path)?;

        let tier_cell = fields.get(tier_col).copied().unwrap_or("").trim();
        let tier = if tier_cell.eq_ignore_ascii_case("exact") {
            Tier::Exact
        } else if tier_cell.is_empty() {
            return Err(TierError::UnknownTier {
                source_path: source_path.to_string(),
            });
        } else {
            let (value, unit, reason) =
                declared.ok_or_else(|| TierError::FloatRowMissingTolerance {
                    source_path: source_path.to_string(),
                })?;
            Tier::Float {
                value,
                unit,
                reason,
                comparison_function,
            }
        };

        return Ok(ValidatedRow { tier, proof_path });
    }

    Err(TierError::RowNotFound {
        manifest_path: manifest_path.to_path_buf(),
        source_path: source_path.to_string(),
    })
}

fn load_row(manifest_path: &Path, source_path: &str) -> Result<ValidatedRow, TierError> {
    let content =
        std::fs::read_to_string(manifest_path).map_err(|e| TierError::ManifestMissing {
            manifest_path: manifest_path.to_path_buf(),
            io_error: e.to_string(),
        })?;
    parse_row(&content, manifest_path, source_path)
}

/// `compare_unit` at an explicit manifest path, bypassing `GRAMPUS_MANIFEST_PATH` -- the same
/// explicit-path idiom `crates/oracle-compare/src/tolerance.rs`'s readers use. `compare_unit`
/// itself is a thin wrapper over this that resolves the path from the environment.
pub fn compare_unit_at<T: ProofValue>(
    manifest_path: impl AsRef<Path>,
    source_path: &str,
    oracle: Oracle,
    actual: T,
    expected: T,
) -> Result<(), TierError> {
    let manifest_path = manifest_path.as_ref();
    let row = load_row(manifest_path, source_path)?;

    let outcome = match &row.tier {
        Tier::Exact => {
            if actual.tiers_eq(&expected) {
                Ok(())
            } else {
                Err(TierError::Mismatch {
                    source_path: source_path.to_string(),
                    proof_path: row.proof_path.clone(),
                    oracle,
                    actual: format!("{actual:?}"),
                    expected: format!("{expected:?}"),
                })
            }
        }
        Tier::Float {
            value,
            reason,
            comparison_function: ComparisonFunction::Abs,
            ..
        } => {
            let deviation = actual.abs_deviation(&expected);
            if deviation <= *value {
                Ok(())
            } else {
                Err(TierError::ToleranceExceeded {
                    source_path: source_path.to_string(),
                    proof_path: row.proof_path.clone(),
                    oracle,
                    deviation,
                    tolerance: *value,
                    reason: reason.clone(),
                })
            }
        }
    };

    println!(
        "slic3r-proof: {source_path} tier={tier} proof_path={proof_path} oracle={oracle} -> {status}",
        tier = row.tier.name(),
        proof_path = row.proof_path,
        status = if outcome.is_ok() { "pass" } else { "FAIL" },
    );
    outcome
}

/// The tier dispatch from the manifest row (task 31-proof-harness, plan step 18, R19): looks up
/// `source_path`'s row in `port/manifest.tsv` (located from [`MANIFEST_PATH_VAR`], defaulting to
/// the workspace root) and compares `actual` against `expected` at the tier that row declares --
/// exact equality for an `exact` row, the row's declared tolerance and comparison function for
/// any other (non-empty) tier. `oracle` is the caller's own record of which real libslic3r this
/// proof ran against; no manifest column holds it. Both a passing and a failing comparison print
/// the row's path, tier, proof path and oracle.
pub fn compare_unit<T: ProofValue>(
    source_path: &str,
    oracle: Oracle,
    actual: T,
    expected: T,
) -> Result<(), TierError> {
    compare_unit_at(
        resolve_manifest_path(),
        source_path,
        oracle,
        actual,
        expected,
    )
}
