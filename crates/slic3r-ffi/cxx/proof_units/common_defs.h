#pragma once

// Declarations only (see cxx/clipper2_shim.h's comment for why this crate always splits the
// cxx-generated header from the hand-written declarations file). `cxx_build::bridges` generates
// this header's counterpart -- `rust::Vec` and the bridge's own signature -- at
// <crate-name>/src/proof_units/common_defs.rs.h on the include path it sets up, and
// `cxx/proof_units/common_defs.cpp` is the one translation unit that defines the function below.
#include "slic3r-ffi/src/proof_units/common_defs.rs.h"

namespace slic3r_ffi {
namespace proof_units {
namespace common_defs {

rust::Vec<int64_t> path2_nozzle_type_discriminants();

} // namespace common_defs
} // namespace proof_units
} // namespace slic3r_ffi
