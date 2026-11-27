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

// One key's behaviourally load-bearing metadata (type, nullability, CLI spelling, default
// value's serialized form, enum value list and enum name/value map) straight off
// `Slic3r::print_config_def.get(key)`. `found = false` when the oracle's registry has no such
// key at all.
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

} // namespace print_config
} // namespace proof_units
} // namespace slic3r_ffi
