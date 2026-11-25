#pragma once

// Declarations only: never includes a libslic3r or Clipper2 header itself (see
// cxx/clipper2_shim.h's comment for why this crate always splits the cxx-generated header from
// the hand-written declarations file). cxx_build::bridges([..., "src/proof.rs"]) generates this
// header's counterpart -- the shared ProofZPoint struct and rust::Vec -- at
// <crate-name>/src/proof.rs.h on the include path it sets up. cxx/proof_shims.cpp (proof path 1)
// and cxx/include_vendored.cpp (proof path 3) are the two translation units that define the
// functions below (task 31-proof-harness, plan step 18, R19).
#include "slic3r-ffi/src/proof.rs.h"

namespace slic3r_ffi {
namespace proof {

uint64_t path1_wipe_tower_object_id();
rust::Vec<ProofZPoint> path3_to_paths64(const rust::Vec<ProofZPoint> &path);

} // namespace proof
} // namespace slic3r_ffi
