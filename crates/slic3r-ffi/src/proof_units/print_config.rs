//! Proof path 3 bridge for task `64-foundation-print-config` (`port/manifest.tsv`'s
//! `PrintConfig.cpp`/`PrintConfig.hpp` rows). See `cxx/proof_units/print_config.cpp`'s own
//! header comment for why this shim is shaped like proof path 1 against the externally linked
//! `Slic3r::print_config_def` despite the rows' seeded `proof path` cell of `3` --
//! `cxx/proof_units/material_type.cpp` and `cxx/proof_units/utils.cpp` record the same kind of
//! mismatch for the same reason (a `.cpp` with too many externally-linked symbols for an
//! `#include`-whole to avoid a duplicate-symbol link failure).
//!
//! `oracle_option_keys`/`oracle_option_info` turn the registry's whole key set into one
//! mechanically comparable table: the behaviourally load-bearing fields (type, default value,
//! enum name/value map, CLI spelling) rather than the GUI-presentation ones (label, tooltip,
//! category, ...), which this task's own plan scopes to the two-pass adversarial review instead
//! (`intent/libslic3r-rust-port/tasks/64-foundation-print-config/plan.md`, "The proof design").

#[cxx::bridge(namespace = "slic3r_ffi::proof_units::print_config")]
mod ffi {
    /// One `ConfigOptionDef::enum_keys_map` entry (`Config.hpp:2619` for the field, `Config.hpp:2072`
    /// for the `t_config_enum_values` typedef it points at, `std::map<std::string, int>`).
    struct OracleEnumKeyValue {
        name: String,
        value: i32,
    }

    /// The behaviourally load-bearing slice of one oracle `ConfigOptionDef`: everything this
    /// task's proof holds to exact equality. `found = false` means the key is not in the
    /// oracle's `print_config_def.options` at all (an unknown key, or one the port invented).
    struct OracleOptionInfo {
        found: bool,
        option_type: i32,
        nullable: bool,
        cli: String,
        has_default: bool,
        default_serialized: String,
        enum_values: Vec<String>,
        enum_keys_map: Vec<OracleEnumKeyValue>,
    }

    /// Both arguments of `PrintConfigDef::handle_legacy` after it has rewritten them in place
    /// (`PrintConfig.cpp:9748-10041`). An empty `opt_key` is the vendored function's own "this
    /// key is no longer valid in this version of Slic3r" answer, not an error.
    struct OracleLegacyResult {
        opt_key: String,
        value: String,
    }

    // SAFETY: this is cxx's own unsafe-extern declaration block, not a hand-written unsafe
    // expression (see src/clipper2.rs's identical note) -- it carries no body, and the only
    // unsafety is in the bindings `cxx` generates from it.
    unsafe extern "C++" {
        include!("slic3r-ffi/cxx/proof_units/print_config.h");

        /// The full key set of `Slic3r::print_config_def.options`, in that `std::map`'s own
        /// iteration order (alphabetical by key -- `// FIDELITY(port):` in the port's own
        /// table-builder, since this order has to match for the comparison test to zip the two
        /// lists positionally as well as by key).
        fn oracle_option_keys() -> Vec<String>;

        /// One key's behaviourally load-bearing metadata, straight off
        /// `Slic3r::print_config_def.get(key)`.
        fn oracle_option_info(key: &str) -> OracleOptionInfo;

        /// `PrintConfigDef::handle_legacy`, with both of its in-place-rewritten arguments
        /// handed back. Used by this unit's proof to hold the ported `handle_legacy` to exact
        /// equality over every key the registry knows plus the obsolete and renamed ones.
        fn oracle_handle_legacy(opt_key: &str, value: &str) -> OracleLegacyResult;

        /// `print_config_def.extruder_option_keys()` (`PrintConfig.hpp:852`), in the
        /// `std::vector`'s own braced-initialiser order rather than sorted.
        fn oracle_extruder_option_keys() -> Vec<String>;

        /// `print_config_def.extruder_retract_keys()` (`PrintConfig.hpp:856`).
        fn oracle_extruder_retract_keys() -> Vec<String>;

        /// `print_config_def.filament_option_keys()` (`PrintConfig.hpp:859`). Empty at the
        /// pin: `init_filament_option_keys` is never called -- see the C++ shim's header.
        fn oracle_filament_option_keys() -> Vec<String>;

        /// `print_config_def.filament_retract_keys()` (`PrintConfig.hpp:860`). Empty at the
        /// pin, for the same reason.
        fn oracle_filament_retract_keys() -> Vec<String>;
    }
}

pub use ffi::{
    oracle_extruder_option_keys, oracle_extruder_retract_keys, oracle_filament_option_keys,
    oracle_filament_retract_keys, oracle_handle_legacy, oracle_option_info, oracle_option_keys,
    OracleEnumKeyValue, OracleLegacyResult, OracleOptionInfo,
};
