// Proof shims for the `linearAlg2D.hpp` unit (R19, proof path 1, task
// `72-foundation-arachne-grids`), compiled onto `crates/slic3r-ffi`'s shared `cxx_build::Build`
// -- the one sanctioned route a ported unit reaches the oracle by (`docs/PORTING.md`,
// "Decisions recorded on 2026-11-25").
//
// `pointIsLeftOfLine` and `getAngleLeft` (`Arachne/utils/linearAlg2D.hpp:26`, `:48`) are
// `static inline` free functions at namespace scope -- internal linkage per translation unit,
// but reached by any in-scope caller through the header alone. This shim includes the header
// directly and calls both, the same technique `cxx/proof_units/point.cpp`'s three
// `scaled`/`unscaled` shims use for `Point.hpp`'s own header-only templates.
#include "linear_alg2_d.h"

#include "Point.hpp"
#include "Arachne/utils/linearAlg2D.hpp"

namespace slic3r_ffi {
namespace proof_units {
namespace linear_alg2_d {

int64_t path1_point_is_left_of_line(int64_t px, int64_t py, int64_t ax, int64_t ay, int64_t bx,
                                     int64_t by) {
    const Slic3r::Point p(px, py);
    const Slic3r::Point a(ax, ay);
    const Slic3r::Point b(bx, by);
    return Slic3r::Arachne::LinearAlg2D::pointIsLeftOfLine(p, a, b);
}

float path1_get_angle_left(int64_t ax, int64_t ay, int64_t bx, int64_t by, int64_t cx,
                            int64_t cy) {
    const Slic3r::Point a(ax, ay);
    const Slic3r::Point b(bx, by);
    const Slic3r::Point c(cx, cy);
    return Slic3r::Arachne::LinearAlg2D::getAngleLeft(a, b, c);
}

} // namespace linear_alg2_d
} // namespace proof_units
} // namespace slic3r_ffi
