#pragma once

// Declarations only (see cxx/clipper2_shim.h's comment for why this crate always splits the
// cxx-generated header from the hand-written declarations file). `cxx_build::bridges` generates
// this header's counterpart at <crate-name>/src/proof_units/technologies.rs.h on the include
// path it sets up, and `cxx/proof_units/technologies.cpp` is the one translation unit that
// defines the function below.
#include "slic3r-ffi/src/proof_units/technologies.rs.h"

namespace slic3r_ffi {
namespace proof_units {
namespace technologies {

rust::Vec<int32_t> path1_flags();

} // namespace technologies
} // namespace proof_units
} // namespace slic3r_ffi
