#pragma once

// Declarations only (see cxx/clipper2_shim.h's comment for why this crate always splits the
// cxx-generated header from the hand-written declarations file). `cxx_build::bridges` generates
// this header's counterpart at <crate-name>/src/proof_units/aabb_tree_lines.rs.h on the include
// path it sets up, and `cxx/proof_units/aabb_tree_lines.cpp` is the one translation unit that
// defines the functions below.
#include "slic3r-ffi/src/proof_units/aabb_tree_lines.rs.h"

namespace slic3r_ffi {
namespace proof_units {
namespace aabb_tree_lines {

rust::Vec<double> path2_distance_and_outside(rust::Slice<const double> ax,
                                              rust::Slice<const double> ay,
                                              rust::Slice<const double> bx,
                                              rust::Slice<const double> by, double qx, double qy);

} // namespace aabb_tree_lines
} // namespace proof_units
} // namespace slic3r_ffi
