#pragma once

// Declarations only (see cxx/clipper2_shim.h's comment for why this crate always splits the
// cxx-generated header from the hand-written declarations file). `cxx_build::bridges` generates
// this header's counterpart at <crate-name>/src/proof_units/print_config_constants.rs.h on the
// include path it sets up, and `cxx/proof_units/print_config_constants.cpp` is the one
// translation unit that defines the functions below.
#include "slic3r-ffi/src/proof_units/print_config_constants.rs.h"

namespace slic3r_ffi {
namespace proof_units {
namespace print_config_constants {

double path1_initial_layer_height();
int32_t path1_initial_raft_layers();
bool path1_initial_reduce_crossing_wall();

} // namespace print_config_constants
} // namespace proof_units
} // namespace slic3r_ffi
