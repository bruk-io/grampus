#pragma once

// Declarations only (see cxx/clipper2_shim.h's comment for why this crate always splits the
// cxx-generated header from the hand-written declarations file). `cxx_build::bridges` generates
// this header's counterpart at <crate-name>/src/proof_units/format.rs.h on the include path it
// sets up, and `cxx/proof_units/format.cpp` is the one translation unit that defines the
// function below.
#include "slic3r-ffi/src/proof_units/format.rs.h"

namespace slic3r_ffi {
namespace proof_units {
namespace format {

rust::Vec<int64_t> path2_format_positional_bytes(double v);

} // namespace format
} // namespace proof_units
} // namespace slic3r_ffi
