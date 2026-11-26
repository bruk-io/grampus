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

// The three shims below drive `Point.hpp`'s own header-only `scaled<Tout>()`/`unscaled<T>()`
// templates (`:669-678`, `:692-700`) directly, each instantiated in this translation unit --
// the technique `cxx/proof_units/mt_utils.cpp`'s `Slic3r::grid<double>` instantiation already
// uses for a `proof path: 2` header-only row. `path2_point_rotate` above only reaches
// `Point.cpp`'s own `Point::rotate` definition; these three are what actually exercise
// `Point.hpp`'s declared-and-defined-in-the-header surface for this unit's `// PROOF: 2` row.
int64_t path2_scaled_coord_f64(double v) {
    // `Slic3r::scaled<coord_t>(const double &v)` (`Point.hpp:669-678`, body at `:677`): the
    // integral overload, explicit `Tout = coord_t`, `Tin` deduced as `double`.
    return static_cast<int64_t>(Slic3r::scaled<coord_t>(v));
}

int64_t path2_scaled_coord_f32(float v) {
    // Same function, `Tin` deduced as `float` -- the input is divided at its own precision,
    // never widened to `double` first (`docs/PORTING.md`'s "Scaling conversions" rule).
    return static_cast<int64_t>(Slic3r::scaled<coord_t>(v));
}

double path2_unscaled_f64(int64_t v) {
    // `Slic3r::unscaled<double>(const coord_t &v)` (`Point.hpp:692-700`, body at `:699`):
    // explicit `Tout = double`, `Tin` deduced as `coord_t`.
    return Slic3r::unscaled<double>(static_cast<coord_t>(v));
}

// `Point::nearest_point_index(const Points &points)` (`Point.cpp:197-204`), the `Points`-
// (`std::vector<Point>`-)consuming entry point `crates/slic3r-ffi/fuzz/fuzz_targets/
// slic3r-point_point.rs`'s arbitrary-`Points` arm drives differentially -- genuinely linked
// `Point.cpp` code, proof path 1, the same route `path1_int128_orient` above uses.
int32_t path1_nearest_point_index(rust::Slice<const int64_t> xs, rust::Slice<const int64_t> ys,
                                   int64_t qx, int64_t qy) {
    Slic3r::Points points;
    points.reserve(xs.size());
    for (size_t i = 0; i < xs.size(); ++i) {
        points.emplace_back(static_cast<coord_t>(xs[i]), static_cast<coord_t>(ys[i]));
    }
    const Slic3r::Point q(qx, qy);
    return q.nearest_point_index(points);
}

} // namespace point
} // namespace proof_units
} // namespace slic3r_ffi
