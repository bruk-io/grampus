#pragma once

// Declarations only (see cxx/clipper2_shim.h's comment for why this crate always splits the
// cxx-generated header from the hand-written declarations file). `cxx_build::bridges` generates
// this header's counterpart at <crate-name>/src/proof_units/brim_ears_point.rs.h on the include
// path it sets up, and `cxx/proof_units/brim_ears_point.cpp` is the one translation unit that
// defines the function below.
#include "slic3r-ffi/src/proof_units/brim_ears_point.rs.h"

namespace slic3r_ffi {
namespace proof_units {
namespace brim_ears_point {

// Path 2: `BrimEarsPoint.hpp`'s own header-only `BrimPoint::operator==`, driven directly through
// a real vendored `Slic3r::BrimPoint` instantiated in this translation unit.
bool path2_brim_point_eq(float ax, float ay, float az, float ar, float bx, float by, float bz,
                          float br);

} // namespace brim_ears_point
} // namespace proof_units
} // namespace slic3r_ffi
