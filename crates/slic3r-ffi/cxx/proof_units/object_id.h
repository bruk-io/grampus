#pragma once

// Declarations only (see cxx/clipper2_shim.h's comment for why this crate always splits the
// cxx-generated header from the hand-written declarations file). `cxx_build::bridges` generates
// this header's counterpart -- rust::Vec and the bridge's own signatures -- at
// <crate-name>/src/proof_units/object_id.rs.h on the include path it sets up, and
// `cxx/proof_units/object_id.cpp` is the one translation unit that defines the functions below.
#include "slic3r-ffi/src/proof_units/object_id.rs.h"

namespace slic3r_ffi {
namespace proof_units {
namespace object_id {

uint64_t path1_wipe_tower_object_id();
uint64_t path1_wipe_tower_instance_id();
rust::Vec<int64_t> path2_header_observations();

} // namespace object_id
} // namespace proof_units
} // namespace slic3r_ffi
