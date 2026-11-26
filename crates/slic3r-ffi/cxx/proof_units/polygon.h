#pragma once

// Declarations only (see cxx/clipper2_shim.h's comment for why this crate always splits the
// cxx-generated header from the hand-written declarations file). `cxx_build::bridges` generates
// this header's counterpart at <crate-name>/src/proof_units/polygon.rs.h on the include path it
// sets up, and `cxx/proof_units/polygon.cpp` is the one translation unit that defines the
// functions below.
#include "slic3r-ffi/src/proof_units/polygon.rs.h"

namespace slic3r_ffi {
namespace proof_units {
namespace polygon {

// Every function here is driven through a real `Slic3r::Polygon`, built from this `#include
// <Polygon.cpp>` translation unit's own copy of the class -- the manifest's `proof path: 3` for
// both of this unit's rows, not a per-method choice: `point_in_polygon` (`:738-766`) is the one
// symbol with no external linkage to call any other way, and every other function below shares
// this same translation unit rather than opening a second, ordinary header-include shim, so the
// unit's whole surface is proven through the one route its manifest row names.
int32_t path3_point_in_polygon(int64_t px, int64_t py, rust::Slice<const int64_t> xs,
                                rust::Slice<const int64_t> ys);
double path3_area(rust::Slice<const int64_t> xs, rust::Slice<const int64_t> ys);
bool path3_is_counter_clockwise(rust::Slice<const int64_t> xs, rust::Slice<const int64_t> ys);
rust::Vec<int64_t> path3_centroid(rust::Slice<const int64_t> xs, rust::Slice<const int64_t> ys);
bool path3_contains(rust::Slice<const int64_t> xs, rust::Slice<const int64_t> ys, int64_t px,
                     int64_t py);
rust::Vec<int64_t> path3_point_projection(rust::Slice<const int64_t> xs,
                                           rust::Slice<const int64_t> ys, int64_t px, int64_t py);
bool path3_polygon_is_convex(rust::Slice<const int64_t> xs, rust::Slice<const int64_t> ys);
rust::Vec<int64_t> path3_make_circle_num_segments(double radius, uint64_t num_segments);
rust::Vec<int64_t> path3_remove_sticks(rust::Slice<const int64_t> xs,
                                        rust::Slice<const int64_t> ys);
rust::Vec<float> path3_parameter_by_length(rust::Slice<const int64_t> xs,
                                            rust::Slice<const int64_t> ys);
rust::Vec<int64_t> path3_densify(rust::Slice<const int64_t> xs, rust::Slice<const int64_t> ys,
                                  float min_length);
rust::Vec<int64_t> path3_intersection(rust::Slice<const int64_t> xs, rust::Slice<const int64_t> ys,
                                       int64_t ax, int64_t ay, int64_t bx, int64_t by);
rust::Vec<int64_t> path3_first_intersection(rust::Slice<const int64_t> xs,
                                             rust::Slice<const int64_t> ys, int64_t ax,
                                             int64_t ay, int64_t bx, int64_t by);
rust::Vec<int64_t> path3_intersections(rust::Slice<const int64_t> xs,
                                        rust::Slice<const int64_t> ys, int64_t ax, int64_t ay,
                                        int64_t bx, int64_t by);
rust::Vec<int64_t> path3_douglas_peucker(rust::Slice<const int64_t> xs,
                                          rust::Slice<const int64_t> ys, double tolerance);

} // namespace polygon
} // namespace proof_units
} // namespace slic3r_ffi
