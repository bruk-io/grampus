#include "libnest2d_shim.h"

#include <libnest2d/libnest2d.hpp>
#include <libslic3r/ExPolygon.hpp>

namespace slic3r_ffi {
namespace nest2d {

// The surface `src/libslic3r/Support/TreeSupport.cpp:3834` reaches:
// `libnest2d::placers::EdgeCache<ExPolygon>`, constructed straight from an `ExPolygon` the same
// way (`nfpplacer.hpp:286`). `EdgeCache` is header-only template code -- a later task ports
// `TreeSupport.cpp` itself against this binding, this one just proves it compiles and runs
// against this crate's build of `libnest2d.cpp`.
CEdgeCacheStats libnest2d_edge_cache_stats(const rust::Vec<CPoint> &contour) {
    Slic3r::ExPolygon expoly;
    expoly.contour.points.reserve(contour.size());
    for (const CPoint &p : contour)
        expoly.contour.points.emplace_back(p.x, p.y);

    libnest2d::placers::EdgeCache<Slic3r::ExPolygon> cache(expoly);

    CEdgeCacheStats stats{};
    stats.circumference = cache.circumference();
    Slic3r::Point first = cache.coords(0.0);
    stats.first_point.x = first.x();
    stats.first_point.y = first.y();
    return stats;
}

} // namespace nest2d
} // namespace slic3r_ffi
