// Proof shims for the `PolygonsSegmentIndex.hpp` unit (R19, proof path 1, task
// `73-foundation-arachne-indices`), compiled onto `crates/slic3r-ffi`'s shared
// `cxx_build::Build` -- the one sanctioned route a ported unit reaches the oracle by
// (`docs/PORTING.md`, "Decisions recorded on 2026-11-25").
//
// `path1_from_to` builds a one-polygon `Slic3r::Polygons` from parallel `xs`/`ys` slices (the
// same technique `cxx/proof_units/polygons_point_index.cpp` uses), constructs a
// `Slic3r::Arachne::PolygonsSegmentIndex` at `point_idx`, and reads back `from()`/`to()` as
// four packed `int64_t`s.
#include "polygons_segment_index.h"

#include "Arachne/utils/PolygonsSegmentIndex.hpp"
#include "Point.hpp"
#include "Polygon.hpp"

namespace slic3r_ffi {
namespace proof_units {
namespace polygons_segment_index {

rust::Vec<int64_t> path1_from_to(rust::Slice<const int64_t> xs, rust::Slice<const int64_t> ys,
                                  uint32_t point_idx) {
    Slic3r::Points points;
    for (size_t i = 0; i < xs.size(); ++i) {
        points.emplace_back(xs[i], ys[i]);
    }
    Slic3r::Polygons polys;
    polys.emplace_back(points);

    Slic3r::Arachne::PolygonsSegmentIndex seg(&polys, 0, point_idx);
    const Slic3r::Point from = seg.from();
    const Slic3r::Point to = seg.to();

    rust::Vec<int64_t> out;
    out.push_back(static_cast<int64_t>(from.x()));
    out.push_back(static_cast<int64_t>(from.y()));
    out.push_back(static_cast<int64_t>(to.x()));
    out.push_back(static_cast<int64_t>(to.y()));
    return out;
}

} // namespace polygons_segment_index
} // namespace proof_units
} // namespace slic3r_ffi
