//! Proof path 2 (R19, plan step 18, task 32-proof-dumps-fuzz): the committed stage dumps task
//! `25-stage-dumper` and `26-stage-dump-seams` write (`just oracle-dump`), read back here for a
//! unit whose manifest row names `proof path` `2` -- a whole pipeline step, rather than a single
//! externally linked symbol (path 1) or an included translation unit (path 3).
//!
//! The dump format itself is documented once, in `oracle/stage-dump/README.md`'s "Dump format"
//! section (the writer, `dump_seams.cpp`, is held to it) -- this module does not restate the
//! field list, only the shape every line carries and the three things a reader has to get right:
//!
//! 1. Every coordinate (`x`/`y`/`z` inside a `pts=` token) is parsed with [`str::parse::<i64>`],
//!    never through [`f64`] -- `coord_t` is `int64_t` at the pin
//!    (`vendor/OrcaSlicer/src/libslic3r/libslic3r.h:45-48`), so a coordinate above 2^53 is not
//!    representable exactly in `f64` and a float round-trip would lose exactly the exactness an
//!    `exact` tier asserts (`crates/slic3r-proof/tests/data/oversized-coordinate.dump` proves
//!    this). A scalar `coordf_t`/`double`/`float` field (`print_z`, `wipe_tower_depth`, ...) is
//!    written with C's `%a` (hexadecimal floating-point) conversion instead, which *is* exact for
//!    the value held -- [`parse_hexfloat`] parses that shape, used only for those scalar fields,
//!    never for a coordinate.
//! 2. [`parse_dump_text`] (and so [`read_dump`]/[`read_dump_default`]) refuses a dump whose leading
//!    provenance block carries no `oracle-commit` key, or one that disagrees with [`PINNED_COMMIT`],
//!    with a [`DumpError`] naming the file -- never comparing against a stale dump. This is the
//!    same judgement `oracle/provenance --check` makes (plan step 3), reimplemented here rather
//!    than shelled out to: that script's own job is checking every covered file at once from a
//!    path list, not parsing one dump's body into queryable per-seam state.
//! 3. [`resolve_dumps_path`] takes the dump directory from [`DUMPS_PATH_VAR`], defaulting to the
//!    workspace root's `port/dumps/` -- no path is hardcoded into [`read_dump`] itself, mirroring
//!    `tiers.rs`'s `GRAMPUS_MANIFEST_PATH` convention for `port/manifest.tsv`.
//!
//! [`parse_hand_written_fixture_records`] is the one exception to rule 2: a pure body-line parser
//! with no header requirement at all, used internally by [`parse_dump_text`] and exposed directly
//! for `crates/slic3r-proof/tests/data/oversized-coordinate.dump` -- the one hand-written fixture
//! this branch commits (see its own leading comment), which carries no provenance header by
//! design (a 256 by 256 mm bed at `SCALING_FACTOR` cannot produce a real coordinate anywhere
//! near 2^53, so no oracle run can ever write this case) and so cannot be read through
//! [`read_dump`]/[`parse_dump_text`], which would refuse it for exactly that reason. Its name
//! says so, so a consumer reaching for a general dump reader finds only [`read_dump`]/
//! [`read_dump_default`]/[`parse_dump_text`] instead of a provenance-free entry point into any
//! dump text.

use std::collections::BTreeMap;
use std::env;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

/// The vendored commit every covered oracle output is pinned to (task.md's own wording; the same
/// literal `oracle/provenance`'s `PINNED_COMMIT` and `crates/slic3r-ffi/build.rs`'s module doc
/// each carry, duplicated here rather than taken as a cross-language dependency).
pub const PINNED_COMMIT: &str = "f92a7c065c5718b8ef1baac1cd704df2911238f4";

/// The environment variable [`read_dump`] locates the dump directory from; its default is the
/// workspace root (this crate's `CARGO_MANIFEST_DIR`, two directories up) plus `port/dumps`. No
/// path is hardcoded beyond that default -- the same shape `tiers.rs`'s `MANIFEST_PATH_VAR` uses
/// for `port/manifest.tsv`.
pub const DUMPS_PATH_VAR: &str = "GRAMPUS_DUMPS_PATH";

fn default_dumps_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../port/dumps")
}

/// Resolves the directory [`read_dump`] reads a `<pair>/<seam>.dump` file under:
/// [`DUMPS_PATH_VAR`] if set, else the workspace-root default (`port/dumps/`).
pub fn resolve_dumps_path() -> PathBuf {
    match env::var_os(DUMPS_PATH_VAR) {
        Some(p) => PathBuf::from(p),
        None => default_dumps_path(),
    }
}

/// Every error this module can return, each naming the file (and, where the failure is within
/// one line, the line) it concerns -- the same "refused rather than silently accepted, and named"
/// rule `tiers.rs`'s `TierError` follows for a manifest row.
#[derive(Debug)]
pub enum DumpError {
    /// The dump file could not be read at all.
    Io { path: PathBuf, io_error: String },
    /// The leading `;`-comment block carries no `oracle-commit` key.
    MissingProvenanceHeader { path: PathBuf },
    /// The leading block's `oracle-commit` key disagrees with [`PINNED_COMMIT`].
    ProvenanceMismatch {
        path: PathBuf,
        found: String,
        expected: &'static str,
    },
    /// A body line's first token is not `field=<name>`, or a later token is neither `key=value`
    /// nor (once a `pts=` token has been seen) a bare coordinate tuple.
    MalformedLine {
        path: PathBuf,
        line: usize,
        reason: String,
    },
    /// A `pts=` tuple component, or a plain-integer scalar `value=`, did not parse as an integer.
    MalformedInteger {
        path: PathBuf,
        line: usize,
        token: String,
        detail: String,
    },
    /// A scalar `value=`/`min=`/`max=` token looked like a hex float (`0x...`) but did not parse
    /// as one.
    MalformedHexFloat {
        path: PathBuf,
        line: usize,
        token: String,
        detail: String,
    },
    /// The field named by `field` and the record-identifying arguments at the call site (layer,
    /// object, seam, ...) names no record in the dump this call read.
    FieldNotFound {
        path: PathBuf,
        field: String,
        detail: String,
    },
    /// A record [`Dump::points_at`] (or [`Dump::points_at_support_layer`]) matched on `field`
    /// (and `layer`/`region`, or `support_layer`) also carries a record-placing key the call did
    /// not constrain at all (`support_layer`, `object`, `group` or `brim`) -- refused rather than
    /// silently concatenated with every other record that differs only in that key, which would
    /// merge e.g. two print objects' `object_skirt` coordinates into one list.
    AmbiguousAxis {
        path: PathBuf,
        field: String,
        key: &'static str,
        line: usize,
    },
}

impl fmt::Display for DumpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DumpError::Io { path, io_error } => {
                write!(
                    f,
                    "slic3r-proof/dumps: cannot read {}: {io_error}",
                    path.display()
                )
            }
            DumpError::MissingProvenanceHeader { path } => write!(
                f,
                "slic3r-proof/dumps: {} carries no provenance header (no 'oracle-commit' key in \
                 its leading comment block); refused rather than compared",
                path.display()
            ),
            DumpError::ProvenanceMismatch {
                path,
                found,
                expected,
            } => write!(
                f,
                "slic3r-proof/dumps: {} names oracle-commit {found}, expected the pinned commit \
                 {expected}; refused rather than compared",
                path.display()
            ),
            DumpError::MalformedLine { path, line, reason } => {
                write!(f, "slic3r-proof/dumps: {}:{line}: {reason}", path.display())
            }
            DumpError::MalformedInteger {
                path,
                line,
                token,
                detail,
            } => write!(
                f,
                "slic3r-proof/dumps: {}:{line}: '{token}' is not a valid integer: {detail}",
                path.display()
            ),
            DumpError::MalformedHexFloat {
                path,
                line,
                token,
                detail,
            } => write!(
                f,
                "slic3r-proof/dumps: {}:{line}: '{token}' is not a valid hex float: {detail}",
                path.display()
            ),
            DumpError::FieldNotFound {
                path,
                field,
                detail,
            } => write!(
                f,
                "slic3r-proof/dumps: {}: no '{field}' record found ({detail})",
                path.display()
            ),
            DumpError::AmbiguousAxis {
                path,
                field,
                key,
                line,
            } => write!(
                f,
                "slic3r-proof/dumps: {}:{line}: field '{field}' carries a '{key}=' key this call \
                 did not constrain; refused rather than merged across its values (use a \
                 '{key}'-aware accessor instead)",
                path.display()
            ),
        }
    }
}

impl std::error::Error for DumpError {}

/// One body line of a dump, parsed generically: every `key=value` token the line carries (except
/// `pts=`, which is instead collected into [`Record::points`]) is kept verbatim in `raw`, so a
/// caller reads a key by the same name `oracle/stage-dump/README.md`'s "Dump format" section
/// gives it, rather than this module hand-naming one field per key up front. The sentinel `-`
/// (the README's own notation for "this axis does not apply") is kept as the literal string here;
/// [`Record::key_i64`] is what turns it into `None`.
#[derive(Debug, Clone)]
pub struct Record {
    /// The line's `field=<name>` token.
    pub field: String,
    /// Every other `key=value` token, keyed by `key`.
    pub raw: BTreeMap<String, String>,
    /// The `pts=<...>` token's coordinate tuples, in stored order -- each inner `Vec` is one
    /// point, length 2 (`x,y`, an `ExPolygons`/`SurfaceCollection` ring) or 3 (`x,y,z`, an
    /// `ExtrusionEntityCollection` leaf path), every component parsed with
    /// [`str::parse::<i64>`] and never through a float.
    pub points: Vec<Vec<i64>>,
    /// This record's 1-based line number in the file it was read from -- `text`'s own
    /// `str::lines()` index, header lines included, so it is the line a reader would actually
    /// find by opening the file, not an index into [`Dump::records`] (which holds body records
    /// only, since the header is never pushed into it).
    pub line: usize,
}

impl Record {
    /// Reads `key`'s raw string value, or `None` if the line does not carry that key at all, or
    /// carries it as the `-` sentinel.
    pub fn get(&self, key: &str) -> Option<&str> {
        self.raw.get(key).map(|s| s.as_str()).filter(|s| *s != "-")
    }

    /// Reads `key` as a plain decimal integer (the shape every index key -- `layer`, `region`,
    /// `support_layer`, `poly`, `surface`, `path`, `loop`, `group`, `brim`, `filament_id`,
    /// `object` -- uses). `Ok(None)` for an absent key or the `-` sentinel; `Err` for a key
    /// present with a value that is not a valid integer, named by file and this record's own
    /// [`Record::line`] so a malformed index is refused rather than silently read as `None`.
    pub fn key_i64(&self, path: &Path, key: &str) -> Result<Option<i64>, DumpError> {
        match self.get(key) {
            None => Ok(None),
            Some(v) => v
                .parse::<i64>()
                .map(Some)
                .map_err(|e| DumpError::MalformedInteger {
                    path: path.to_path_buf(),
                    line: self.line,
                    token: format!("{key}={v}"),
                    detail: e.to_string(),
                }),
        }
    }

    /// `layer=<L|->`, carried by every record (the README's own rule: "every record -- with no
    /// exception -- then carries both `layer=<L>` and `region=<R>`").
    pub fn layer(&self, path: &Path) -> Result<Option<i64>, DumpError> {
        self.key_i64(path, "layer")
    }

    /// `region=<R|->`, carried by every record alongside `layer`.
    pub fn region(&self, path: &Path) -> Result<Option<i64>, DumpError> {
        self.key_i64(path, "region")
    }

    /// `support_layer=<S|->`, carried alongside `layer=-` by a support-layer field
    /// (`support_islands`, `support_fills`) -- a separate index from `layer` (`support_layers()`
    /// vs `layers()`), never read as the same axis.
    pub fn support_layer(&self, path: &Path) -> Result<Option<i64>, DumpError> {
        self.key_i64(path, "support_layer")
    }

    /// `count=<n>`, the summary line every `ExPolygons`/`SurfaceCollection`/
    /// `ExtrusionEntityCollection` field carries once, before its per-element lines.
    pub fn count(&self, path: &Path) -> Result<Option<u64>, DumpError> {
        match self.get("count") {
            None => Ok(None),
            Some(v) => v
                .parse::<u64>()
                .map(Some)
                .map_err(|e| DumpError::MalformedInteger {
                    path: path.to_path_buf(),
                    line: self.line,
                    token: format!("count={v}"),
                    detail: e.to_string(),
                }),
        }
    }

    /// Any single-valued scalar key (`value`, or, on a record that uses them instead, `min`/
    /// `max`): a plain decimal integer (`wipe_tower_number_of_toolchanges`) or a hex float (every
    /// other scalar field -- `print_z`, `slice_z`, `height`, each `wipe_tower_*` field).
    /// Dispatches on whether the token looks like a hex float (`0x`/`-0x` prefix) -- never a
    /// coordinate, so this is never used for a `pts=` component. `Ok(None)` only when `key` is
    /// absent or the `-` sentinel; a key present but holding a comma-separated pair (`min=`/
    /// `max=`/`value=` on `wipe_tower_bbx`/`wipe_tower_rib_offset`) is a [`DumpError`], not a
    /// silent `None` -- see [`Record::pair_f64`] for that shape.
    pub fn scalar_f64(&self, path: &Path, key: &str) -> Result<Option<f64>, DumpError> {
        match self.get(key) {
            None => Ok(None),
            Some(v) if v.contains(',') => Err(DumpError::MalformedHexFloat {
                path: path.to_path_buf(),
                line: self.line,
                token: v.to_string(),
                detail: format!(
                    "'{key}={v}' is a comma-separated pair; read it with Record::pair_f64, not scalar_f64"
                ),
            }),
            Some(v) => parse_scalar(v, path, self.line).map(Some),
        }
    }

    /// A comma-separated pair of scalar values under `key` (`min=`/`max=` on `wipe_tower_bbx`,
    /// `value=` on `wipe_tower_rib_offset` -- a bounding box and a 2D offset are each two scalars,
    /// not one, per `oracle/stage-dump/README.md`'s "Dump format" section). `Ok(None)` only when
    /// `key` is absent or the `-` sentinel; a single, non-paired value is a [`DumpError`].
    pub fn pair_f64(&self, path: &Path, key: &str) -> Result<Option<(f64, f64)>, DumpError> {
        match self.get(key) {
            None => Ok(None),
            Some(v) => match v.split_once(',') {
                Some((a, b)) => {
                    let a = parse_scalar(a, path, self.line)?;
                    let b = parse_scalar(b, path, self.line)?;
                    Ok(Some((a, b)))
                }
                None => Err(DumpError::MalformedHexFloat {
                    path: path.to_path_buf(),
                    line: self.line,
                    token: v.to_string(),
                    detail: format!(
                        "'{key}={v}' has no ',' -- not a pair; read it with scalar_f64"
                    ),
                }),
            },
        }
    }

    /// A scalar `value=` token, read through [`Record::scalar_f64`]. Kept as its own method
    /// because `value` is overwhelmingly the key a caller reaches for; `wipe_tower_bbx`'s
    /// `min=`/`max=` pair and `wipe_tower_rib_offset`'s paired `value=` need
    /// [`Record::scalar_f64`] (by key) or [`Record::pair_f64`] instead, since this method's
    /// `Ok(None)` cannot tell "absent" apart from "present but paired".
    pub fn value_f64(&self, path: &Path) -> Result<Option<f64>, DumpError> {
        self.scalar_f64(path, "value")
    }
}

/// Parses one scalar token as either a hex float (`0x`/`-0x` prefix) or a plain decimal integer
/// promoted to `f64` -- the two shapes `oracle/stage-dump/README.md` documents for a `value=`
/// (and `min=`/`max=`) token. Never applied to a `pts=` coordinate, which is always a plain
/// integer parsed with [`str::parse::<i64>`] directly, never through this function.
fn parse_scalar(token: &str, path: &Path, line: usize) -> Result<f64, DumpError> {
    let bare = token.strip_prefix('-').unwrap_or(token);
    if bare.starts_with("0x") || bare.starts_with("0X") {
        parse_hexfloat(token, path, line)
    } else {
        token
            .parse::<i64>()
            .map(|v| v as f64)
            .map_err(|e| DumpError::MalformedInteger {
                path: path.to_path_buf(),
                line,
                token: token.to_string(),
                detail: e.to_string(),
            })
    }
}

/// Parses a C `%a` hexadecimal floating-point literal (`[-]0x<hex>.<hex>p<exp>`), the shape every
/// `coordf_t`/`double`/`float` scalar field is written with (never a coordinate -- see the module
/// doc's rule 1). Built from the mantissa's own hex digits rather than through
/// [`str::parse::<f64>`], which does not accept this syntax at all.
fn parse_hexfloat(token: &str, path: &Path, line: usize) -> Result<f64, DumpError> {
    let malformed = |detail: String| DumpError::MalformedHexFloat {
        path: path.to_path_buf(),
        line,
        token: token.to_string(),
        detail,
    };
    let (neg, rest) = match token.strip_prefix('-') {
        Some(r) => (true, r),
        None => (false, token),
    };
    let rest = rest
        .strip_prefix("0x")
        .or_else(|| rest.strip_prefix("0X"))
        .ok_or_else(|| malformed("missing '0x' prefix".to_string()))?;
    let (mantissa, exp_str) = rest
        .split_once(['p', 'P'])
        .ok_or_else(|| malformed("missing 'p' exponent marker".to_string()))?;
    let exp: i32 = exp_str
        .parse()
        .map_err(|e: std::num::ParseIntError| malformed(format!("bad exponent: {e}")))?;
    let (int_part, frac_part) = match mantissa.split_once('.') {
        Some((i, f)) => (i, f),
        None => (mantissa, ""),
    };
    if int_part.is_empty() && frac_part.is_empty() {
        return Err(malformed("empty mantissa".to_string()));
    }
    let int_val: u64 = if int_part.is_empty() {
        0
    } else {
        u64::from_str_radix(int_part, 16)
            .map_err(|e| malformed(format!("bad integer part '{int_part}': {e}")))?
    };
    let mut value = int_val as f64;
    let mut scale = 1.0_f64 / 16.0;
    for c in frac_part.chars() {
        let digit = c
            .to_digit(16)
            .ok_or_else(|| malformed(format!("bad hex digit '{c}' in fractional part")))?;
        value += digit as f64 * scale;
        scale /= 16.0;
    }
    // Scaled in two halves rather than one `2f64.powi(exp)`: a single `powi` call for an extreme
    // negative exponent (a subnormal double, exp <= -1023 or so) computes 2^-exp as an
    // intermediate that overflows to infinity, so `value / infinity` silently rounds to 0.0
    // instead of the true (tiny, nonzero) subnormal value. Splitting `exp` into two halves keeps
    // each factor's own exponent within the normal range, so each `powi` call -- and so the
    // product -- stays exact (multiplying by a power of two only ever shifts a float's binary
    // exponent, never rounds its mantissa, as long as neither factor over/underflows).
    let half = exp / 2;
    let rest = exp - half;
    value *= 2f64.powi(half);
    value *= 2f64.powi(rest);
    Ok(if neg { -value } else { value })
}

/// One parsed dump file: the two header keys every seam line carries (`seam`, `object`), the
/// `oracle-commit` recorded in its provenance block (already checked against [`PINNED_COMMIT`] by
/// the time a [`Dump`] exists -- see [`parse_dump_text`]), and every body [`Record`] in the
/// file's own stored order. This is "the per-seam state ... keyed by seam, object, layer and
/// region" the module doc promises: `seam`/`object` here, `layer`/`region` (and every other axis
/// key) on each [`Record`].
#[derive(Debug, Clone)]
pub struct Dump {
    pub path: PathBuf,
    pub oracle_commit: String,
    pub seam: String,
    pub object: String,
    pub records: Vec<Record>,
}

impl Dump {
    /// Every record whose `field=<name>` equals `field`, in the file's own stored order -- a
    /// single call away from, for example, every `lslices` line regardless of layer.
    pub fn field<'a>(&'a self, field: &'a str) -> impl Iterator<Item = &'a Record> {
        self.records.iter().filter(move |r| r.field == field)
    }

    /// Record-placing keys that identify *which* record a field's line belongs to beyond
    /// `field`/`layer`/`region`: `support_layer` (a support-layer field indexes
    /// `PrintObject::support_layers()`, a separate collection from `layers()`), `object` (a
    /// print-wide seam's `object_skirt` writes one record per `Print::objects()` entry), and
    /// `group`/`brim` (`psSkirtBrim`'s own `skirt_brim_group_skirt`/`skirt_brim_group_brim`
    /// records, one per `SkirtBrimGroup` and, for a brim, one per brim within it).
    /// `filament_id` is deliberately excluded: `dump_seams.cpp` writes it as an attribute of the
    /// record `group`/`brim` already place, never as a second axis a brim is indexed by.
    const UNCONSTRAINED_AXES: [&'static str; 4] = ["support_layer", "object", "group", "brim"];

    /// The flattened coordinates of every point `field`'s records carry at `(layer, region)` (an
    /// `ExPolygons`/`SurfaceCollection` field summary record contributes no points of its own, so
    /// concatenating every matching record's `points` is exactly the field's full point list at
    /// that layer and region, in stored order). Both keys are matched -- a field with more than
    /// one region per layer (`slices`, `raw_slices`, `thin_fills`, `fill_surfaces`, `perimeters`,
    /// `fills`) would otherwise have every region's points silently merged into one list and
    /// compared as if it were a single polygon, a false pass this harness exists to prevent.
    /// Pass `region: None` for a field with no region axis at all (`lslices`; every print-wide
    /// field). The same protection applies to every other record-placing key this call does not
    /// take as a parameter (see [`Dump::UNCONSTRAINED_AXES`]): a matched record that carries one
    /// of those keys is refused with [`DumpError::AmbiguousAxis`] rather than merged across its
    /// values -- this is why `object_skirt` (`object=`), `skirt_brim_group_skirt`/
    /// `skirt_brim_group_brim` (`group=`/`brim=`) and `support_islands`/`support_fills`
    /// (`support_layer=`) are not reachable through this call at all; the last of those has its
    /// own accessor, [`Dump::points_at_support_layer`]. `Err(DumpError::FieldNotFound)` when no
    /// record matches at all -- refused rather than silently returning an empty `Vec`
    /// indistinguishable from "this polygon has zero points".
    pub fn points_at(
        &self,
        field: &str,
        layer: Option<i64>,
        region: Option<i64>,
    ) -> Result<Vec<i64>, DumpError> {
        let mut matched_any = false;
        let mut out = Vec::new();
        for record in &self.records {
            if record.field != field {
                continue;
            }
            if record.layer(&self.path)? != layer {
                continue;
            }
            if record.region(&self.path)? != region {
                continue;
            }
            let mut ambiguous_key: Option<&'static str> = None;
            for key in Self::UNCONSTRAINED_AXES {
                if record.get(key).is_some() {
                    ambiguous_key = Some(key);
                    break;
                }
            }
            if let Some(key) = ambiguous_key {
                return Err(DumpError::AmbiguousAxis {
                    path: self.path.clone(),
                    field: field.to_string(),
                    key,
                    line: record.line,
                });
            }
            matched_any = true;
            for point in &record.points {
                out.extend(point.iter().copied());
            }
        }
        if !matched_any {
            return Err(DumpError::FieldNotFound {
                path: self.path.clone(),
                field: field.to_string(),
                detail: format!("no record at layer={layer:?} region={region:?}"),
            });
        }
        Ok(out)
    }

    /// The support-layer counterpart to [`Dump::points_at`]: a support-layer field
    /// (`support_islands`, `support_fills`) carries `layer=-` and indexes by `support_layer=<S>`
    /// instead -- a separate collection (`PrintObject::support_layers()`) from `layers()`, so
    /// matching it through `points_at`'s `layer` parameter would silently compare the wrong axis.
    /// A record with no `support_layer` key at all is never a candidate match, regardless of
    /// `support_layer`'s value here -- it is not a support-layer record, so treating its absent
    /// key as equal to `None` would (and, before this check existed, did) silently merge every
    /// layer and region of a non-support field (or the whole, unindexed `support_layer=-`
    /// summary line a support field itself carries no such line for) into one list.
    pub fn points_at_support_layer(
        &self,
        field: &str,
        support_layer: Option<i64>,
    ) -> Result<Vec<i64>, DumpError> {
        let mut matched_any = false;
        let mut out = Vec::new();
        for record in &self.records {
            if record.field != field {
                continue;
            }
            let Some(sl) = record.support_layer(&self.path)? else {
                continue;
            };
            if Some(sl) != support_layer {
                continue;
            }
            matched_any = true;
            for point in &record.points {
                out.extend(point.iter().copied());
            }
        }
        if !matched_any {
            return Err(DumpError::FieldNotFound {
                path: self.path.clone(),
                field: field.to_string(),
                detail: format!("no record at support_layer={support_layer:?}"),
            });
        }
        Ok(out)
    }
}

/// Parses one record's tokens: `field=<name>` first, then every other `key=value` token, with
/// the one exception that once a `pts=` token is seen, it and every token after it (which carry
/// no `=` of their own) are the point list, which ends the line.
fn parse_record_line(line_text: &str, lineno: usize, path: &Path) -> Result<Record, DumpError> {
    let malformed = |reason: String| DumpError::MalformedLine {
        path: path.to_path_buf(),
        line: lineno,
        reason,
    };

    let mut tokens = line_text.split_whitespace();
    let first = tokens
        .next()
        .ok_or_else(|| malformed("empty line".to_string()))?;
    let field = first
        .strip_prefix("field=")
        .ok_or_else(|| malformed(format!("line does not start with 'field=': '{first}'")))?
        .to_string();

    let mut raw: BTreeMap<String, String> = BTreeMap::new();
    let mut points: Vec<Vec<i64>> = Vec::new();

    while let Some(tok) = tokens.next() {
        if let Some(rest) = tok.strip_prefix("pts=") {
            points.push(parse_point(rest, path, lineno)?);
            for more in tokens.by_ref() {
                points.push(parse_point(more, path, lineno)?);
            }
            break;
        } else if let Some((k, v)) = tok.split_once('=') {
            raw.insert(k.to_string(), v.to_string());
        } else {
            return Err(malformed(format!(
                "token has no '=' and is not a 'pts=' tuple: '{tok}'"
            )));
        }
    }

    // `n=<k>` (an `ExPolygons`/`SurfaceCollection` ring or an `ExtrusionEntityCollection` leaf
    // path) declares how many `pts=` tuples the line carries; cross-checked against the number
    // actually parsed, rather than trusting a declared count a truncated or garbled line could
    // disagree with -- the same "refuse rather than compare" rule the header applies, applied
    // here to the body.
    if let Some(n_str) = raw.get("n") {
        let declared: usize = n_str.parse().map_err(|e: std::num::ParseIntError| {
            malformed(format!("'n={n_str}' is not a valid integer: {e}"))
        })?;
        if declared != points.len() {
            return Err(malformed(format!(
                "declared n={declared} does not match the {} 'pts=' tuple(s) actually present",
                points.len()
            )));
        }
    }

    Ok(Record {
        field,
        raw,
        points,
        line: lineno,
    })
}

/// Parses one `x,y` or `x,y,z` coordinate tuple -- every component through [`str::parse::<i64>`],
/// never through a float (the module doc's rule 1) -- and refuses a tuple whose component count
/// is neither 2 nor 3, the only two shapes [`Record::points`]' own doc promises a caller.
fn parse_point(tok: &str, path: &Path, lineno: usize) -> Result<Vec<i64>, DumpError> {
    let point: Vec<i64> = tok
        .split(',')
        .map(|part| {
            part.parse::<i64>()
                .map_err(|e| DumpError::MalformedInteger {
                    path: path.to_path_buf(),
                    line: lineno,
                    token: tok.to_string(),
                    detail: e.to_string(),
                })
        })
        .collect::<Result<_, _>>()?;
    if point.len() != 2 && point.len() != 3 {
        return Err(DumpError::MalformedLine {
            path: path.to_path_buf(),
            line: lineno,
            reason: format!(
                "'pts=' tuple '{tok}' has {} component(s), expected 2 (x,y) or 3 (x,y,z)",
                point.len()
            ),
        });
    }
    Ok(point)
}

/// The pure body-line parser: skips every blank or `;`-prefixed line (a dump's header, wherever
/// it appears) and parses every other line as a [`Record`]. Carries no provenance requirement at
/// all -- rule 2 of the module doc is [`parse_dump_text`]'s job, not this function's. Private
/// rather than exposed directly: a public, generically-named "parse any dump text" function
/// would be a second, unguarded way into a dump's body that bypasses rule 2 entirely, for every
/// caller, not only the one hand-written fixture that genuinely needs it. See
/// [`parse_hand_written_fixture_records`] for that one caller's own, specifically-named entry
/// point.
fn parse_records(text: &str, path: &Path) -> Result<Vec<Record>, DumpError> {
    let mut records = Vec::new();
    for (idx, line) in text.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with(';') {
            continue;
        }
        records.push(parse_record_line(trimmed, idx + 1, path)?);
    }
    Ok(records)
}

/// The one exception to rule 2 (see the module doc): reads
/// `crates/slic3r-proof/tests/data/oversized-coordinate.dump`-shaped text with no provenance
/// header requirement at all. Named for that one caller rather than for general use -- a
/// consumer reaching for a dump reader should find [`read_dump`]/[`read_dump_default`]/
/// [`parse_dump_text`] instead, each of which does enforce rule 2 -- and is otherwise exactly
/// [`parse_records`], the body-line parser every real dump is also read through internally (via
/// [`parse_dump_text`]), kept private so this is the only way to reach it without a provenance
/// check.
pub fn parse_hand_written_fixture_records(
    text: &str,
    path: &Path,
) -> Result<Vec<Record>, DumpError> {
    parse_records(text, path)
}

/// Parses `key: value` out of one leading `;`-comment line, the shape both the indented
/// provenance sub-block (`;   oracle-commit: ...`) and the `; seam: ...`/`; object: ...` lines
/// use -- stripped of its leading `;` and surrounding whitespace first, so the indentation
/// difference between the two blocks never matters.
fn parse_header_line(line: &str) -> Option<(String, String)> {
    let body = line.trim().strip_prefix(';')?.trim();
    let (key, value) = body.split_once(':')?;
    Some((key.trim().to_string(), value.trim().to_string()))
}

/// Parses `text`'s leading run of blank-or-`;` lines into a `key -> value` map (last occurrence
/// wins, which never matters here since every key `read_dump` needs appears at most once), then
/// validates `oracle-commit` against [`PINNED_COMMIT`] -- rule 2 of the module doc. `display_path`
/// names the file in every error and in the returned [`Dump`]; it need not be where `text` was
/// actually read from (the header/mismatch tests construct `text` in memory and name a synthetic
/// path for their error message's sake).
pub fn parse_dump_text(text: &str, display_path: &Path) -> Result<Dump, DumpError> {
    let mut header: BTreeMap<String, String> = BTreeMap::new();
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if !trimmed.starts_with(';') {
            break;
        }
        if let Some((k, v)) = parse_header_line(trimmed) {
            header.insert(k, v);
        }
    }

    let oracle_commit =
        header
            .get("oracle-commit")
            .cloned()
            .ok_or_else(|| DumpError::MissingProvenanceHeader {
                path: display_path.to_path_buf(),
            })?;
    if oracle_commit != PINNED_COMMIT {
        return Err(DumpError::ProvenanceMismatch {
            path: display_path.to_path_buf(),
            found: oracle_commit.clone(),
            expected: PINNED_COMMIT,
        });
    }

    let seam = header.get("seam").cloned().unwrap_or_default();
    let object = header.get("object").cloned().unwrap_or_default();
    let records = parse_records(text, display_path)?;

    Ok(Dump {
        path: display_path.to_path_buf(),
        oracle_commit,
        seam,
        object,
        records,
    })
}

/// Reads and parses `<dumps_dir>/<pair>/<seam>.dump` -- `pair` is the committed
/// `<fixture-stem>-<configuration>` directory name (`20mm_cube-base-fff` for the corpus this
/// branch's own test reads). Refuses (named by file) a missing file, a missing provenance header,
/// or one that disagrees with [`PINNED_COMMIT`] -- never a comparison against a stale or foreign
/// dump.
pub fn read_dump(dumps_dir: &Path, pair: &str, seam: &str) -> Result<Dump, DumpError> {
    let path = dumps_dir.join(pair).join(format!("{seam}.dump"));
    let text = fs::read_to_string(&path).map_err(|e| DumpError::Io {
        path: path.clone(),
        io_error: e.to_string(),
    })?;
    parse_dump_text(&text, &path)
}

/// `read_dump` at [`resolve_dumps_path`]'s directory -- the convenience a unit's test calls,
/// mirroring `tiers::compare_unit`'s relationship to `tiers::compare_unit_at`.
pub fn read_dump_default(pair: &str, seam: &str) -> Result<Dump, DumpError> {
    read_dump(&resolve_dumps_path(), pair, seam)
}
