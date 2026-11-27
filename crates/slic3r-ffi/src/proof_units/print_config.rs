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

        /// One of the seven option-set globals (`PrintConfig.cpp:10105-10305`), by variable
        /// name, in the `std::set`'s own order. Empty for an unknown name -- and for
        /// `empty_options`, which really is empty at the pin.
        fn oracle_option_set(which: &str) -> Vec<String>;

        /// `get_shaper_type_values_for_flavor(GCodeFlavor)` (`PrintConfig.cpp:159-173`).
        fn oracle_shaper_type_values_for_flavor(flavor: i32) -> Vec<String>;

        /// `get_extruder_variant_string(ExtruderType, NozzleVolumeType)`
        /// (`PrintConfig.cpp:777-796`).
        fn oracle_extruder_variant_string(extruder_type: i32, nozzle_volume_type: i32) -> String;

        /// `get_nozzle_volume_type_string(NozzleVolumeType)` (`PrintConfig.cpp:860-867`).
        fn oracle_nozzle_volume_type_string(nozzle_volume_type: i32) -> String;

        /// `convert_to_nvt_type(const std::string&)` (`PrintConfig.cpp:908-938`), as the
        /// returned enumerator's underlying value.
        fn oracle_convert_to_nvt_type(variant_str: &str) -> i32;

        /// `find_variant_index(...)` (`PrintConfig.cpp:798-815`).
        fn oracle_find_variant_index(
            variant: &str,
            variant_id_1based: i32,
            variant_list: &Vec<String>,
            variant_ids_1based: &Vec<i32>,
        ) -> i32;

        /// `map_variant_indices(...)` (`PrintConfig.cpp:817-831`).
        fn oracle_map_variant_indices(
            variants: &Vec<String>,
            ids: &Vec<i32>,
            from_variants: &Vec<String>,
            from_ids: &Vec<i32>,
        ) -> Vec<i32>;

        /// `get_config_index_base(...)` (`PrintConfig.cpp:833-838`).
        fn oracle_config_index_base(
            volume_type: i32,
            extruder_type: i32,
            variant_id_1based: i32,
            variant_list: &Vec<String>,
            variant_ids_1based: &Vec<i32>,
        ) -> i32;

        /// `save_extruder_ams_count_to_string(get_extruder_ams_count(strs))`
        /// (`PrintConfig.cpp:869-906`): one call covering the parse and its inverse, so a
        /// parse difference shows up as a different saved string.
        fn oracle_extruder_ams_count_roundtrip(strs: &Vec<String>) -> Vec<String>;

        /// `save_extruder_nozzle_stats_to_string(get_extruder_nozzle_stats(strs))`
        /// (`PrintConfig.cpp:940-989`), the same round trip for the nozzle-stats pair.
        fn oracle_extruder_nozzle_stats_roundtrip(strs: &Vec<String>) -> Vec<String>;

        /// `get_extruder_index(GCodeConfig::defaults(), filament_id)`
        /// (`PrintConfig.cpp:140-146`).
        fn oracle_extruder_index_of_defaults(filament_id: u32) -> u64;

        /// `nozzle_diameter_for_filament(PrintConfig::defaults(), filament_id, is_bbl_printer)`
        /// (`PrintConfig.cpp:148-156`).
        fn oracle_nozzle_diameter_for_filament_of_defaults(
            filament_id: i32,
            is_bbl_printer: bool,
        ) -> f64;

        /// `DynamicPrintConfig::full_print_config()` (`PrintConfig.cpp:10307-10310`), as every
        /// key in the `std::map`'s own order, one `"<key>=<serialized>"` entry each.
        fn oracle_full_print_config_serialized() -> Vec<String>;

        /// `DynamicPrintConfig::new_from_defaults_keys(keys)` (`:10316-10321`), serialized the
        /// same way.
        fn oracle_new_from_defaults_keys_serialized(keys: &Vec<String>) -> Vec<String>;

        /// `DynamicPrintConfig(const StaticPrintConfig&)` (`:10312-10314`) over one of the
        /// **eight non-derived** static classes by name. The three derived ones inherit
        /// `StaticPrintConfig` through each parent, so a conversion to it is ambiguous and the
        /// vendored constructor cannot be called with one; empty for those and for an unknown
        /// name.
        fn oracle_dynamic_from_static_serialized(which: &str) -> Vec<String>;

        /// `DynamicPrintConfig::get_parameter_size(param_name, extruder_nums)`
        /// (`:10565-10595`), against a config built from `full_print_config()`.
        fn oracle_parameter_size_of_full(param_name: &str, extruder_nums: u64) -> u64;

        /// `min_object_distance(const ConfigBase&)` (`:10323-10348`), against a config carrying
        /// exactly the given overrides; an empty string leaves that key absent, which is what
        /// the function's two null checks and its `ptUnknown` fallback turn on.
        fn oracle_min_object_distance(
            print_sequence: &str,
            clearance_radius: &str,
            printer_technology: &str,
        ) -> f64;

        /// `get_filament_type(displayed, id)` (`:10698-10771`), `get_filament_vendor()` and
        /// `get_filament_type()` (`:11552-11575`), against a config carrying exactly the given
        /// option values -- `"?"` for a key left absent, `""` for one set to an empty vector,
        /// which is the other case the body's "treat an empty option the same as a missing one"
        /// guard covers. The result is
        /// `[returned type, displayed type, vendor, first filament_type]`.
        fn oracle_filament_type_probe(
            filament_type: &str,
            filament_id: &str,
            filament_is_support: &str,
            filament_vendor: &str,
            id: u64,
        ) -> Vec<String>;

        /// The config the shim builds from `keys_and_values`, serialized, with no method
        /// called on it -- what proves both sides start from the same config before a method
        /// comparison rather than assuming two construction routes agree.
        fn oracle_config_from(keys_and_values: &Vec<String>) -> Vec<String>;

        /// The config `config_from_base` builds, serialized, with no method called. `base` is
        /// `"empty"` for a bare config or `"full"` for `full_print_config()`; the latter exists
        /// because several vendored bodies dereference an option pointer without checking it
        /// and **segfault** on a config missing the key -- see the C++ shim's own note.
        fn oracle_config_from_base(base: &str, keys_and_values: &Vec<String>) -> Vec<String>;

        /// `DynamicPrintConfig::normalize_fdm(used_filaments)`
        /// (`PrintConfig.cpp:10350-10425`), with the whole config handed back.
        fn oracle_normalize_fdm(
            base: &str,
            keys_and_values: &Vec<String>,
            used_filaments: i32,
        ) -> Vec<String>;

        /// `DynamicPrintConfig::set_num_extruders(n)` (`:10636-10654`).
        fn oracle_set_num_extruders(
            base: &str,
            keys_and_values: &Vec<String>,
            num_extruders: u32,
        ) -> Vec<String>;

        /// `DynamicPrintConfig::set_num_filaments(n)` (`:10656-10678`).
        fn oracle_set_num_filaments(
            base: &str,
            keys_and_values: &Vec<String>,
            num_filaments: u32,
        ) -> Vec<String>;

        /// `normalize_filament_values_to_variants(DynamicPrintConfig&)` (`:11404-11430`).
        fn oracle_normalize_filament_values_to_variants(
            keys_and_values: &Vec<String>,
        ) -> Vec<String>;

        /// `set_filament_dev_options(DynamicPrintConfig&, filaments)` (`:11432-11446`), with
        /// two filament configs; an empty list leaves that filament out of the vector.
        fn oracle_set_filament_dev_options(
            keys_and_values: &Vec<String>,
            filament_a: &Vec<String>,
            filament_b: &Vec<String>,
        ) -> Vec<String>;

        /// `has_skirt(const DynamicPrintConfig&)` (`:14376-14383`) as 0/1, -1 on a refusal.
        fn oracle_has_skirt(base: &str, keys_and_values: &Vec<String>) -> i32;

        /// `get_real_skirt_dist(const DynamicPrintConfig&)` (`:14384-14416`), widened to
        /// `double` so the bridge carries the `float` exactly; NaN on a refusal.
        fn oracle_real_skirt_dist(base: &str, keys_and_values: &Vec<String>) -> f64;

        /// `is_XL_printer(const DynamicPrintConfig&)` (`:14424-14428`) as 0/1.
        fn oracle_is_xl_printer_dynamic(base: &str, keys_and_values: &Vec<String>) -> i32;

        /// `is_XL_printer(const PrintConfig&)` (`:14430-14433`) against
        /// `PrintConfig::defaults()`, the only instance a shim can build without a preset.
        fn oracle_is_xl_printer_of_print_defaults() -> i32;

        /// `DynamicPrintAndCLIConfig::def()`'s key set (`PrintConfig.hpp:2519`, `:2531-2538`):
        /// the union of the registry and the three command line tables, in the map's own order.
        fn oracle_print_and_cli_def_keys() -> Vec<String>;

        /// `update_static_print_config_from_dynamic` (`PrintConfig.cpp:12393-12413`), with
        /// the target config handed back whole.
        fn oracle_update_static_print_config_from_dynamic(
            base: &str,
            target_kv: &Vec<String>,
            dest_kv: &Vec<String>,
            variant_index: &Vec<i32>,
            key_set1: &Vec<String>,
            stride: i32,
        ) -> Vec<String>;

        /// `update_diff_values_to_child_config` (`PrintConfig.cpp:12271-12356`), with the
        /// receiver handed back whole. `["<threw>"]` when the vendored body threw.
        fn oracle_update_diff_values_to_child_config(
            base: &str,
            self_kv: &Vec<String>,
            new_kv: &Vec<String>,
            extruder_id_name: &str,
            extruder_variant_name: &str,
            key_set1: &Vec<String>,
            key_set2: &Vec<String>,
        ) -> Vec<String>;

        /// `update_non_diff_values_to_base_config` (`:12141-12269`), the same shape.
        fn oracle_update_non_diff_values_to_base_config(
            base: &str,
            self_kv: &Vec<String>,
            new_kv: &Vec<String>,
            keys: &Vec<String>,
            different_keys: &Vec<String>,
            extruder_id_name: &str,
            extruder_variant_name: &str,
            key_set1: &Vec<String>,
            key_set2: &Vec<String>,
        ) -> Vec<String>;

        /// `update_values_from_single_to_multi` (`PrintConfig.cpp:10891-10993`): the
        /// method's `int` return as the first entry, then the receiver serialized whole.
        fn oracle_update_values_from_single_to_multi(
            base: &str,
            self_kv: &Vec<String>,
            multi_kv: &Vec<String>,
            key_set: &Vec<String>,
            id_name: &str,
            variant_name: &str,
        ) -> Vec<String>;

        /// `set_variant_override` (`:11394-11402`) over one `coFloats` pair, as the target's
        /// serialized value after the override.
        fn oracle_set_variant_override(
            target_value: &str,
            source_value: &str,
            variant_index: &Vec<i32>,
            stride: i32,
        ) -> String;

        /// `DynamicPrintAndCLIConfig::handle_legacy` (`PrintConfig.cpp:13267-13274`), which
        /// delegates to `PrintConfigDef::handle_legacy` only for a key none of the three
        /// command line tables names.
        fn oracle_print_and_cli_handle_legacy(opt_key: &str, value: &str) -> OracleLegacyResult;

        /// `get_bed_shape(const SLAPrinterConfig&)` (`:13854`) against
        /// `SLAPrinterConfig::defaults()`, as flattened `x, y` scaled coordinates.
        fn oracle_bed_shape_of_sla_defaults() -> Vec<i64>;

        /// `get_extruder_supported_nozzle_volume_types(const DynamicPrintConfig&, int)`
        /// (`:840-858`), in the `std::set`'s own ascending order; `[-1]` on a refusal.
        fn oracle_extruder_supported_nozzle_volume_types(
            keys_and_values: &Vec<String>,
            extruder_id: i32,
        ) -> Vec<i32>;

        /// `DynamicPrintConfig::normalize_fdm_1()` (`PrintConfig.cpp:10428-10478`) against a
        /// config carrying exactly the given `"<key>=<value>"` assignments, with the whole
        /// serialized config handed back. `["<refused>"]` when an assignment was rejected.
        fn oracle_normalize_fdm_1(keys_and_values: &Vec<String>) -> Vec<String>;

        /// `handle_legacy_sla(DynamicPrintConfig&)` (`:10543-10563`), the same shape.
        fn oracle_handle_legacy_sla(keys_and_values: &Vec<String>) -> Vec<String>;

        /// `DynamicPrintConfig::is_using_different_extruders()` (`:10773-10800`) as 0/1, or
        /// -1 when an assignment was refused.
        fn oracle_is_using_different_extruders(keys_and_values: &Vec<String>) -> i32;

        /// `DynamicPrintConfig::has_multi_variant_filament()` (`:10802-10807`), the same shape.
        fn oracle_has_multi_variant_filament(keys_and_values: &Vec<String>) -> i32;

        /// `DynamicPrintConfig::support_different_extruders(int&)` (`:10809-10830`) as
        /// `[result, was the out-parameter written?, its value]`, or `[-1]` on a refusal. The
        /// shim seeds the out-parameter with a value the method never writes, so "left
        /// untouched" is distinguishable from any value it could write.
        fn oracle_support_different_extruders(keys_and_values: &Vec<String>) -> Vec<i32>;

        /// `DynamicPrintConfig::get_extruder_nozzle_volume_count(int, out)`
        /// (`:11577-11603`) as `[count, outer length, (inner length, values...)...]`, or
        /// `[-1]` on a refusal.
        fn oracle_extruder_nozzle_volume_count(
            keys_and_values: &Vec<String>,
            extruder_count: i32,
        ) -> Vec<i32>;

        /// `DynamicPrintConfig::get_index_for_extruder(...)` (`:10832-10889`), or -12345 on a
        /// refusal (a value the method never returns).
        fn oracle_index_for_extruder(
            keys_and_values: &Vec<String>,
            extruder_or_filament_id: i32,
            id_name: &str,
            extruder_type: i32,
            nozzle_volume_type: i32,
            variant_name: &str,
            stride: u32,
        ) -> i32;
    }
}

pub use ffi::{
    oracle_bed_shape_of_sla_defaults, oracle_config_from, oracle_config_from_base,
    oracle_config_index_base, oracle_convert_to_nvt_type, oracle_custom_gcode_placeholder_keys,
    oracle_custom_gcode_placeholders, oracle_dynamic_from_static_serialized,
    oracle_extruder_ams_count_roundtrip, oracle_extruder_index_of_defaults,
    oracle_extruder_nozzle_stats_roundtrip, oracle_extruder_nozzle_volume_count,
    oracle_extruder_option_keys, oracle_extruder_retract_keys,
    oracle_extruder_supported_nozzle_volume_types, oracle_extruder_variant_string,
    oracle_filament_option_keys, oracle_filament_retract_keys, oracle_filament_type_probe,
    oracle_find_variant_index, oracle_full_print_config_serialized, oracle_handle_legacy,
    oracle_handle_legacy_sla, oracle_has_multi_variant_filament, oracle_has_skirt,
    oracle_index_for_extruder, oracle_is_using_different_extruders, oracle_is_xl_printer_dynamic,
    oracle_is_xl_printer_of_print_defaults, oracle_map_variant_indices, oracle_min_object_distance,
    oracle_new_from_defaults_keys_serialized, oracle_normalize_fdm, oracle_normalize_fdm_1,
    oracle_normalize_filament_values_to_variants, oracle_nozzle_diameter_for_filament_of_defaults,
    oracle_nozzle_volume_type_string, oracle_option_info, oracle_option_keys, oracle_option_set,
    oracle_other_def_keys, oracle_other_def_option_info, oracle_parameter_size_of_full,
    oracle_print_and_cli_def_keys, oracle_print_and_cli_handle_legacy, oracle_real_skirt_dist,
    oracle_set_filament_dev_options, oracle_set_num_extruders, oracle_set_num_filaments,
    oracle_set_variant_override, oracle_shaper_type_values_for_flavor,
    oracle_static_class_defaults, oracle_static_class_keys, oracle_support_different_extruders,
    oracle_update_diff_values_to_child_config, oracle_update_non_diff_values_to_base_config,
    oracle_update_static_print_config_from_dynamic, oracle_update_values_from_single_to_multi,
    OracleEnumKeyValue, OracleLegacyResult, OracleOptionInfo,
};
