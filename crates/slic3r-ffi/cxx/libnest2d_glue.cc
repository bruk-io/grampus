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
//
// Guard this bridge owns, not the vendored header: `EdgeCache::createCache`
// (`nfpplacer.hpp:173-186`) takes `next = std::next(first)` before checking `first != endit`, so
// a contour of fewer than two points walks `next` past `endit` with no bounds check. No in-scope
// caller constructs an `EdgeCache` over such a contour, but this fn is a safe Rust entry point,
// so it refuses the input itself rather than relying on that.
CEdgeCacheStats libnest2d_edge_cache_stats(const rust::Vec<CPoint> &contour, double distance) {
    if (contour.size() < 2)
        return CEdgeCacheStats{};

    Slic3r::ExPolygon expoly;
    expoly.contour.points.reserve(contour.size());
    for (const CPoint &p : contour)
        expoly.contour.points.emplace_back(p.x, p.y);

    libnest2d::placers::EdgeCache<Slic3r::ExPolygon> cache(expoly);

    CEdgeCacheStats stats{};
    stats.circumference = cache.circumference();
    Slic3r::Point point = cache.coords(distance);
    stats.point_at_distance.x = point.x();
    stats.point_at_distance.y = point.y();
    return stats;
}

} // namespace nest2d
} // namespace slic3r_ffi
