#pragma once

// Declarations only (see cxx/clipper2_shim.h's comment for why this crate always splits the
// cxx-generated header from the hand-written declarations file). `cxx_build::bridges` generates
// this header's counterpart at <crate-name>/src/proof_units/line.rs.h on the include path it
// sets up, and `cxx/proof_units/line.cpp` is the one translation unit that defines the
// functions below.
#include "slic3r-ffi/src/proof_units/line.rs.h"

namespace slic3r_ffi {
namespace proof_units {
namespace line {

rust::Vec<int64_t> path1_intersection_infinite(int64_t ax, int64_t ay, int64_t bx, int64_t by,
                                                int64_t cx, int64_t cy, int64_t dx, int64_t dy);
double path2_line_length(int64_t ax, int64_t ay, int64_t bx, int64_t by);
rust::Vec<int64_t> path2_line_midpoint_normal(int64_t ax, int64_t ay, int64_t bx, int64_t by);
rust::Vec<double> path2_line_distance_to_infinite_squared(int64_t ax, int64_t ay, int64_t bx,
                                                           int64_t by, int64_t px, int64_t py);

} // namespace line
} // namespace proof_units
} // namespace slic3r_ffi
