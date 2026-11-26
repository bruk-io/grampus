#pragma once

// Declarations only (see cxx/clipper2_shim.h's comment for why this crate always splits the
// cxx-generated header from the hand-written declarations file). `cxx_build::bridges` generates
// this header's counterpart at <crate-name>/src/proof_units/aabb_tree_indirect.rs.h on the
// include path it sets up, and `cxx/proof_units/aabb_tree_indirect.cpp` is the one translation
// unit that defines the functions below.
#include "slic3r-ffi/src/proof_units/aabb_tree_indirect.rs.h"

namespace slic3r_ffi {
namespace proof_units {
namespace aabb_tree_indirect {

int64_t path2_intersect_ray_first_hit(const rust::Vec<double> &vertices,
                                       const rust::Vec<int64_t> &faces, double ox, double oy,
                                       double oz, double dx, double dy, double dz);

PathTwoSquaredDistance path2_squared_distance(const rust::Vec<double> &vertices,
                                               const rust::Vec<int64_t> &faces, double px,
                                               double py, double pz);

} // namespace aabb_tree_indirect
} // namespace proof_units
} // namespace slic3r_ffi
