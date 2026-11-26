#pragma once

// Declarations only (see cxx/clipper2_shim.h's comment for why this crate always splits the
// cxx-generated header from the hand-written declarations file). `cxx_build::bridges` generates
// this header's counterpart at <crate-name>/src/proof_units/linear_alg2_d.rs.h on the include
// path it sets up, and `cxx/proof_units/linear_alg2_d.cpp` is the one translation unit that
// defines the functions below.
#include "slic3r-ffi/src/proof_units/linear_alg2_d.rs.h"

namespace slic3r_ffi {
namespace proof_units {
namespace linear_alg2_d {

int64_t path1_point_is_left_of_line(int64_t px, int64_t py, int64_t ax, int64_t ay, int64_t bx,
                                     int64_t by);
float path1_get_angle_left(int64_t ax, int64_t ay, int64_t bx, int64_t by, int64_t cx,
                            int64_t cy);

} // namespace linear_alg2_d
} // namespace proof_units
} // namespace slic3r_ffi
