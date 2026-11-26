// Proof shims for the `PolygonsPointIndex.hpp` unit (R19, proof path 1, task
// `73-foundation-arachne-indices`), compiled onto `crates/slic3r-ffi`'s shared
// `cxx_build::Build` -- the one sanctioned route a ported unit reaches the oracle by
// (`docs/PORTING.md`, "Decisions recorded on 2026-11-25").
//
// `path1_p_next_prev` builds a one-polygon `Slic3r::Polygons` from parallel `xs`/`ys` slices
// (`cxx` has no `Vec<(i64, i64)>`, the same technique `point.cpp`'s own
// `path1_nearest_point_index` uses), constructs a `Slic3r::Arachne::PolygonsPointIndex` at
// `point_idx`, and reads back `p()`, `next().p()` and `prev().p()` as six packed `int64_t`s.
#include "polygons_point_index.h"

#include "Arachne/utils/PolygonsPointIndex.hpp"
#include "Point.hpp"
#include "Polygon.hpp"

namespace slic3r_ffi {
namespace proof_units {
namespace polygons_point_index {

rust::Vec<int64_t> path1_p_next_prev(rust::Slice<const int64_t> xs, rust::Slice<const int64_t> ys,
                                      uint32_t point_idx) {
    Slic3r::Points points;
    for (size_t i = 0; i < xs.size(); ++i) {
        points.emplace_back(xs[i], ys[i]);
    }
    Slic3r::Polygons polys;
    polys.emplace_back(points);

    Slic3r::Arachne::PolygonsPointIndex idx(&polys, 0, point_idx);
    const Slic3r::Point p = idx.p();
    const Slic3r::Point next = idx.next().p();
    const Slic3r::Point prev = idx.prev().p();

    rust::Vec<int64_t> out;
    out.push_back(static_cast<int64_t>(p.x()));
    out.push_back(static_cast<int64_t>(p.y()));
    out.push_back(static_cast<int64_t>(next.x()));
    out.push_back(static_cast<int64_t>(next.y()));
    out.push_back(static_cast<int64_t>(prev.x()));
    out.push_back(static_cast<int64_t>(prev.y()));
    return out;
}

} // namespace polygons_point_index
} // namespace proof_units
} // namespace slic3r_ffi
