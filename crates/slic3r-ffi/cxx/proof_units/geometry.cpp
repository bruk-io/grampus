// Proof shims for the `Geometry.cpp`/`Geometry.hpp` unit (R19, task `66-foundation-geometry`).
// See `geometry.h`'s own comment for why these call the externally-linked `Slic3r::Geometry::`
// surface directly (the same mechanism proof path 1 uses) rather than `#include`ing
// `Geometry.cpp` whole.
#include "geometry.h"

#include "BoundingBox.hpp"
#include "Geometry.hpp"
#include "Point.hpp"
#include "Polygon.hpp"

namespace slic3r_ffi {
namespace proof_units {
namespace geometry {

int32_t path_orient(int64_t ax, int64_t ay, int64_t bx, int64_t by, int64_t cx, int64_t cy) {
    return static_cast<int32_t>(
        Slic3r::Geometry::orient(Slic3r::Point(ax, ay), Slic3r::Point(bx, by), Slic3r::Point(cx, cy)));
}

bool path_is_ccw(rust::Slice<const int64_t> xs, rust::Slice<const int64_t> ys) {
    Slic3r::Points pts;
    pts.reserve(xs.size());
    for (size_t i = 0; i < xs.size(); ++i)
        pts.emplace_back(xs[i], ys[i]);
    Slic3r::Polygon poly(pts);
    return Slic3r::Geometry::is_ccw(poly);
}

bool path_segments_intersect(int64_t ip1x, int64_t ip1y, int64_t ip2x, int64_t ip2y, int64_t jp1x,
                              int64_t jp1y, int64_t jp2x, int64_t jp2y) {
    return Slic3r::Geometry::segments_intersect(
        Slic3r::Point(ip1x, ip1y), Slic3r::Point(ip2x, ip2y), Slic3r::Point(jp1x, jp1y),
        Slic3r::Point(jp2x, jp2y));
}

rust::Vec<double> path_ray_ray_intersection(double p1x, double p1y, double v1x, double v1y,
                                             double p2x, double p2y, double v2x, double v2y) {
    Slic3r::Vec2d res;
    bool ok = Slic3r::Geometry::ray_ray_intersection(Slic3r::Vec2d(p1x, p1y), Slic3r::Vec2d(v1x, v1y),
                                                      Slic3r::Vec2d(p2x, p2y), Slic3r::Vec2d(v2x, v2y),
                                                      res);
    rust::Vec<double> out;
    out.push_back(ok ? 1.0 : 0.0);
    if (ok) {
        out.push_back(res.x());
        out.push_back(res.y());
    }
    return out;
}

rust::Vec<double> path_segment_segment_intersection(double p1x, double p1y, double v1x,
                                                     double v1y, double p2x, double p2y,
                                                     double v2x, double v2y) {
    Slic3r::Vec2d res;
    bool ok = Slic3r::Geometry::segment_segment_intersection(
        Slic3r::Vec2d(p1x, p1y), Slic3r::Vec2d(v1x, v1y), Slic3r::Vec2d(p2x, p2y),
        Slic3r::Vec2d(v2x, v2y), res);
    rust::Vec<double> out;
    out.push_back(ok ? 1.0 : 0.0);
    if (ok) {
        out.push_back(res.x());
        out.push_back(res.y());
    }
    return out;
}

double path_deg2rad(double angle) { return Slic3r::Geometry::deg2rad<double>(angle); }

double path_angle_to_0_2pi(double angle) { return Slic3r::Geometry::angle_to_0_2PI<double>(angle); }

double path_to_range_pi_pi(double angle) {
    Slic3r::Geometry::to_range_pi_pi<double>(angle);
    return angle;
}

bool path_directions_parallel(double angle1, double angle2, double max_diff) {
    return Slic3r::Geometry::directions_parallel(angle1, angle2, max_diff);
}

bool path_directions_perpendicular(double angle1, double angle2, double max_diff) {
    return Slic3r::Geometry::directions_perpendicular(angle1, angle2, max_diff);
}

double path_rad2deg_dir(double angle) { return Slic3r::Geometry::rad2deg_dir(angle); }

double path_linint(double value, double oldmin, double oldmax, double newmin, double newmax) {
    return Slic3r::Geometry::linint(value, oldmin, oldmax, newmin, newmax);
}

rust::Vec<double> path_arrange(size_t total_parts, double part_w, double part_h, double dist,
                                bool bb_defined, double bb_min_x, double bb_min_y,
                                double bb_max_x, double bb_max_y) {
    // Built through the real `BoundingBoxf(pmin, pmax)` constructor (`BoundingBox.hpp:260`),
    // which derives `defined` from `pmin.x() < pmax.x() && pmin.y() < pmax.y()` -- the same rule
    // `crates/slic3r-boundingbox/src/bounding_box.rs`'s `BoundingBoxf::from_min_max` uses -- so a
    // degenerate box takes the same `defined` branch on both sides of the comparison this proof
    // makes. Setting `min`/`max`/`defined` by hand (an earlier version of this shim) could
    // disagree with that rule and would be the shim's own bug, not the port's.
    Slic3r::BoundingBoxf bb(Slic3r::Vec2d(bb_min_x, bb_min_y), Slic3r::Vec2d(bb_max_x, bb_max_y));
    if (!bb_defined) {
        bb.reset();
    }
    Slic3r::Pointfs positions;
    bool ok = Slic3r::Geometry::arrange(total_parts, Slic3r::Vec2d(part_w, part_h), dist,
                                         bb_defined ? &bb : nullptr, positions);
    rust::Vec<double> out;
    out.push_back(ok ? 1.0 : 0.0);
    for (const Slic3r::Vec2d &p : positions) {
        out.push_back(p.x());
        out.push_back(p.y());
    }
    return out;
}

bool path_is_rotation_ninety_degrees(double a) { return Slic3r::Geometry::is_rotation_ninety_degrees(a); }

bool path_is_point_inside_polygon_corner(int64_t ax, int64_t ay, int64_t bx, int64_t by,
                                          int64_t cx, int64_t cy, int64_t qx, int64_t qy) {
    return Slic3r::Geometry::is_point_inside_polygon_corner(
        Slic3r::Point(ax, ay), Slic3r::Point(bx, by), Slic3r::Point(cx, cy), Slic3r::Point(qx, qy));
}

} // namespace geometry
} // namespace proof_units
} // namespace slic3r_ffi
