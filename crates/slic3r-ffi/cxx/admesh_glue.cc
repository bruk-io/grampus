#include "admesh_shim.h"

#include "stl.h"

namespace slic3r_ffi {
namespace admesh {

// Builds an in-memory stl_file from the given facets (each already carrying its own outward
// unit normal -- admesh's get_area() (util.cpp) re-derives its own normal from the vertices for
// the area term, but get_volume()'s height term reads the *stored* normal, so a facet whose
// stored normal disagrees with its vertices changes only that term, not the area one), then
// calls the real `stl_calculate_volume` (deps_src/admesh/util.cpp) and returns exactly what it
// wrote to stl->stats.volume. No volume arithmetic happens on this side of the bridge.
//
// Two guards this crate owns, not the vendored code: `get_volume` (util.cpp) indexes
// `facet_start[0]` unconditionally with no empty check, and `stl_calculate_volume` calls
// `stl_reverse_all_facets` (which indexes `neighbors_start`) whenever the computed volume is
// negative. The oracle's only caller, `TriangleMesh::repair` (TriangleMesh.cpp), always reaches
// this function after the repair chain has populated `neighbors_start`
// (`stl_check_facets_exact`/`stl_fix_normal_directions`); this bridge builds no such chain, so it
// guards both cases itself the same way `its_convex_hull` guards its own qhull call with `if (!
// pts.empty())` -- empty input returns 0.0 before the vendored code is reached, and
// `neighbors_start` is sized to one default-constructed (all-`-1`, "no neighbour") entry per
// facet so `stl_reverse_all_facets`'s neighbour-fixup branches, which are each gated on `!= -1`,
// are no-ops rather than out-of-bounds reads on an empty vector.
float admesh_compute_volume(const rust::Vec<CFacet> &facets) {
    if (facets.empty())
        return 0.0f;

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
    stl.neighbors_start.resize(stl.facet_start.size());

    stl_calculate_volume(&stl);

    return stl.stats.volume;
}

} // namespace admesh
} // namespace slic3r_ffi
