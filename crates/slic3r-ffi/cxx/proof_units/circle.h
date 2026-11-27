#pragma once

#include "slic3r-ffi/src/proof_units/circle.rs.h"

namespace slic3r_ffi {
namespace proof_units {
namespace circle {

// Path 1 (task 76-foundation-voronoi-curves): `Geometry/Circle.cpp`'s own, non-template
// functions -- externally linked symbols in the archive `just oracle-build` produces.
// `flat` is `[x0, y0, x1, y1, ...]`.
rust::Vec<double> path1_circle_center_taubin_newton(rust::Vec<double> flat, uint64_t cycles);
rust::Vec<double> path1_circle_taubin_newton(rust::Vec<double> flat, uint64_t cycles);

// Path 2: `Geometry/Circle.hpp`'s own header-only templates, instantiated directly in this
// translation unit's counterpart `.cpp`.
rust::Vec<double> path2_circle_center(double ax, double ay, double bx, double by, double cx,
                                       double cy, double epsilon);
// `flat` is `[x0, y0, x1, y1, ...]`, scaled (integer) coordinates.
rust::Vec<double> path2_smallest_enclosing_circle_welzl(rust::Vec<int64_t> flat);
rust::Vec<double> path2_ray_circle_intersections(double r2, double a, double b, double lv2,
                                                  double c);

} // namespace circle
} // namespace proof_units
} // namespace slic3r_ffi
