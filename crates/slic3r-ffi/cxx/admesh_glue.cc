#include "admesh_shim.h"

#include "stl.h"

namespace slic3r_ffi {
namespace admesh {

// Builds an in-memory stl_file from the given facets (each already carrying its own outward
// unit normal -- admesh's get_volume() (util.cpp) uses the *stored* normal, it does not
// recompute one from the vertices), then calls the real `stl_calculate_volume`
// (deps_src/admesh/util.cpp) and returns exactly what it wrote to stl->stats.volume. No volume
// arithmetic happens on this side of the bridge.
float admesh_compute_volume(const rust::Vec<CFacet> &facets) {
    stl_file stl;
    stl.facet_start.reserve(facets.size());
    for (const CFacet &f : facets) {
        stl_facet sf{};
        sf.normal = stl_normal(f.normal.x, f.normal.y, f.normal.z);
        sf.vertex[0] = stl_vertex(f.v0.x, f.v0.y, f.v0.z);
        sf.vertex[1] = stl_vertex(f.v1.x, f.v1.y, f.v1.z);
        sf.vertex[2] = stl_vertex(f.v2.x, f.v2.y, f.v2.z);
        stl.facet_start.push_back(sf);
    }
    stl.stats.number_of_facets = static_cast<uint32_t>(stl.facet_start.size());

    stl_calculate_volume(&stl);

    return stl.stats.volume;
}

} // namespace admesh
} // namespace slic3r_ffi
