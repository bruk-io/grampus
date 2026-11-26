#pragma once

// Declarations only (see cxx/clipper2_shim.h's comment for why this crate always splits the
// cxx-generated header from the hand-written declarations file). `cxx_build::bridges` generates
// this header's counterpart at <crate-name>/src/proof_units/point.rs.h on the include path it
// sets up, and `cxx/proof_units/point.cpp` is the one translation unit that defines the
// functions below.
#include "slic3r-ffi/src/proof_units/point.rs.h"

namespace slic3r_ffi {
namespace proof_units {
namespace point {

int32_t path1_int128_orient(int64_t p1x, int64_t p1y, int64_t p2x, int64_t p2y, int64_t p3x,
                             int64_t p3y);
rust::Vec<int64_t> path2_point_rotate(int64_t px, int64_t py, int64_t cx, int64_t cy,
                                       double angle);

} // namespace point
} // namespace proof_units
} // namespace slic3r_ffi
