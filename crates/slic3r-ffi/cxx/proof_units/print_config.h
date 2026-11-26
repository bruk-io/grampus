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

} // namespace print_config
} // namespace proof_units
} // namespace slic3r_ffi
