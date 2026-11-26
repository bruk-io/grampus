// Proof shims for the `AABBTreeIndirect.hpp` unit (R19, proof path 2, task
// `60-foundation-trees-grids`), compiled onto `crates/slic3r-ffi`'s shared `cxx_build::Build` --
// the one sanctioned route a ported unit reaches the oracle by (`docs/PORTING.md`,
// "Decisions recorded on 2026-11-25", decision 1; decision 2 of "Decisions recorded on
// 2026-11-26" is why a `proof path: 2` row is proven through this same per-unit bridge rather
// than a stage dump: this header-only template writes nothing any committed dump carries).
//
// Both shims build a real `Slic3r::AABBTreeIndirect::Tree<3, double>` over the same flattened
// vertex/face arrays the Rust test builds its own tree from (`Eigen::Vector3d` is `Eigen::Matrix
// <double, 3, 1>`, used here as both `VertexType` and `VectorType` -- `AABBTreeIndirect.hpp`
// never requires them to differ), and call the real vendored
// `Slic3r::AABBTreeIndirect::intersect_ray_first_hit`/`squared_distance_to_indexed_triangle_set`
// free functions.
#include "aabb_tree_indirect.h"

#include "AABBTreeIndirect.hpp"

#include <Eigen/Core>
#include <vector>

namespace slic3r_ffi {
namespace proof_units {
namespace aabb_tree_indirect {

namespace {

std::vector<Eigen::Vector3d> unflatten_vertices(const rust::Vec<double> &vertices) {
    std::vector<Eigen::Vector3d> out;
    out.reserve(vertices.size() / 3);
    for (size_t i = 0; i + 2 < vertices.size(); i += 3) {
        out.emplace_back(vertices[i], vertices[i + 1], vertices[i + 2]);
    }
    return out;
}

std::vector<Eigen::Vector3i> unflatten_faces(const rust::Vec<int64_t> &faces) {
    std::vector<Eigen::Vector3i> out;
    out.reserve(faces.size() / 3);
    for (size_t i = 0; i + 2 < faces.size(); i += 3) {
        out.emplace_back(static_cast<int>(faces[i]), static_cast<int>(faces[i + 1]),
                          static_cast<int>(faces[i + 2]));
    }
    return out;
}

} // namespace

int64_t path2_intersect_ray_first_hit(const rust::Vec<double> &vertices,
                                       const rust::Vec<int64_t> &faces, double ox, double oy,
                                       double oz, double dx, double dy, double dz) {
    std::vector<Eigen::Vector3d> verts = unflatten_vertices(vertices);
    std::vector<Eigen::Vector3i> tris = unflatten_faces(faces);
    auto tree = Slic3r::AABBTreeIndirect::build_aabb_tree_over_indexed_triangle_set(verts, tris);

    igl::Hit<float> hit;
    Eigen::Vector3d origin(ox, oy, oz);
    Eigen::Vector3d dir(dx, dy, dz);
    // `intersect_ray_first_hit`'s own default argument (`AABBTreeIndirect.hpp:742`), not
    // `detail::intersect_triangle_epsilon` -- see the Rust module's `TODO(port)` note on why
    // that function is left unported (dead, non-compiling vendored code).
    bool found = Slic3r::AABBTreeIndirect::intersect_ray_first_hit(verts, tris, tree, origin, dir,
                                                                    hit);
    return found ? static_cast<int64_t>(hit.id) : -1;
}

PathTwoSquaredDistance path2_squared_distance(const rust::Vec<double> &vertices,
                                               const rust::Vec<int64_t> &faces, double px,
                                               double py, double pz) {
    std::vector<Eigen::Vector3d> verts = unflatten_vertices(vertices);
    std::vector<Eigen::Vector3i> tris = unflatten_faces(faces);
    auto tree = Slic3r::AABBTreeIndirect::build_aabb_tree_over_indexed_triangle_set(verts, tris);

    size_t hit_idx_out = 0;
    Eigen::Vector3d hit_point_out;
    Eigen::Vector3d point(px, py, pz);
    double sq = Slic3r::AABBTreeIndirect::squared_distance_to_indexed_triangle_set(
        verts, tris, tree, point, hit_idx_out, hit_point_out);

    PathTwoSquaredDistance out;
    out.sq = sq;
    out.idx = static_cast<int64_t>(hit_idx_out);
    return out;
}

} // namespace aabb_tree_indirect
} // namespace proof_units
} // namespace slic3r_ffi
