//! Proof path 3 bridge for task `64-foundation-print-config` (`port/manifest.tsv`'s
//! `PrintConfig.cpp`/`PrintConfig.hpp` rows). See `cxx/proof_units/print_config.cpp`'s own
//! header comment for why this shim is shaped like proof path 1 against the externally linked
//! `Slic3r::print_config_def` despite the rows' seeded `proof path` cell of `3` --
//! `cxx/proof_units/material_type.cpp` and `cxx/proof_units/utils.cpp` record the same kind of
//! mismatch for the same reason (a `.cpp` with too many externally-linked symbols for an
//! `#include`-whole to avoid a duplicate-symbol link failure).
//!
//! `oracle_option_keys`/`oracle_option_info` turn the registry's whole key set into one
//! mechanically comparable table carrying **every** field of `ConfigOptionDef`
//! (`Config.hpp:2315-2637`) that the registry's `init_*` bodies can set: the behaviourally
//! load-bearing ones (type, nullability, default value, enum name/value map, CLI spelling) and
//! the GUI-presentation ones (label, tooltip, category, sidetext, mode, `gui_type`, the numeric
//! `min`/`max`/`max_literal`, the alias and shortcut lists, ...) alike.
//!
//! **Why the presentation fields are in the proof and not left to review.** This task's own
//! plan scoped them to the two-pass adversarial review, on the reading that `L()` wraps them
//! and a translated string could not be compared. That reading is wrong at the pin:
//! `vendor/OrcaSlicer/src/libslic3r/I18N.hpp:31` defines `L` inside libslic3r as
//! `const char* L(const char* s) { return s; }` -- the identity extraction marker, not a
//! lookup -- so `def->label` holds the English literal the vendored line spells, and it
//! compares bytewise. Only `_u8L` (`I18N.hpp:33`) goes through `I18N::translate`, and no
//! `ConfigOptionDef` field in `PrintConfig.cpp`'s registry bodies is assigned from it. Leaving
//! these fields out would leave the label, tooltip, category, sidetext, mode and `gui_type` of
//! all 881 rows checked by nothing at all, which is where a mechanical transcription of 881
//! near-identical rows would hide its errors.
//!
//! The three enum-valued fields (`gui_type`, `printer_technology`, `mode`) cross as `i32` casts
//! of the C++ enumerator, which is sound because `slic3r-config` transcribes all three
//! enumerator lists in the vendored declaration order
//! (`ConfigOptionDef::GUIType`, `Config.hpp:2318-2339`; `PrinterTechnology`, `:228-238`;
//! `ConfigOptionMode`, `:221-226`), so the two sides' discriminants agree by construction and
//! the comparison would fail loudly if one ever did not.
//!
//! `serialization_key_ordinal` is deliberately **not** exposed: `slic3r-config` draws it from a
//! process-wide atomic counter rather than from the registry's own construction order, so it
//! depends on how many registries the process has built and is not a property of the
//! transcription. It is named as a finding in this task's pull request body instead.

#[cxx::bridge(namespace = "slic3r_ffi::proof_units::print_config")]
mod ffi {
    /// One `ConfigOptionDef::enum_keys_map` entry (`Config.hpp:2619` for the field, `Config.hpp:2072`
    /// for the `t_config_enum_values` typedef it points at, `std::map<std::string, int>`).
    struct OracleEnumKeyValue {
        name: String,
        value: i32,
    }

    /// One oracle `ConfigOptionDef` (`Config.hpp:2315-2637`), field for field, less
    /// `opt_key` (which is the lookup key) and `serialization_key_ordinal` (see this module's
    /// own doc comment). Everything here is held to exact equality by this task's proof.
    /// `found = false` means the key is not in the oracle's `print_config_def.options` at all
    /// (an unknown key, or one the port invented).
    struct OracleOptionInfo {
        found: bool,
        option_type: i32,
        nullable: bool,
        has_default: bool,
        default_serialized: String,
        gui_type: i32,
        gui_flags: String,
        plugin_type: String,
        label: String,
        full_label: String,
        printer_technology: i32,
        category: String,
        tooltip: String,
        sidetext: String,
        cli: String,
        cli_params: String,
        ratio_over: String,
        multiline: bool,
        full_width: bool,
        is_code: bool,
        readonly: bool,
        height: i32,
        width: i32,
        min: f32,
        max: f32,
        max_literal: f64,
        mode: i32,
        aliases: Vec<String>,
        shortcut: Vec<String>,
        enum_values: Vec<String>,
        enum_labels: Vec<String>,
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

        /// One key's whole `ConfigOptionDef`, straight off
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

        /// The key set of one of the other 13 `ConfigDef` subclasses
        /// (`PrintConfig.hpp:2417-2501`), by class name -- "CLIActionsConfigDef",
        /// "TemperaturesConfigDef", and so on. None of the 13 enters `print_config_def`, so
        /// none is covered by `oracle_option_keys`. Empty for an unknown name, which the
        /// port's own proof asserts against so a typo fails loudly rather than vacuously.
        fn oracle_other_def_keys(which: &str) -> Vec<String>;

        /// One key's whole `ConfigOptionDef` from one of those 13 tables, read through the
        /// same field list `oracle_option_info` uses.
        fn oracle_other_def_option_info(which: &str, key: &str) -> OracleOptionInfo;

        /// The keys of `custom_gcode_specific_placeholders()`
        /// (`PrintConfig.cpp:13643-13646`), in that `std::map`'s own order.
        fn oracle_custom_gcode_placeholder_keys() -> Vec<String>;

        /// One custom G-code's placeholder list, in that `std::vector`'s own
        /// braced-initialiser order rather than sorted.
        fn oracle_custom_gcode_placeholders(key: &str) -> Vec<String>;

        /// One `StaticPrintConfig`-derived class's `defaults().keys_ref()`
        /// (`PrintConfig.hpp:1147`), by class name -- the list `StaticCache::finalize` filled
        /// by walking the registry in order and keeping each key the class has a field for.
        /// Empty for an unknown name, which the port's own proof asserts against.
        fn oracle_static_class_keys(which: &str) -> Vec<String>;

        /// The same class's defaults, each key's `optptr(key)->serialize()` in the same order.
        /// This is what checks the cache wiring rather than only the field list: a field of the
        /// wrong option type, or one whose name does not match the registry row it should
        /// carry, serializes differently from the default `finalize` assigned it.
        fn oracle_static_class_defaults(which: &str) -> Vec<String>;
    }
}

pub use ffi::{
    oracle_custom_gcode_placeholder_keys, oracle_custom_gcode_placeholders,
    oracle_extruder_option_keys, oracle_extruder_retract_keys, oracle_filament_option_keys,
    oracle_filament_retract_keys, oracle_handle_legacy, oracle_option_info, oracle_option_keys,
    oracle_other_def_keys, oracle_other_def_option_info, oracle_static_class_defaults,
    oracle_static_class_keys, OracleEnumKeyValue, OracleLegacyResult, OracleOptionInfo,
};
