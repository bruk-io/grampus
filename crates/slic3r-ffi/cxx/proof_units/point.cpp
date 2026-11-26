// Proof shims for the `Point.cpp`/`Point.hpp` unit (R19, proof paths 1 and 2, task
// `56-foundation-point`), compiled onto `crates/slic3r-ffi`'s shared `cxx_build::Build` by this
// crate's `build.rs` -- the one sanctioned route a ported unit reaches the oracle by
// (`docs/PORTING.md`, "Decisions recorded on 2026-11-25").
//
// `path1_int128_orient` calls `Slic3r::int128::orient` (`Point.cpp:255-260`), an externally
// linked, non-template free function in the archive `just oracle-build` produces -- genuine
// `Point.cpp` code, not a forward into `Int128.cpp` alone (`slic3r-int128`'s own proof already
// covers `Int128::sign_determinant_2x2_filtered` itself).
//
// `path2_point_rotate` drives `Slic3r::Point::rotate(double angle, const Point &center)`
// (`Point.cpp:57-65`) directly through a real `Slic3r::Point` -- `Point.hpp`'s own row is
// `proof path: 2` (no externally linked symbol of its own; the method is defined in the `.cpp`,
// but the row this proves is the header's declared method surface, read through the oracle's
// real, compiled behaviour, not a stage dump: no committed dump field is `Point::rotate`'s own
// output, since `posSlice.dump`'s fields are `PrintObject`/`Layer` geometry, not this one method
// call in isolation).
#include "point.h"

#include "Point.hpp"

namespace slic3r_ffi {
namespace proof_units {
namespace point {

int32_t path1_int128_orient(int64_t p1x, int64_t p1y, int64_t p2x, int64_t p2y, int64_t p3x,
                             int64_t p3y) {
    const Slic3r::Vec2crd p1(p1x, p1y);
    const Slic3r::Vec2crd p2(p2x, p2y);
    const Slic3r::Vec2crd p3(p3x, p3y);
    return static_cast<int32_t>(Slic3r::int128::orient(p1, p2, p3));
}

rust::Vec<int64_t> path2_point_rotate(int64_t px, int64_t py, int64_t cx, int64_t cy,
                                       double angle) {
    Slic3r::Point p(px, py);
    const Slic3r::Point center(cx, cy);
    p.rotate(angle, center);
    rust::Vec<int64_t> out;
    out.push_back(static_cast<int64_t>(p.x()));
    out.push_back(static_cast<int64_t>(p.y()));
    return out;
}

} // namespace point
} // namespace proof_units
} // namespace slic3r_ffi
