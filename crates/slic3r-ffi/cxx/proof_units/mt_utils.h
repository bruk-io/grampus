#pragma once

// Declarations only (see cxx/clipper2_shim.h's comment for why this crate always splits the
// cxx-generated header from the hand-written declarations file). `cxx_build::bridges` generates
// this header's counterpart at <crate-name>/src/proof_units/mt_utils.rs.h on the include path it
// sets up, and `cxx/proof_units/mt_utils.cpp` is the one translation unit that defines the
// function below.
#include "slic3r-ffi/src/proof_units/mt_utils.rs.h"

namespace slic3r_ffi {
namespace proof_units {
namespace mt_utils {

rust::Vec<double> path2_grid(double start, double stop, double stride);

} // namespace mt_utils
} // namespace proof_units
} // namespace slic3r_ffi
