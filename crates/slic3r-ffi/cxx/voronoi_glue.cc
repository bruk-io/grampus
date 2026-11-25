#include "voronoi_shim.h"

// Boost is a superbuild project (`vendor/OrcaSlicer/deps/Boost/`), so these are the only Boost
// headers this translation unit includes, and they resolve against the superbuild prefix's
// `include` directory alone -- `build.rs` puts no other Boost copy on this unit's include path
// (there is none vendored). Same include order `src/libslic3r/Geometry/Voronoi.hpp` itself uses
// (`:5` then `:17`): `polygon.hpp` is what actually defines the `point_data`/`segment_data`
// class templates `voronoi.hpp` alone only declares concepts and traits for.
#include <boost/polygon/polygon.hpp>
#include <boost/polygon/voronoi.hpp>

namespace slic3r_ffi {
namespace voronoi {

namespace bp = boost::polygon;

// `src/libslic3r/Geometry/Voronoi.hpp:33-36`'s own aliases: the diagram is always
// `voronoi_diagram<double>` regardless of the integer input coordinate type construct_voronoi
// consumes (`boost/polygon/voronoi.hpp`'s own doc comment: input coordinates are 32-bit signed
// integers; the builder's internal fixed-point arithmetic produces the `double` diagram).
using VD = bp::voronoi_diagram<double>;

CVoronoiStats voronoi_diagram_stats(const rust::Vec<CSegment> &segments) {
    std::vector<bp::segment_data<std::int32_t>> input;
    input.reserve(segments.size());
    for (const CSegment &s : segments) {
        input.emplace_back(bp::point_data<std::int32_t>(s.x0, s.y0),
                            bp::point_data<std::int32_t>(s.x1, s.y1));
    }

    VD vd;
    bp::construct_voronoi(input.begin(), input.end(), &vd);

    CVoronoiStats stats{};
    stats.cell_count = static_cast<std::int32_t>(vd.num_cells());
    stats.vertex_count = static_cast<std::int32_t>(vd.num_vertices());
    stats.edge_count = static_cast<std::int32_t>(vd.num_edges());
    return stats;
}

} // namespace voronoi
} // namespace slic3r_ffi
