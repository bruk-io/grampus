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
    /// One `ConfigOptionDef::enum_keys_map` entry (`Config.hpp:2356` / `t_config_enum_values`,
    /// `std::map<std::string, int>`).
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
    }
}

pub use ffi::{oracle_option_info, oracle_option_keys, OracleEnumKeyValue, OracleOptionInfo};
