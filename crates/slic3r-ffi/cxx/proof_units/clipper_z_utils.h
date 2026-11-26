#pragma once

// Declarations only: never includes a libslic3r or Clipper2 header itself (see
// cxx/clipper2_shim.h's comment for why this crate always splits the cxx-generated header from
// the hand-written declarations file). `cxx_build::bridges` generates this header's counterpart
// -- the shared ProofZPoint struct and rust::Vec -- at
// <crate-name>/src/proof_units/clipper_z_utils.rs.h on the include path it sets up, and
// `cxx/proof_units/clipper_z_utils.cpp` is the one translation unit that defines the function
// below.
#include "slic3r-ffi/src/proof_units/clipper_z_utils.rs.h"

namespace slic3r_ffi {
namespace proof_units {
namespace clipper_z_utils {

rust::Vec<ProofZPoint> path3_to_paths64(const rust::Vec<ProofZPoint> &path);

} // namespace clipper_z_utils
} // namespace proof_units
} // namespace slic3r_ffi
