#pragma once

// Declarations only (see cxx/clipper2_shim.h's comment for why this crate always splits the
// cxx-generated header from the hand-written declarations file). `cxx_build::bridges` generates
// this header's counterpart at <crate-name>/src/proof_units/kd_tree_indirect.rs.h on the include
// path it sets up, and `cxx/proof_units/kd_tree_indirect.cpp` is the one translation unit that
// defines the functions below.
#include "slic3r-ffi/src/proof_units/kd_tree_indirect.rs.h"

namespace slic3r_ffi {
namespace proof_units {
namespace kd_tree_indirect {

uint64_t path2_find_closest_point(const rust::Vec<double> &coords, double qx, double qy);
rust::Vec<uint64_t> path2_find_nearby_points(const rust::Vec<double> &coords, double cx, double cy,
                                              double max_distance);

} // namespace kd_tree_indirect
} // namespace proof_units
} // namespace slic3r_ffi
