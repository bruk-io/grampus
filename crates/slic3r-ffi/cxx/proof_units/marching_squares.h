#pragma once

// Declarations only (see cxx/clipper2_shim.h's comment for why this crate always splits the
// cxx-generated header from the hand-written declarations file). `cxx_build::bridges` generates
// this header's counterpart at <crate-name>/src/proof_units/marching_squares.rs.h on the include
// path it sets up, and `cxx/proof_units/marching_squares.cpp` is the one translation unit that
// defines the function below.
#include "slic3r-ffi/src/proof_units/marching_squares.rs.h"

namespace slic3r_ffi {
namespace proof_units {
namespace marching_squares {

rust::Vec<int64_t> path1_execute(const rust::Vec<double> &raster, size_t rows, size_t cols,
                                  double isoval, int64_t window_r, int64_t window_c);

} // namespace marching_squares
} // namespace proof_units
} // namespace slic3r_ffi
