#pragma once

// Declarations only (see cxx/clipper2_shim.h's comment for why this crate always splits the
// cxx-generated header from the hand-written declarations file). `cxx_build::bridges` generates
// this header's counterpart at <crate-name>/src/proof_units/polygons_point_index.rs.h on the
// include path it sets up, and `cxx/proof_units/polygons_point_index.cpp` is the one
// translation unit that defines the functions below.
#include "slic3r-ffi/src/proof_units/polygons_point_index.rs.h"

namespace slic3r_ffi {
namespace proof_units {
namespace polygons_point_index {

rust::Vec<int64_t> path1_p_next_prev(rust::Slice<const int64_t> xs, rust::Slice<const int64_t> ys,
                                      uint32_t point_idx);

} // namespace polygons_point_index
} // namespace proof_units
} // namespace slic3r_ffi
