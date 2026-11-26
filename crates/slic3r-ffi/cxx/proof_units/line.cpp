// Proof shims for the `Line.cpp`/`Line.hpp` unit (R19, proof paths 1 and 2, task
// `61-foundation-line-slicing`), compiled onto `crates/slic3r-ffi`'s shared `cxx_build::Build`
// by this crate's `build.rs` -- the one sanctioned route a ported unit reaches the oracle by
// (`docs/PORTING.md`, "Decisions recorded on 2026-11-25").
//
// `path1_intersection_infinite` calls `Slic3r::Line::intersection_infinite` (`Line.cpp:27-46`),
// genuinely linked `Line.cpp` code, proof path 1.
//
// `path2_line_length`/`path2_line_midpoint_normal` drive `Line.hpp`'s own header-only, inline
// `length()`, `midpoint()` and `normal()` (`Line.hpp:166-167,183`) directly through a real
// `Slic3r::Line` -- `Line.hpp`'s own row is `proof path: 2` with no stage-dump field that is any
// one of these calls' own output in isolation, so this proof reaches the oracle through the
// per-unit bridge instead, the same `proof path: 2`-with-no-dump-coverage shape
// `docs/PORTING.md`'s "Decisions recorded on 2026-11-26", decision 2, accepts.
#include "line.h"

#include "Line.hpp"

namespace slic3r_ffi {
namespace proof_units {
namespace line {

rust::Vec<int64_t> path1_intersection_infinite(int64_t ax, int64_t ay, int64_t bx, int64_t by,
                                                int64_t cx, int64_t cy, int64_t dx, int64_t dy) {
    const Slic3r::Line l1(Slic3r::Point(ax, ay), Slic3r::Point(bx, by));
    const Slic3r::Line l2(Slic3r::Point(cx, cy), Slic3r::Point(dx, dy));
    Slic3r::Point intersection;
    bool hit = l1.intersection_infinite(l2, &intersection);
    rust::Vec<int64_t> out;
    if (hit) {
        out.push_back(1);
        out.push_back(static_cast<int64_t>(intersection.x()));
        out.push_back(static_cast<int64_t>(intersection.y()));
    } else {
        out.push_back(0);
        out.push_back(0);
        out.push_back(0);
    }
    return out;
}

double path2_line_length(int64_t ax, int64_t ay, int64_t bx, int64_t by) {
    const Slic3r::Line line(Slic3r::Point(ax, ay), Slic3r::Point(bx, by));
    return line.length();
}

rust::Vec<int64_t> path2_line_midpoint_normal(int64_t ax, int64_t ay, int64_t bx, int64_t by) {
    const Slic3r::Line line(Slic3r::Point(ax, ay), Slic3r::Point(bx, by));
    const Slic3r::Point mid = line.midpoint();
    const Slic3r::Point normal = line.normal();
    rust::Vec<int64_t> out;
    out.push_back(static_cast<int64_t>(mid.x()));
    out.push_back(static_cast<int64_t>(mid.y()));
    out.push_back(static_cast<int64_t>(normal.x()));
    out.push_back(static_cast<int64_t>(normal.y()));
    return out;
}

// `path2_line_distance_to_infinite_squared` drives `Line.hpp`'s own header-only, inline
// `distance_to_infinite_squared(const Point&, Point*)` (`:173`) -- added on review: this
// function's own row (proof path 2, the `Line.hpp` cell) had no oracle comparison anywhere in
// the tree, which is what let `line_distance_to_infinite_squared_i64`'s wrong formula stand.
rust::Vec<double> path2_line_distance_to_infinite_squared(int64_t ax, int64_t ay, int64_t bx,
                                                           int64_t by, int64_t px, int64_t py) {
    const Slic3r::Line line(Slic3r::Point(ax, ay), Slic3r::Point(bx, by));
    const Slic3r::Point point(px, py);
    Slic3r::Point closest;
    double sq_dist = line.distance_to_infinite_squared(point, &closest);
    rust::Vec<double> out;
    out.push_back(sq_dist);
    out.push_back(static_cast<double>(closest.x()));
    out.push_back(static_cast<double>(closest.y()));
    return out;
}

} // namespace line
} // namespace proof_units
} // namespace slic3r_ffi
