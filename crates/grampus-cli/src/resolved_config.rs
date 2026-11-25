//! The port's configuration input (R11): a resolved configuration the oracle writes with
//! `--export-settings` on an unarmed run, committed under `port/goldens/configs/` by task
//! `12-config-goldens`. Bed shape (`printable_area`), exclusion areas (`bed_exclude_area`,
//! `bed_exclude_volume_mode`, ...), wipe-tower placement (`wipe_tower_x`, `wipe_tower_y`, ...)
//! and filament maps (`filament_map`, ...) reach the port as resolved configuration values
//! through this reader and in no other way: nothing here reads
//! `vendor/OrcaSlicer/resources/profiles/`, a preset file or an `inherits` chain.
//!
//! **The Bambu-printer flag is not one of these keys.** The oracle assigns it from the printer's
//! vendor preset, not from the resolved configuration, at `src/OrcaSlicer.cpp:7026`
//! (`(dynamic_cast<Print*>(print))->is_BBL_printer() = is_bbl_vendor_preset;`), and
//! `Print.hpp:1351-1353` keeps it as a plain field; `src/libslic3r/GCode.cpp:3437` reads it at
//! emission. So a golden never carries it, and `ResolvedConfig` exposes it as a value the
//! caller sets explicitly (`set_bambu_printer`) rather than one this reader derives from the
//! JSON object.
//!
//! **Two keys exist only in the G-code config block and are derived, not read.**
//! `first_layer_temperature` and `first_layer_bed_temperature` are absent from every golden
//! `--export-settings` writes; the oracle computes them at emission time
//! (`src/libslic3r/GCode.cpp:3720-3724`, written into the G-code at `:4333-4334`). Reading one
//! does not fail: `get` reports [`ConfigField::DerivedAtEmission`] for both, so the Phase B
//! emission module (not this task) takes them from the surface rather than from the golden.

use std::collections::BTreeMap;
use std::fmt;
use std::fs;
use std::path::Path;

/// The two keys the G-code config block carries that `--export-settings` never does, because
/// the oracle computes them at emission time rather than at resolution time. Kept as a small
/// fixed list, not inferred from the JSON, so a porter reads the citation above rather than the
/// absence of a key in one golden.
const DERIVED_AT_EMISSION: &[&str] = &["first_layer_temperature", "first_layer_bed_temperature"];

/// The header key every golden carries: provenance about how it was produced (oracle commit,
/// platform, preset set, fixture, transform — plan step 3), never a configuration value.
const PROVENANCE_KEY: &str = "provenance";

#[derive(Debug)]
pub enum ResolvedConfigError {
    /// The golden could not be read from disk.
    Io {
        path: String,
        source: std::io::Error,
    },
    /// The golden's bytes did not parse as JSON.
    Parse {
        path: String,
        source: serde_json::Error,
    },
    /// The golden parsed, but not to a JSON object (every golden is a flat object of config
    /// keys plus the `provenance` header).
    NotAnObject { path: String },
    /// The golden carries no `provenance` header (R7's provenance object, plan step 3). A
    /// golden with no header is not treated as a configuration with an empty one; it is
    /// rejected, because an unheaded golden cannot be attributed to an oracle commit or a
    /// fixture/transform pair.
    MissingProvenance { path: String },
    /// The golden's `provenance` key is present but is not itself a JSON object.
    ProvenanceNotAnObject { path: String },
}

impl fmt::Display for ResolvedConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ResolvedConfigError::Io { path, source } => {
                write!(f, "cannot read resolved config '{path}': {source}")
            }
            ResolvedConfigError::Parse { path, source } => {
                write!(f, "resolved config '{path}' is not valid JSON: {source}")
            }
            ResolvedConfigError::NotAnObject { path } => {
                write!(f, "resolved config '{path}' is not a JSON object")
            }
            ResolvedConfigError::MissingProvenance { path } => {
                write!(
                    f,
                    "resolved config '{path}' carries no '{PROVENANCE_KEY}' header"
                )
            }
            ResolvedConfigError::ProvenanceNotAnObject { path } => {
                write!(
                    f,
                    "resolved config '{path}': '{PROVENANCE_KEY}' is not a JSON object"
                )
            }
        }
    }
}

impl std::error::Error for ResolvedConfigError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ResolvedConfigError::Io { source, .. } => Some(source),
            ResolvedConfigError::Parse { source, .. } => Some(source),
            _ => None,
        }
    }
}

/// One key's reading, returned by [`ResolvedConfig::get`].
#[derive(Debug, Clone, PartialEq)]
pub enum ConfigField {
    /// The key's value, read verbatim from the golden's JSON object.
    Value(serde_json::Value),
    /// `first_layer_temperature` / `first_layer_bed_temperature`: not a read failure, but a
    /// report that the oracle computes this key at emission time rather than at resolution
    /// time, so the golden never carries it (see this module's doc comment).
    DerivedAtEmission,
}

/// The oracle's resolved configuration (R11), read from a committed golden under
/// `port/goldens/configs/`, with its `provenance` header split out.
#[derive(Debug)]
pub struct ResolvedConfig {
    path: String,
    values: BTreeMap<String, serde_json::Value>,
    provenance: serde_json::Map<String, serde_json::Value>,
    /// Set by the caller (`set_bambu_printer`), never derived from `values`: see this module's
    /// doc comment for why the golden itself never carries this flag.
    bambu_printer: Option<bool>,
}

impl ResolvedConfig {
    /// Reads and parses `path` as a resolved-configuration golden. Fails with a named
    /// [`ResolvedConfigError`] variant — never a panic — on an unreadable file, malformed JSON,
    /// a JSON value that is not an object, or an object with no `provenance` header. Does
    /// **not** fail when the two derive-at-emission keys (`first_layer_temperature`,
    /// `first_layer_bed_temperature`) are absent, since every real golden omits them by
    /// construction.
    pub fn from_golden(path: &Path) -> Result<Self, ResolvedConfigError> {
        let path_str = path.display().to_string();

        let bytes = fs::read_to_string(path).map_err(|source| ResolvedConfigError::Io {
            path: path_str.clone(),
            source,
        })?;

        let parsed: serde_json::Value =
            serde_json::from_str(&bytes).map_err(|source| ResolvedConfigError::Parse {
                path: path_str.clone(),
                source,
            })?;

        let mut object = match parsed {
            serde_json::Value::Object(object) => object,
            _ => {
                return Err(ResolvedConfigError::NotAnObject {
                    path: path_str.clone(),
                })
            }
        };

        let provenance_value = object.remove(PROVENANCE_KEY).ok_or_else(|| {
            ResolvedConfigError::MissingProvenance {
                path: path_str.clone(),
            }
        })?;

        let provenance = match provenance_value {
            serde_json::Value::Object(provenance) => provenance,
            _ => {
                return Err(ResolvedConfigError::ProvenanceNotAnObject {
                    path: path_str.clone(),
                })
            }
        };

        let values = object.into_iter().collect();

        Ok(ResolvedConfig {
            path: path_str,
            values,
            provenance,
            bambu_printer: None,
        })
    }

    /// The golden's own path, for error messages and for citing which fixture/transform a
    /// `Surface` was built from.
    pub fn path(&self) -> &str {
        &self.path
    }

    /// The `provenance` header (plan step 3): oracle commit, platform, preset set, fixture and
    /// transform. Never one of the configuration keys `get` reads.
    pub fn provenance(&self) -> &serde_json::Map<String, serde_json::Value> {
        &self.provenance
    }

    /// Reads one configuration key. `None` means the golden carries no such key and it is not
    /// one of the two derived-at-emission keys either; `Some(ConfigField::DerivedAtEmission)`
    /// means the key is read from the surface at G-code emission time, not from this golden
    /// (see this module's doc comment); `Some(ConfigField::Value(_))` is the key's value, read
    /// back unchanged from the JSON this reader parsed.
    pub fn get(&self, key: &str) -> Option<ConfigField> {
        if DERIVED_AT_EMISSION.contains(&key) {
            return Some(ConfigField::DerivedAtEmission);
        }
        self.values.get(key).cloned().map(ConfigField::Value)
    }

    /// The explicit Bambu-printer flag the caller has set, or `None` before `set_bambu_printer`
    /// is called. Validation and G-code emission (Phase B) need this to be `Some`; this task
    /// only carries the value through, since nothing underneath reads it yet.
    pub fn bambu_printer(&self) -> Option<bool> {
        self.bambu_printer
    }

    /// Sets the Bambu-printer flag explicitly, as `src/OrcaSlicer.cpp:7026` sets it on the
    /// oracle's own `Print` from the printer's vendor preset — never derived from this golden's
    /// JSON object, which carries no such key.
    pub fn set_bambu_printer(&mut self, is_bbl_printer: bool) {
        self.bambu_printer = Some(is_bbl_printer);
    }
}
