// Proof shims for the `KDTreeIndirect.hpp` unit (R19, proof path 2, task
// `60-foundation-trees-grids`), compiled onto `crates/slic3r-ffi`'s shared `cxx_build::Build` --
// the one sanctioned route a ported unit reaches the oracle by (`docs/PORTING.md`,
// "Decisions recorded on 2026-11-25", decision 1; decision 2 of "Decisions recorded on
// 2026-11-26" is why a `proof path: 2` row is proven through this same per-unit bridge rather
// than a stage dump: this header-only template writes nothing any committed dump carries).
//
// Both shims build a real `Slic3r::KDTreeIndirect<2, double, ...>` over the same flattened point
// set the Rust test builds its own tree from, and call the real vendored
// `Slic3r::find_closest_point`/`Slic3r::find_nearby_points` free functions -- the same technique
// `cxx/proof_units/linear_alg2_d.cpp` uses for another header-only template, just with the point
// set passed across the bridge as a flat `rust::Vec<double>` (`x0,y0,x1,y1,...`) rather than
// fixed arguments, since the tree's own point count varies by test.
#include "kd_tree_indirect.h"

#include "KDTreeIndirect.hpp"

#include <array>
#include <vector>

namespace slic3r_ffi {
namespace proof_units {
namespace kd_tree_indirect {

namespace {

std::vector<std::array<double, 2>> unflatten(const rust::Vec<double> &coords) {
    std::vector<std::array<double, 2>> pts;
    pts.reserve(coords.size() / 2);
    for (size_t i = 0; i + 1 < coords.size(); i += 2) {
        pts.push_back({coords[i], coords[i + 1]});
    }
    return pts;
}

} // namespace

uint64_t path2_find_closest_point(const rust::Vec<double> &coords, double qx, double qy) {
    std::vector<std::array<double, 2>> pts = unflatten(coords);
    auto coordinate = [&pts](size_t idx, size_t dim) -> double { return pts[idx][dim]; };
    Slic3r::KDTreeIndirect<2, double, decltype(coordinate)> tree(coordinate, pts.size());
    std::array<double, 2> query{qx, qy};
    size_t idx = Slic3r::find_closest_point(tree, query);
    return static_cast<uint64_t>(idx);
}

rust::Vec<uint64_t> path2_find_nearby_points(const rust::Vec<double> &coords, double cx, double cy,
                                              double max_distance) {
    std::vector<std::array<double, 2>> pts = unflatten(coords);
    auto coordinate = [&pts](size_t idx, size_t dim) -> double { return pts[idx][dim]; };
    Slic3r::KDTreeIndirect<2, double, decltype(coordinate)> tree(coordinate, pts.size());
    std::array<double, 2> center{cx, cy};
    std::vector<size_t> found = Slic3r::find_nearby_points(tree, center, max_distance);

    rust::Vec<uint64_t> out;
    out.reserve(found.size());
    for (size_t idx : found) {
        out.push_back(static_cast<uint64_t>(idx));
    }
    return out;
}

} // namespace kd_tree_indirect
} // namespace proof_units
} // namespace slic3r_ffi
