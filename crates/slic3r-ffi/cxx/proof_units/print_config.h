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
rust::Vec<rust::String> oracle_extruder_ams_count_roundtrip(const rust::Vec<rust::String> &strs);
rust::Vec<rust::String> oracle_extruder_nozzle_stats_roundtrip(const rust::Vec<rust::String> &strs);

// `get_extruder_index(const GCodeConfig&, unsigned int)` (`PrintConfig.cpp:140-146`) and
// `nozzle_diameter_for_filament(const PrintConfig&, int, bool)` (`:148-156`), each against the
// default-constructed config the class caches hold -- the only instance of either class a shim
// can build without a preset, and the one the port's own side reads too.
uint64_t oracle_extruder_index_of_defaults(uint32_t filament_id);
double oracle_nozzle_diameter_for_filament_of_defaults(int32_t filament_id, bool is_bbl_printer);

} // namespace print_config
} // namespace proof_units
} // namespace slic3r_ffi
