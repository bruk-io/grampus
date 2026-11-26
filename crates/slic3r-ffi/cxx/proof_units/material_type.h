#pragma once

// Declarations only (see cxx/clipper2_shim.h's comment for why this crate always splits the
// cxx-generated header from the hand-written declarations file). `cxx_build::bridges` generates
// this header's counterpart at <crate-name>/src/proof_units/material_type.rs.h on the include
// path it sets up, and `cxx/proof_units/material_type.cpp` is the one translation unit that
// defines the functions below.
#include "slic3r-ffi/src/proof_units/material_type.rs.h"

namespace slic3r_ffi {
namespace proof_units {
namespace material_type {

// Each returns the out-param(s) `Slic3r::MaterialType::get_X` writes, plus whether it found the
// named material, as the last element (`1`/`0` or `1.0`/`0.0`) -- see material_type.cpp's own
// header comment for why this calls the externally linked class statics directly rather than
// including the vendored `.cpp` whole, despite this row's seeded `proof path` cell being `3`.
rust::Vec<int64_t> path_temperature_range(rust::Str name);
rust::Vec<int64_t> path_chamber_temperature_range(rust::Str name);
rust::Vec<double> path_adhesion_coefficient(rust::Str name);
rust::Vec<double> path_yield_strength(rust::Str name);
rust::Vec<double> path_thermal_length(rust::Str name);

} // namespace material_type
} // namespace proof_units
} // namespace slic3r_ffi
