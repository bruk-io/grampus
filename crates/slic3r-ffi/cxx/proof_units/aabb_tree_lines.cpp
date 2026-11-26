// Proof shim for the `AABBTreeLines.hpp` unit (R19, proof path 2, task
// `61-foundation-line-slicing`), compiled onto `crates/slic3r-ffi`'s shared `cxx_build::Build`
// by this crate's `build.rs` -- the one sanctioned route a ported unit reaches the oracle by
// (`docs/PORTING.md`, "Decisions recorded on 2026-11-25").
//
// `AABBTreeLines.hpp`'s own row is `proof path: 2` with no stage-dump field that is this
// header's own output (its callers are all fan-out modules not yet written), so this proof
// reaches the oracle through the per-unit bridge instead -- `docs/PORTING.md`'s "Decisions
// recorded on 2026-11-26", decision 2, the same route `60-foundation-trees-grids`'s
// `AABBTreeIndirect.hpp` precedent (`aabb_tree_indirect.cpp`) uses. This builds a real
// `Slic3r::AABBTreeLines::LinesDistancer<Slic3r::Linef>` over the lines the Rust test passes in
// and calls its real vendored `distance_from_lines_extra<true>` and `outside`.
//
// `path2_all_lines_in_radius`/`path2_intersections_with_line` call the same real distancer's
// `all_lines_in_radius`/`intersections_with_line<true>` -- added on review: the first version
// of this shim proved only `distance_from_lines_extra`/`outside`, leaving these two unproven.
#include "aabb_tree_lines.h"

#include "libslic3r/AABBTreeLines.hpp"
#include "libslic3r/Line.hpp"

namespace slic3r_ffi {
namespace proof_units {
namespace aabb_tree_lines {

rust::Vec<double> path2_distance_and_outside(rust::Slice<const double> ax,
                                              rust::Slice<const double> ay,
                                              rust::Slice<const double> bx,
                                              rust::Slice<const double> by, double qx, double qy) {
    std::vector<Slic3r::Linef> lines;
    lines.reserve(ax.size());
    for (size_t i = 0; i < ax.size(); ++i) {
        lines.emplace_back(Eigen::Vector2d(ax[i], ay[i]), Eigen::Vector2d(bx[i], by[i]));
    }

    Slic3r::AABBTreeLines::LinesDistancer<Slic3r::Linef> distancer(std::move(lines));
    const Eigen::Vector2d point(qx, qy);
    auto [distance, idx, nearest] = distancer.distance_from_lines_extra<true>(point);
    int outside = distancer.outside(point);

    rust::Vec<double> out;
    out.push_back(distance);
    out.push_back(static_cast<double>(idx));
    out.push_back(nearest.x());
    out.push_back(nearest.y());
    out.push_back(static_cast<double>(outside));
    return out;
}

rust::Vec<int64_t> path2_all_lines_in_radius(rust::Slice<const double> ax,
                                              rust::Slice<const double> ay,
                                              rust::Slice<const double> bx,
                                              rust::Slice<const double> by, double qx, double qy,
                                              double radius) {
    std::vector<Slic3r::Linef> lines;
    lines.reserve(ax.size());
    for (size_t i = 0; i < ax.size(); ++i) {
        lines.emplace_back(Eigen::Vector2d(ax[i], ay[i]), Eigen::Vector2d(bx[i], by[i]));
    }

    Slic3r::AABBTreeLines::LinesDistancer<Slic3r::Linef> distancer(std::move(lines));
    const Eigen::Vector2d point(qx, qy);
    // Review finding: this used to sort the oracle's own result before returning it. The
    // vendored `all_lines_in_radius` (`AABBTreeLines.hpp:228-243`) returns indices in the
    // deterministic push order of `indexed_primitives_within_distance_squared_recurisve`
    // (`AABBTreeIndirect.hpp:650-666`), with no sort anywhere in its own call chain, and the
    // row's equality tier is `exact`; sorting both sides stood a set comparison in for the
    // sequence comparison the tier asks for.
    std::vector<size_t> found = distancer.all_lines_in_radius(point, radius);

    rust::Vec<int64_t> out;
    out.reserve(found.size());
    for (size_t idx : found) {
        out.push_back(static_cast<int64_t>(idx));
    }
    return out;
}

rust::Vec<double> path2_intersections_with_line(rust::Slice<const double> ax,
                                                 rust::Slice<const double> ay,
                                                 rust::Slice<const double> bx,
                                                 rust::Slice<const double> by, double lax,
                                                 double lay, double lbx, double lby) {
    std::vector<Slic3r::Linef> lines;
    lines.reserve(ax.size());
    for (size_t i = 0; i < ax.size(); ++i) {
        lines.emplace_back(Eigen::Vector2d(ax[i], ay[i]), Eigen::Vector2d(bx[i], by[i]));
    }

    Slic3r::AABBTreeLines::LinesDistancer<Slic3r::Linef> distancer(std::move(lines));
    const Slic3r::Linef probe(Eigen::Vector2d(lax, lay), Eigen::Vector2d(lbx, lby));
    auto hits = distancer.intersections_with_line<true>(probe);

    rust::Vec<double> out;
    out.reserve(hits.size() * 3);
    for (const auto &hit : hits) {
        out.push_back(hit.first.x());
        out.push_back(hit.first.y());
        out.push_back(static_cast<double>(hit.second));
    }
    return out;
}

} // namespace aabb_tree_lines
} // namespace proof_units
} // namespace slic3r_ffi
