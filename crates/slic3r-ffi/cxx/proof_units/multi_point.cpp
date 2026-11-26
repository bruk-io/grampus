// Proof shims for the `MultiPoint.cpp`/`MultiPoint.hpp` unit (R19, proof paths 1 and 2, task
// `62-foundation-polygon`), compiled onto `crates/slic3r-ffi`'s shared `cxx_build::Build` by this
// crate's `build.rs`. `MultiPoint` is abstract (`last_point()`/`lines()` pure virtual, no public
// constructor), so every shim below drives its methods through a real `Slic3r::Polygon` --
// `#include "Polygon.hpp"` only (never `Polygon.cpp`: these functions need no internal-linkage
// symbol, so there is no reason to widen the duplicate-symbol exposure `cxx/proof_units/
// polygon.cpp`'s own header comment measures separately for that unit's own proof).
#include "multi_point.h"

#include "BoundingBox.hpp"
#include "Polygon.hpp"

namespace slic3r_ffi {
namespace proof_units {
namespace multi_point {

namespace {

Slic3r::Polygon make_polygon(rust::Slice<const int64_t> xs, rust::Slice<const int64_t> ys) {
    Slic3r::Points pts;
    pts.reserve(xs.size());
    for (size_t i = 0; i < xs.size(); ++i)
        pts.emplace_back(xs[i], ys[i]);
    return Slic3r::Polygon(pts);
}

rust::Vec<int64_t> pack_points(const Slic3r::Points &pts) {
    rust::Vec<int64_t> out;
    out.reserve(pts.size() * 2);
    for (const Slic3r::Point &p : pts) {
        out.push_back(static_cast<int64_t>(p.x()));
        out.push_back(static_cast<int64_t>(p.y()));
    }
    return out;
}

} // namespace

rust::Vec<int64_t> path1_scale(rust::Slice<const int64_t> xs, rust::Slice<const int64_t> ys,
                                double factor) {
    Slic3r::Polygon poly = make_polygon(xs, ys);
    poly.scale(factor);
    return pack_points(poly.points);
}

rust::Vec<int64_t> path1_translate(rust::Slice<const int64_t> xs, rust::Slice<const int64_t> ys,
                                    int64_t vx, int64_t vy) {
    Slic3r::Polygon poly = make_polygon(xs, ys);
    poly.translate(Slic3r::Point(vx, vy));
    return pack_points(poly.points);
}

rust::Vec<int64_t> path1_rotate_cos_sin(rust::Slice<const int64_t> xs,
                                        rust::Slice<const int64_t> ys, double cos_angle,
                                        double sin_angle) {
    Slic3r::Polygon poly = make_polygon(xs, ys);
    poly.rotate(cos_angle, sin_angle);
    return pack_points(poly.points);
}

double path1_length(rust::Slice<const int64_t> xs, rust::Slice<const int64_t> ys) {
    Slic3r::Polygon poly = make_polygon(xs, ys);
    return poly.MultiPoint::length();
}

int32_t path1_find_point(rust::Slice<const int64_t> xs, rust::Slice<const int64_t> ys,
                          int64_t px, int64_t py) {
    Slic3r::Polygon poly = make_polygon(xs, ys);
    return static_cast<int32_t>(poly.find_point(Slic3r::Point(px, py)));
}

rust::Vec<int64_t> path1_bounding_box(rust::Slice<const int64_t> xs,
                                       rust::Slice<const int64_t> ys) {
    Slic3r::Polygon poly = make_polygon(xs, ys);
    Slic3r::BoundingBox bb = poly.bounding_box();
    rust::Vec<int64_t> out;
    out.push_back(static_cast<int64_t>(bb.min.x()));
    out.push_back(static_cast<int64_t>(bb.min.y()));
    out.push_back(static_cast<int64_t>(bb.max.x()));
    out.push_back(static_cast<int64_t>(bb.max.y()));
    return out;
}

rust::Vec<int64_t> path1_intersection(rust::Slice<const int64_t> xs,
                                       rust::Slice<const int64_t> ys, int64_t ax, int64_t ay,
                                       int64_t bx, int64_t by) {
    Slic3r::Polygon poly = make_polygon(xs, ys);
    Slic3r::Line line(Slic3r::Point(ax, ay), Slic3r::Point(bx, by));
    Slic3r::Point out;
    bool found = poly.MultiPoint::intersection(line, &out);
    rust::Vec<int64_t> result;
    result.push_back(found ? 1 : 0);
    result.push_back(found ? static_cast<int64_t>(out.x()) : 0);
    result.push_back(found ? static_cast<int64_t>(out.y()) : 0);
    return result;
}

rust::Vec<int64_t> path1_symmetric_y(rust::Slice<const int64_t> xs,
                                      rust::Slice<const int64_t> ys, int64_t x_axis) {
    Slic3r::Polygon poly = make_polygon(xs, ys);
    poly.symmetric_y(x_axis);
    return pack_points(poly.points);
}

namespace {
Slic3r::Points to_points(rust::Slice<const int64_t> xs, rust::Slice<const int64_t> ys) {
    Slic3r::Points pts;
    pts.reserve(xs.size());
    for (size_t i = 0; i < xs.size(); ++i)
        pts.emplace_back(xs[i], ys[i]);
    return pts;
}
} // namespace

rust::Vec<int64_t> path1_concave_hull_2d(rust::Slice<const int64_t> xs,
                                          rust::Slice<const int64_t> ys, double tolerance) {
    Slic3r::Points hull = Slic3r::MultiPoint::concave_hull_2d(to_points(xs, ys), tolerance);
    return pack_points(hull);
}

double path1_min_distance_between_lines(rust::Slice<const int64_t> a_xs,
                                         rust::Slice<const int64_t> a_ys,
                                         rust::Slice<const int64_t> b_xs,
                                         rust::Slice<const int64_t> b_ys) {
    return Slic3r::MultiPoint::minimumDistanceBetweenLinesDefinedByPoints(
        to_points(a_xs, a_ys), to_points(b_xs, b_ys));
}

rust::Vec<int64_t> path1_get_extents_rotated(rust::Slice<const int64_t> xs,
                                              rust::Slice<const int64_t> ys, double angle) {
    Slic3r::BoundingBox bb = Slic3r::get_extents_rotated(to_points(xs, ys), angle);
    rust::Vec<int64_t> out;
    out.push_back(static_cast<int64_t>(bb.min.x()));
    out.push_back(static_cast<int64_t>(bb.min.y()));
    out.push_back(static_cast<int64_t>(bb.max.x()));
    out.push_back(static_cast<int64_t>(bb.max.y()));
    out.push_back(bb.defined ? 1 : 0);
    return out;
}

rust::Vec<int64_t> path2_front_back(rust::Slice<const int64_t> xs,
                                     rust::Slice<const int64_t> ys) {
    Slic3r::Polygon poly = make_polygon(xs, ys);
    const Slic3r::Point &front = poly.front();
    const Slic3r::Point &back = poly.back();
    rust::Vec<int64_t> out;
    out.push_back(static_cast<int64_t>(front.x()));
    out.push_back(static_cast<int64_t>(front.y()));
    out.push_back(static_cast<int64_t>(back.x()));
    out.push_back(static_cast<int64_t>(back.y()));
    return out;
}

rust::Vec<int64_t> path2_closest_point(rust::Slice<const int64_t> xs,
                                        rust::Slice<const int64_t> ys, int64_t px, int64_t py) {
    Slic3r::Polygon poly = make_polygon(xs, ys);
    const Slic3r::Point *cp = poly.closest_point(Slic3r::Point(px, py));
    rust::Vec<int64_t> out;
    if (cp == nullptr) {
        out.push_back(0);
        out.push_back(0);
        out.push_back(0);
    } else {
        out.push_back(1);
        out.push_back(static_cast<int64_t>(cp->x()));
        out.push_back(static_cast<int64_t>(cp->y()));
    }
    return out;
}

double path2_distance_to(rust::Slice<const int64_t> xs, rust::Slice<const int64_t> ys, int64_t px,
                          int64_t py) {
    Slic3r::Polygon poly = make_polygon(xs, ys);
    return poly.distance_to(Slic3r::Point(px, py));
}

bool path2_is_valid(rust::Slice<const int64_t> xs, rust::Slice<const int64_t> ys) {
    Slic3r::Polygon poly = make_polygon(xs, ys);
    return poly.MultiPoint::is_valid();
}

double path2_length_free(rust::Slice<const int64_t> xs, rust::Slice<const int64_t> ys) {
    Slic3r::Points pts;
    pts.reserve(xs.size());
    for (size_t i = 0; i < xs.size(); ++i)
        pts.emplace_back(xs[i], ys[i]);
    return Slic3r::length(pts);
}

double path2_area_free(rust::Slice<const int64_t> xs, rust::Slice<const int64_t> ys) {
    Slic3r::Points pts;
    pts.reserve(xs.size());
    for (size_t i = 0; i < xs.size(); ++i)
        pts.emplace_back(xs[i], ys[i]);
    return Slic3r::area(pts);
}

} // namespace multi_point
} // namespace proof_units
} // namespace slic3r_ffi
