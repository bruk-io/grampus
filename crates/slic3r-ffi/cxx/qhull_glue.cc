#include "qhull_shim.h"

// Same include order `src/libslic3r/TriangleMesh.cpp:22-29` uses for this surface.
#include <libqhull_r/libqhull_r.h>
#include <libqhullcpp/Qhull.h>
#include <libqhullcpp/QhullFacetList.h>
#include <libqhullcpp/QhullVertex.h>
#include <libqhullcpp/QhullPoint.h>
#include <libqhullcpp/QhullVertexSet.h>

namespace slic3r_ffi {
namespace qhull {

// The qhull call: same shape as TriangleMesh.cpp:1364-1372's `orgQhull::Qhull::runQhull("", 3,
// count, coords, "Qt")` -- "Qt" triangulates the output, so facetList().count() is the number
// of triangles qhull itself produced, not one this bridge computed.
CHullStats qhull_convex_hull_stats(const rust::Vec<CPoint3> &points) {
    std::vector<realT> coords;
    coords.reserve(points.size() * 3);
    for (const CPoint3 &p : points) {
        coords.push_back(static_cast<realT>(p.x));
        coords.push_back(static_cast<realT>(p.y));
        coords.push_back(static_cast<realT>(p.z));
    }

    orgQhull::Qhull qhull;
    qhull.disableOutputStream(); // we want qhull to be quiet
    qhull.runQhull("", 3, static_cast<int>(points.size()), coords.data(), "Qt");

    CHullStats stats;
    stats.facet_count = static_cast<int32_t>(qhull.facetList().count());
    stats.vertex_count = static_cast<int32_t>(qhull.vertexList().count());
    return stats;
}

} // namespace qhull
} // namespace slic3r_ffi
