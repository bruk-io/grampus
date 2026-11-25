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
// Two guards this bridge owns, not the vendored header. Both exist because this fn is a safe
// Rust entry point over header-only template code whose preconditions are `assert()`s, and this
// crate compiles without `-DNDEBUG` (the divergence `build.rs` records), so those asserts are
// live here: `__assert_rtn` aborts the process, which no Rust `catch_unwind` can turn back into
// an error. Measured, not assumed: `nm -u` on this file's compiled object lists one undefined
// `___assert_rtn`, and `strings` on it contains the literal `distance >= .0 && distance <= 1.0`.
// Neither guard's input is reachable from the one in-scope caller
// (`src/libslic3r/Support/TreeSupport.cpp:3827-3842` closes its contour first and walks
// `while (distance < 1)` up from `distance = 0`), so both are this bridge's own, not a
// vendored-code fix.
//
//  1. Contour length. `EdgeCache::createCache` (`nfpplacer.hpp:173-186`) takes
//     `next = std::next(first)` before checking `first != endit`, so a contour of fewer than two
//     points walks `next` past `endit` with no bounds check.
//
//  2. `distance` range. `EdgeCache::coords(double)` (`nfpplacer.hpp:303`) delegates to the
//     private `coords(cache, distance)` (`:246`), whose first statement is
//     `assert(distance >= .0 && distance <= 1.0)` (`:247`) -- ahead of every other check in that
//     function, including the `cache.distances.empty()` early return at `:248`. The vendored
//     header documents the `<0.0, 1.0>` contract (`:294-302`) but enforces it only by that
//     assert; with `-DNDEBUG` it would instead `std::fmod` a distance above 1.0 (`:249`) and
//     silently extrapolate backwards along the first edge for a negative one. This bridge
//     refuses out-of-range input rather than emulating either, so its behaviour does not depend
//     on whether the translation unit was built with `-DNDEBUG`.
//
// Both refusals return zeroed stats, the one refusal shape this bridge has.
CEdgeCacheStats libnest2d_edge_cache_stats(const rust::Vec<CPoint> &contour, double distance) {
    if (contour.size() < 2)
        return CEdgeCacheStats{};
    if (!(distance >= 0.0 && distance <= 1.0)) // also refuses NaN, which the assert rejects too
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
