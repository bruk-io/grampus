// Proof shim for the `BrimEarsPoint.hpp` unit (R19, proof path 2, task
// `58-foundation-bounding-box`), compiled onto `crates/slic3r-ffi`'s shared `cxx_build::Build`
// by this crate's `build.rs` -- the one sanctioned route a ported unit reaches the oracle by
// (`docs/PORTING.md`, "Decisions recorded on 2026-11-25").
//
// `BrimPoint::operator==` is entirely inline (`BrimEarsPoint.hpp:53-57`) and never explicitly
// instantiated anywhere in the vendored tree, so it is driven directly, through a real
// `Slic3r::BrimPoint` instantiated in this translation unit -- the same technique
// `cxx/proof_units/point.cpp`'s `scaled<coord_t>()`/`unscaled<double>()` shims use for
// header-only template code.
#include "brim_ears_point.h"

#include "BrimEarsPoint.hpp"

namespace slic3r_ffi {
namespace proof_units {
namespace brim_ears_point {

bool path2_brim_point_eq(float ax, float ay, float az, float ar, float bx, float by, float bz,
                          float br) {
    Slic3r::BrimPoint a(ax, ay, az, ar);
    Slic3r::BrimPoint b(bx, by, bz, br);
    return a == b;
}

} // namespace brim_ears_point
} // namespace proof_units
} // namespace slic3r_ffi
