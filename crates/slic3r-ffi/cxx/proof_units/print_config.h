#pragma once

// Declarations only (see cxx/clipper2_shim.h's comment for why this crate always splits the
// cxx-generated header from the hand-written declarations file). `cxx_build::bridges` generates
// this header's counterpart at <crate-name>/src/proof_units/print_config.rs.h, which is where
// `OracleEnumKeyValue`/`OracleOptionInfo` (declared in the bridge module,
// `src/proof_units/print_config.rs`) come from; this file does not redefine them.
#include "slic3r-ffi/src/proof_units/print_config.rs.h"

namespace slic3r_ffi {
namespace proof_units {
namespace print_config {

// The full key set of `Slic3r::print_config_def.options`, a `std::map<std::string,
// ConfigOptionDef>`, in that map's own (alphabetical) iteration order.
rust::Vec<rust::String> oracle_option_keys();

// One key's whole `ConfigOptionDef` (`Config.hpp:2315-2637`) field for field -- type,
// nullability, default value's serialized form, every GUI-presentation string, the numeric
// `min`/`max`/`max_literal`, the three enum-valued fields as `i32` casts, and the alias,
// shortcut, enum-value, enum-label and enum-key-map lists -- straight off
// `Slic3r::print_config_def.get(key)`. Less `opt_key` (the lookup key) and
// `serialization_key_ordinal` (not a property of the transcription; see the bridge module's doc
// comment). `found = false` when the oracle's registry has no such key at all.
OracleOptionInfo oracle_option_info(rust::Str key);

// `Slic3r::PrintConfigDef::handle_legacy(opt_key, value)` (`PrintConfig.hpp:848`), a public
// static member function with external linkage. It rewrites both arguments in place, so this
// shim copies them in and hands both back.
OracleLegacyResult oracle_handle_legacy(rust::Str opt_key, rust::Str value);

// The four key lists `PrintConfigDef` keeps beside the registry, read through its own public
// const accessors on the global `Slic3r::print_config_def`
// (`PrintConfig.hpp:852,856,859,860`) and handed back in the `std::vector`'s own order, which
// is the braced-initialiser order of `init_extruder_option_keys`/`init_filament_option_keys`
// (`PrintConfig.cpp:8992,9055`) -- not sorted here, so the port's own lists are held to that
// order as well as to that content.
//
// `oracle_filament_option_keys`/`oracle_filament_retract_keys` are expected to come back
// **empty**: `init_filament_option_keys` is never called at the pin (`grep -rn
// init_filament_option_keys vendor/OrcaSlicer/src/libslic3r/` finds only its definition at
// `PrintConfig.cpp:9055` and its declaration at `PrintConfig.hpp:872`), so the two members it
// would fill stay default-constructed on `print_config_def`. They are exposed anyway, so that
// the port being empty there is a measured agreement with the oracle rather than an
// assumption.
rust::Vec<rust::String> oracle_extruder_option_keys();
rust::Vec<rust::String> oracle_extruder_retract_keys();
rust::Vec<rust::String> oracle_filament_option_keys();
rust::Vec<rust::String> oracle_filament_retract_keys();

// The other 13 `ConfigDef` subclasses of `PrintConfig.cpp` (`PrintConfig.hpp:2417-2501`), none
// of which enters `print_config_def`. `which` names one by its class name
// ("CLIActionsConfigDef", "TemperaturesConfigDef", ...); an unknown name yields an empty key
// list and a `found = false` info, which the port's own proof asserts against so a typo in
// `which` fails loudly rather than passing vacuously.
//
// Parameterised over `which` rather than written out as 26 functions, because every one of the
// 13 is read exactly the same way. Four of them have an `extern const` global at the pin and
// are read off it; the other nine are constructed on demand by the GUI and have no global, so
// this shim default-constructs them -- their constructors are externally linked, which is the
// whole of what reading them needs.
rust::Vec<rust::String> oracle_other_def_keys(rust::Str which);
OracleOptionInfo oracle_other_def_option_info(rust::Str which, rust::Str key);

// `custom_gcode_specific_placeholders()` (`PrintConfig.cpp:13643-13646`), the
// `std::map<t_custom_gcode_key, t_config_option_keys>` of placeholders each custom G-code
// accepts. The key list comes back in the map's own (bytewise-ascending) order; each G-code's
// own placeholder vector comes back in that vector's own braced-initialiser order, not sorted,
// so the port's list is held to that order as well as to its content.
rust::Vec<rust::String> oracle_custom_gcode_placeholder_keys();
rust::Vec<rust::String> oracle_custom_gcode_placeholders(rust::Str key);

// The 11 `StaticPrintConfig`-derived option classes (`PrintConfig.hpp:1282-2395`), by class
// name. Each is read off `CLASS_NAME::defaults()` -- the object
// `print_config_static_initializer()` built before `main` from `print_config_def`
// (`PrintConfig.cpp:12689-12695`).
//
// `oracle_static_class_keys` is `defaults().keys_ref()`, the list `StaticCache::finalize` filled
// by walking the registry in order and keeping each key the class has a field for
// (`PrintConfig.hpp:1105-1129`) -- so it is both the class's field set and the order the class
// reports it in.
//
// `oracle_static_class_defaults` is each of those keys' `optptr(key)->serialize()`, in the same
// order. That is what exercises the wiring rather than only the field list: a field declared
// with the wrong option type, or one whose name does not match the registry row it is meant to
// carry, serializes differently from the registry default `finalize` assigned it.
//
// An unknown class name yields empty lists in both, which the port's own proof asserts against.
rust::Vec<rust::String> oracle_static_class_keys(rust::Str which);
rust::Vec<rust::String> oracle_static_class_defaults(rust::Str which);

// One of the seven option-set globals (`PrintConfig.cpp:10105-10305`, declared `extern` at
// `PrintConfig.hpp:1009-1024`), by variable name, in the `std::set`'s own order. Empty for an
// unknown name -- and for `empty_options`, which really is empty at the pin, so the port's own
// proof names that one explicitly rather than reading an empty list as agreement.
rust::Vec<rust::String> oracle_option_set(rust::Str which);

// The externally-linked free functions of `PrintConfig.cpp:140-989`, each called directly. They
// are what make a literal proof path 3 unnecessary for this part of the unit: unlike
// `enum_names_from_keys_map` and the two anonymous-namespace string helpers, these have external
// linkage and a symbol to call.
rust::Vec<rust::String> oracle_shaper_type_values_for_flavor(int32_t flavor);
rust::String oracle_extruder_variant_string(int32_t extruder_type, int32_t nozzle_volume_type);
rust::String oracle_nozzle_volume_type_string(int32_t nozzle_volume_type);
int32_t oracle_convert_to_nvt_type(rust::Str variant_str);
int32_t oracle_find_variant_index(rust::Str variant, int32_t variant_id_1based,
                                  const rust::Vec<rust::String> &variant_list,
                                  const rust::Vec<int32_t> &variant_ids_1based);
rust::Vec<int32_t> oracle_map_variant_indices(const rust::Vec<rust::String> &variants,
                                              const rust::Vec<int32_t> &ids,
                                              const rust::Vec<rust::String> &from_variants,
                                              const rust::Vec<int32_t> &from_ids);
int32_t oracle_config_index_base(int32_t volume_type, int32_t extruder_type,
                                 int32_t variant_id_1based,
                                 const rust::Vec<rust::String> &variant_list,
                                 const rust::Vec<int32_t> &variant_ids_1based);
// `get_extruder_ams_count` / `get_extruder_nozzle_stats` and their two inverses, round-tripped
// through the string form so one shim call covers a parse and a save: the parse's result is
// handed straight back to the save, and the save's output is what crosses. A parse difference
// shows up as a different saved string.
// `get_extruder_ams_count` (`PrintConfig.cpp:943-961`) and
// `save_extruder_ams_count_to_string` (`:963-975`), and the same pair for nozzle stats
// (`:901-922`, `:924-941`), each crossing the bridge on its own.
//
// **These replace two roundtrip functions that composed each parse with its own inverse and
// sent only the final string.** A parse defect that the save happens to invert was invisible on
// both sides of that comparison: the port and the oracle could disagree about the intermediate
// and still agree about the string. All four halves are externally linked, so there is no
// reason to compose them.
//
// The intermediate `std::vector<std::map<K, int>>` crosses as a flat list: entry 0 is the
// number of maps as decimal, then one entry per map holding its `k=v` pairs joined by `;` in
// `std::map`'s own ascending-key order (an empty string for an empty map). An enum key crosses
// as its underlying int.
rust::Vec<rust::String> oracle_get_extruder_ams_count(const rust::Vec<rust::String> &strs);
rust::Vec<rust::String> oracle_save_extruder_ams_count_to_string(
    const rust::Vec<rust::String> &flat);
rust::Vec<rust::String> oracle_get_extruder_nozzle_stats(const rust::Vec<rust::String> &strs);
rust::Vec<rust::String> oracle_save_extruder_nozzle_stats_to_string(
    const rust::Vec<rust::String> &flat);

// `get_extruder_index(const GCodeConfig&, unsigned int)` (`PrintConfig.cpp:140-146`) and
// `nozzle_diameter_for_filament(const PrintConfig&, int, bool)` (`:148-156`), each against the
// default-constructed config the class caches hold -- the only instance of either class a shim
// can build without a preset, and the one the port's own side reads too.
uint64_t oracle_extruder_index_of_defaults(uint32_t filament_id);
double oracle_nozzle_diameter_for_filament_of_defaults(int32_t filament_id, bool is_bbl_printer);

// `DynamicPrintConfig::full_print_config()` (`PrintConfig.cpp:10307-10310`), as the whole
// config serialized: every key in the `std::map`'s own order, one `"<key>=<serialized>"` entry
// each. The construction route for the comparisons below, and a check of its own -- the
// vendored body casts a `FullPrintConfig` to `PrintRegionConfig` before handing it to the
// `const StaticPrintConfig&` constructor, so which keys land is not what the name suggests.
rust::Vec<rust::String> oracle_full_print_config_serialized();

// `DynamicPrintConfig::new_from_defaults_keys(keys)` (`:10316-10321`), serialized the same way.
rust::Vec<rust::String> oracle_new_from_defaults_keys_serialized(const rust::Vec<rust::String> &keys);

// `DynamicPrintConfig(const StaticPrintConfig&)` (`:10312-10314`) over one of the 11 static
// classes by name, serialized the same way.
rust::Vec<rust::String> oracle_dynamic_from_static_serialized(rust::Str which);

// `DynamicPrintConfig::get_parameter_size(param_name, extruder_nums)` (`:10565-10595`), against
// a config built from `full_print_config()`.
uint64_t oracle_parameter_size_of_full(rust::Str param_name, uint64_t extruder_nums);

// `min_object_distance(const ConfigBase&)` (`:10323-10348`), against `full_print_config()` with
// `print_sequence` and `extruder_clearance_radius` overridden to the given values; an empty
// override string leaves the key absent, which is what the function's two null checks turn on.
double oracle_min_object_distance(rust::Str print_sequence, rust::Str clearance_radius,
                                  rust::Str printer_technology);

// `DynamicPrintConfig::get_filament_vendor()` / `get_filament_type()` (`:11552-11575`) and
// `get_filament_type(displayed, id)` (`:10698-10771`), against a config carrying exactly the
// given option values. The three-element result of the last one is
// {returned type, displayed type}.
rust::Vec<rust::String> oracle_filament_type_probe(rust::Str filament_type, rust::Str filament_id,
                                                   rust::Str filament_is_support,
                                                   rust::Str filament_vendor, uint64_t id);

// The seven `DynamicPrintConfig` methods this unit ports, each against a config built from the
// given `"<key>=<value>"` assignments (deserialized in the order given) and nothing else.
//
// `keys_and_values` is the config's whole content. A refused assignment is reported rather than
// allowed to throw out of the bridge -- see `oracle_filament_type_probe`'s own note.
//
// `oracle_normalize_fdm_1` and `oracle_handle_legacy_sla` mutate the config and hand back its
// whole serialized form, the same shape `serialize_config` produces everywhere here, so the
// comparison is of every key and value the call leaves behind.
// The config `config_from` builds, serialized, with no method called on it. This is what lets a
// method comparison mean something: it proves both sides start from the same config before the
// call, rather than assuming two different construction routes agree.
rust::Vec<rust::String> oracle_config_from(const rust::Vec<rust::String> &keys_and_values);

// The config-mutating calls of this batch, each against a config built by `config_from` and
// handed back whole as `serialize_config` produces it. `["<refused>"]` on a rejected assignment.
//
// `base` selects the config the assignments are applied to: "empty" for a bare
// `DynamicPrintConfig`, "full" for `DynamicPrintConfig::full_print_config()`. The "full" base
// exists because several vendored bodies dereference an option pointer **without** checking it:
// `extend_extruder_variant` does `config.option("printer_extruder_id")->values.clear()`
// (`PrintConfig.cpp:10609-10611`, the `assert` above it compiled out of the oracle's Release
// build) and `normalize_fdm` reads `ps_opt->value` (`:10408`) from an uncreated
// `option("print_sequence")`. Called on a config missing those keys, the oracle **segfaults** --
// observed, not inferred. A "full" base carries every registry key, which is the state every
// real caller is in.
rust::Vec<rust::String> oracle_config_from_base(rust::Str base,
                                                const rust::Vec<rust::String> &keys_and_values);
rust::Vec<rust::String> oracle_normalize_fdm(rust::Str base,
                                             const rust::Vec<rust::String> &keys_and_values,
                                             int32_t used_filaments);
rust::Vec<rust::String> oracle_set_num_extruders(rust::Str base,
                                                 const rust::Vec<rust::String> &keys_and_values,
                                                 uint32_t num_extruders);
rust::Vec<rust::String> oracle_set_num_filaments(rust::Str base,
                                                 const rust::Vec<rust::String> &keys_and_values,
                                                 uint32_t num_filaments);
rust::Vec<rust::String>
oracle_normalize_filament_values_to_variants(const rust::Vec<rust::String> &keys_and_values);
// `set_filament_dev_options` takes a vector of filament configs; two are enough to exercise the
// concatenation and the per-filament default fallback, and an empty list for either leaves that
// filament out of the vector entirely.
rust::Vec<rust::String>
oracle_set_filament_dev_options(const rust::Vec<rust::String> &keys_and_values,
                                const rust::Vec<rust::String> &filament_a,
                                const rust::Vec<rust::String> &filament_b);

// The skirt and XL-printer predicates (`PrintConfig.cpp:14376-14433`).
// These take a `base` for the same reason the three above do: `get_real_skirt_dist` reads
// `cfg.opt_float("skirt_distance")` and `cfg.opt_int("skirt_loops")`
// (`PrintConfig.cpp:14387`, `:14389`), and `DynamicConfig::opt_float`/`opt_int` are an
// **unguarded** `dynamic_cast<...>(this->option(key))->value` (`Config.hpp:3048`, `:3057`), so a
// config missing the key segfaults the oracle -- observed.
int32_t oracle_has_skirt(rust::Str base, const rust::Vec<rust::String> &keys_and_values);
double oracle_real_skirt_dist(rust::Str base, const rust::Vec<rust::String> &keys_and_values);
int32_t oracle_is_xl_printer_dynamic(rust::Str base,
                                     const rust::Vec<rust::String> &keys_and_values);
// `is_XL_printer(const PrintConfig&)` and `get_bed_shape(const SLAPrinterConfig&)` can only be
// reached through a class instance, and `defaults()` is the only one a shim can build without a
// preset -- the same instance the port's own side reads.
int32_t oracle_is_xl_printer_of_print_defaults();

// `DynamicPrintAndCLIConfig` (`PrintConfig.hpp:2512-2543`): its `def()`'s key set, which is the
// union of the registry and the three command line tables, and its `handle_legacy`, which
// delegates to the registry's only for a key none of those three tables names.
rust::Vec<rust::String> oracle_print_and_cli_def_keys();

// `update_static_print_config_from_dynamic` (`PrintConfig.cpp:12393-12413`, declared
// `PrintConfig.hpp:1035`) and `set_variant_override` (`:11394-11402`, declared `:1016`), each
// externally linked and called directly.
//
// `ensure_process_variant_columns` (`:11604-11632`) and `normalize_stride2_floats`
// (`:12089-12127`) are **not** here: both are file-scope `static` in the vendored translation
// unit, so neither has a symbol to call. They are the kind of internal-linkage helper this
// unit's `proof path: 3` cell exists for, and the only route to them is through a caller --
// `update_values_to_printer_extruders` and `update_non_diff_values_to_base_config`
// respectively, neither of which this unit ports yet. Both are ported and recorded as awaiting
// their enclosing caller's comparison.
rust::Vec<rust::String>
oracle_update_static_print_config_from_dynamic(rust::Str base,
                                               const rust::Vec<rust::String> &target_kv,
                                               const rust::Vec<rust::String> &dest_kv,
                                               const rust::Vec<int32_t> &variant_index,
                                               const rust::Vec<rust::String> &key_set1,
                                               int32_t stride);
// `update_diff_values_to_child_config` (`PrintConfig.cpp:12271-12356`) and
// `update_non_diff_values_to_base_config` (`:12141-12269`), each against two configs built from
// `"<key>=<value>"` assignments, with the receiver handed back whole.
rust::Vec<rust::String>
oracle_update_diff_values_to_child_config(rust::Str base, const rust::Vec<rust::String> &self_kv,
                                          const rust::Vec<rust::String> &new_kv,
                                          rust::Str extruder_id_name,
                                          rust::Str extruder_variant_name,
                                          const rust::Vec<rust::String> &key_set1,
                                          const rust::Vec<rust::String> &key_set2);
rust::Vec<rust::String>
oracle_update_non_diff_values_to_base_config(rust::Str base,
                                             const rust::Vec<rust::String> &self_kv,
                                             const rust::Vec<rust::String> &new_kv,
                                             const rust::Vec<rust::String> &keys,
                                             const rust::Vec<rust::String> &different_keys,
                                             rust::Str extruder_id_name,
                                             rust::Str extruder_variant_name,
                                             const rust::Vec<rust::String> &key_set1,
                                             const rust::Vec<rust::String> &key_set2);

// `compute_filament_override_value` (`PrintConfig.cpp:12358-12390`) over one
// `extruder_retract_keys()` key, driven exactly as `Print.cpp:4788-4796` drives it: the three
// options are read off the two configs rather than synthesized, so the `dynamic_cast`s at
// `:12366`/`:12374` see the instantiations the real caller gives them. Returns a status, then
// the `diff_keys` the call appended, then the `filament_overrides` config it wrote.
rust::Vec<rust::String> oracle_compute_filament_override_value(
    const rust::Vec<rust::String> &full_kv, const rust::Vec<rust::String> &old_kv,
    const rust::Vec<int32_t> &f_map_indices, rust::Str opt_key);

// `update_values_from_single_to_multi` (`PrintConfig.cpp:10891-10993`): the receiver handed
// back whole, with the method's own `int` return prefixed as the first entry.
rust::Vec<rust::String>
oracle_update_values_from_single_to_multi(rust::Str base, const rust::Vec<rust::String> &self_kv,
                                          const rust::Vec<rust::String> &multi_kv,
                                          const rust::Vec<rust::String> &key_set,
                                          rust::Str id_name, rust::Str variant_name);

// `set_variant_override` (`:11394-11402`) over one `coFloats` pair: the target's serialized
// value after the override.
rust::String oracle_set_variant_override(rust::Str target_value, rust::Str source_value,
                                         const rust::Vec<int32_t> &variant_index,
                                         int32_t stride);
OracleLegacyResult oracle_print_and_cli_handle_legacy(rust::Str opt_key, rust::Str value);
rust::Vec<int64_t> oracle_bed_shape_of_sla_defaults();

// `get_extruder_supported_nozzle_volume_types(const DynamicPrintConfig&, int)`
// (`PrintConfig.cpp:840-858`), as the set's own ascending order.
rust::Vec<int32_t>
oracle_extruder_supported_nozzle_volume_types(const rust::Vec<rust::String> &keys_and_values,
                                              int32_t extruder_id);

rust::Vec<rust::String> oracle_normalize_fdm_1(const rust::Vec<rust::String> &keys_and_values);
rust::Vec<rust::String> oracle_handle_legacy_sla(const rust::Vec<rust::String> &keys_and_values);

// The four query methods, as one call each. `support_different_extruders` and
// `get_extruder_nozzle_volume_count` write an out-parameter, which the returned list carries
// after the result: `[result, written?, values...]`.
int32_t oracle_is_using_different_extruders(const rust::Vec<rust::String> &keys_and_values);
int32_t oracle_has_multi_variant_filament(const rust::Vec<rust::String> &keys_and_values);
rust::Vec<int32_t> oracle_support_different_extruders(const rust::Vec<rust::String> &keys_and_values);
rust::Vec<int32_t>
oracle_extruder_nozzle_volume_count(const rust::Vec<rust::String> &keys_and_values,
                                    int32_t extruder_count);
int32_t oracle_index_for_extruder(const rust::Vec<rust::String> &keys_and_values,
                                  int32_t extruder_or_filament_id, rust::Str id_name,
                                  int32_t extruder_type, int32_t nozzle_volume_type,
                                  rust::Str variant_name, uint32_t stride);

} // namespace print_config
} // namespace proof_units
} // namespace slic3r_ffi
