// Proof shims for the `Geometry/Circle.cpp`/`Circle.hpp` unit (R19, proof paths 1 and 2, task
// `76-foundation-voronoi-curves`), compiled onto `crates/slic3r-ffi`'s shared `cxx_build::Build`
// by this crate's `build.rs`.
//
// `path1_*` calls `Circle.cpp`'s own, non-template functions, externally linked in the archive
// `just oracle-build` produces. `path2_*` drives `Circle.hpp`'s own header-only templates,
// instantiated directly in this translation unit -- the same technique `bounding_box.cpp`'s
// `path2_*` methods use for `BoundingBoxBase`.
#include "circle.h"

#include "Geometry/Circle.hpp"
#include "libslic3r/Point.hpp"

namespace slic3r_ffi {
namespace proof_units {
namespace circle {

namespace {
Slic3r::Vec2ds unpack_vec2ds(const rust::Vec<double> &flat) {
    Slic3r::Vec2ds input;
    input.reserve(flat.size() / 2);
    for (size_t i = 0; i + 1 < flat.size(); i += 2) {
        input.emplace_back(flat[i], flat[i + 1]);
    }
    return input;
}
} // namespace

rust::Vec<double> path1_circle_center_taubin_newton(rust::Vec<double> flat, uint64_t cycles) {
    Slic3r::Vec2ds input = unpack_vec2ds(flat);
    Slic3r::Vec2d center = Slic3r::Geometry::circle_center_taubin_newton(input, static_cast<size_t>(cycles));
    rust::Vec<double> out;
    out.push_back(center.x());
    out.push_back(center.y());
    return out;
}

rust::Vec<double> path1_circle_taubin_newton(rust::Vec<double> flat, uint64_t cycles) {
    Slic3r::Vec2ds input = unpack_vec2ds(flat);
    Slic3r::Geometry::Circled c = Slic3r::Geometry::circle_taubin_newton(input, static_cast<size_t>(cycles));
    rust::Vec<double> out;
    out.push_back(c.center.x());
    out.push_back(c.center.y());
    out.push_back(c.radius);
    return out;
}

rust::Vec<int64_t> path1_circle_center_taubin_newton_point(rust::Vec<int64_t> flat, uint64_t cycles) {
    Slic3r::Points points;
    points.reserve(flat.size() / 2);
    for (size_t i = 0; i + 1 < flat.size(); i += 2) {
        points.emplace_back(flat[i], flat[i + 1]);
    }
    Slic3r::Point center = Slic3r::Geometry::circle_center_taubin_newton(
        points.cbegin(), points.cend(), static_cast<size_t>(cycles));
    rust::Vec<int64_t> out;
    out.push_back(center.x());
    out.push_back(center.y());
    return out;
}

rust::Vec<double> path2_circle_center(double ax, double ay, double bx, double by, double cx,
                                       double cy, double epsilon) {
    Slic3r::Vec2d a(ax, ay), b(bx, by), c(cx, cy);
    Slic3r::Vec2d center = Slic3r::Geometry::circle_center<Slic3r::Vec2d>(a, b, c, epsilon);
    rust::Vec<double> out;
    out.push_back(center.x());
    out.push_back(center.y());
    return out;
}

rust::Vec<double> path2_smallest_enclosing_circle_welzl(rust::Vec<int64_t> flat) {
    Slic3r::Points points;
    points.reserve(flat.size() / 2);
    for (size_t i = 0; i + 1 < flat.size(); i += 2) {
        points.emplace_back(flat[i], flat[i + 1]);
    }
    Slic3r::Geometry::Circled c = Slic3r::Geometry::smallest_enclosing_circle_welzl(points);
    rust::Vec<double> out;
    out.push_back(c.center.x());
    out.push_back(c.center.y());
    out.push_back(c.radius);
    return out;
}

rust::Vec<double> path2_ray_circle_intersections(double r2, double a, double b, double lv2,
                                                  double c) {
    std::pair<Eigen::Matrix<double, 2, 1, Eigen::DontAlign>,
              Eigen::Matrix<double, 2, 1, Eigen::DontAlign>>
        out_pair;
    int count = Slic3r::Geometry::ray_circle_intersections_r2_lv2_c<double>(r2, a, b, lv2, c, out_pair);
    rust::Vec<double> out;
    out.push_back(static_cast<double>(count));
    if (count > 0) {
        out.push_back(out_pair.first.x());
        out.push_back(out_pair.first.y());
        out.push_back(out_pair.second.x());
        out.push_back(out_pair.second.y());
    } else {
        // `out_pair` is left untouched by the vendored function when it returns `0` (`d2 < 0`),
        // so its contents are unspecified; this shim reports zeros rather than that garbage, and
        // the Rust-side proof only compares the count for this case (see
        // `crates/slic3r-geometry-dir/src/circle.rs`'s own test).
        out.push_back(0.0);
        out.push_back(0.0);
        out.push_back(0.0);
        out.push_back(0.0);
    }
    return out;
}

} // namespace circle
} // namespace proof_units
} // namespace slic3r_ffi
