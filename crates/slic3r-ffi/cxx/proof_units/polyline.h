#pragma once

// Declarations only (see cxx/clipper2_shim.h's comment for why this crate always splits the
// cxx-generated header from the hand-written declarations file). `cxx_build::bridges` generates
// this header's counterpart at <crate-name>/src/proof_units/polyline.rs.h on the include path it
// sets up, and `cxx/proof_units/polyline.cpp` is the one translation unit that defines the
// functions below.
#include "slic3r-ffi/src/proof_units/polyline.rs.h"

namespace slic3r_ffi {
namespace proof_units {
namespace polyline {

rust::Vec<int64_t> path1_clip_end(rust::Slice<const int64_t> xs, rust::Slice<const int64_t> ys,
                                   double distance);
double path2_total_length(rust::Slice<const int64_t> xs, rust::Slice<const int64_t> ys);
bool path2_is_closed(rust::Slice<const int64_t> xs, rust::Slice<const int64_t> ys);
rust::Vec<int64_t> path2_new_scale(rust::Slice<const double> xs, rust::Slice<const double> ys);

} // namespace polyline
} // namespace proof_units
} // namespace slic3r_ffi
