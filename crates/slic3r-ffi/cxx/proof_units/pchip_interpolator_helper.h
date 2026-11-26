#pragma once

// Declarations only (see cxx/clipper2_shim.h's comment for why this crate always splits the
// cxx-generated header from the hand-written declarations file). `cxx_build::bridges` generates
// this header's counterpart at <crate-name>/src/proof_units/pchip_interpolator_helper.rs.h on
// the include path it sets up, and `cxx/proof_units/pchip_interpolator_helper.cpp` is the one
// translation unit that defines the functions below.
#include "slic3r-ffi/src/proof_units/pchip_interpolator_helper.rs.h"

namespace slic3r_ffi {
namespace proof_units {
namespace pchip_interpolator_helper {

// Builds a `PchipInterpolatorHelper` from `x`/`y` (assumed well-formed: same length, >= 2
// points, strictly increasing -- the proof's own inputs, never user input) and returns
// `interpolate(q)` for every `q` in `queries`, in order.
rust::Vec<double> path1_interpolate_many(rust::Vec<double> x, rust::Vec<double> y, rust::Vec<double> queries);

// True if constructing a `PchipInterpolatorHelper` from `x`/`y` throws `std::invalid_argument`
// (mismatched lengths, or fewer than two points) -- the unit's one error case
// (`PchipInterpolatorHelper.cpp:30-33`).
bool path1_constructing_throws(rust::Vec<double> x, rust::Vec<double> y);

} // namespace pchip_interpolator_helper
} // namespace proof_units
} // namespace slic3r_ffi
