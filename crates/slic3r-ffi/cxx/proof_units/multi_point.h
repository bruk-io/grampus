#pragma once

// Declarations only (see cxx/clipper2_shim.h's comment for why this crate always splits the
// cxx-generated header from the hand-written declarations file). `cxx_build::bridges` generates
// this header's counterpart at <crate-name>/src/proof_units/multi_point.rs.h on the include path
// it sets up, and `cxx/proof_units/multi_point.cpp` is the one translation unit that defines the
// functions below.
//
// Every function drives `Slic3r::MultiPoint`'s own concrete methods (`MultiPoint.cpp`, proof
// path 1) or its header-only inline methods and free functions (`MultiPoint.hpp`, proof path 2)
// through a real `Slic3r::Polygon` -- the one concrete, instantiable subclass this task's own
// units provide (`MultiPoint` itself has no public constructor, being abstract). `xs`/`ys` are
// parallel slices building the polygon's own `points` (`cxx` has no `Vec<(i64, i64)>`, the same
// convention `cxx/proof_units/bounding_box.cpp`'s `path2_merge_points` uses).
#include "slic3r-ffi/src/proof_units/multi_point.rs.h"

namespace slic3r_ffi {
namespace proof_units {
namespace multi_point {

// Path 1: MultiPoint.cpp's own, externally linked methods.
rust::Vec<int64_t> path1_scale(rust::Slice<const int64_t> xs, rust::Slice<const int64_t> ys,
                                double factor);
rust::Vec<int64_t> path1_translate(rust::Slice<const int64_t> xs, rust::Slice<const int64_t> ys,
                                    int64_t vx, int64_t vy);
rust::Vec<int64_t> path1_rotate_cos_sin(rust::Slice<const int64_t> xs,
                                        rust::Slice<const int64_t> ys, double cos_angle,
                                        double sin_angle);
double path1_length(rust::Slice<const int64_t> xs, rust::Slice<const int64_t> ys);
int32_t path1_find_point(rust::Slice<const int64_t> xs, rust::Slice<const int64_t> ys,
                          int64_t px, int64_t py);
rust::Vec<int64_t> path1_bounding_box(rust::Slice<const int64_t> xs,
                                       rust::Slice<const int64_t> ys);
rust::Vec<int64_t> path1_intersection(rust::Slice<const int64_t> xs,
                                       rust::Slice<const int64_t> ys, int64_t ax, int64_t ay,
                                       int64_t bx, int64_t by);
rust::Vec<int64_t> path1_symmetric_y(rust::Slice<const int64_t> xs,
                                      rust::Slice<const int64_t> ys, int64_t x_axis);
// `MultiPoint::concave_hull_2d(const Points&, double)` (`:364-381`), a `static` method called
// directly -- no instance needed, `MultiPoint` need not be instantiated through `Polygon` here.
rust::Vec<int64_t> path1_concave_hull_2d(rust::Slice<const int64_t> xs,
                                          rust::Slice<const int64_t> ys, double tolerance);
// `MultiPoint::minimumDistanceBetweenLinesDefinedByPoints(const Points&, const Points&)`
// (`:414-434`), which is the only external caller of the `private` `squaredDistanceToLineSegment`
// (`:393-404`) -- proving this proves that helper too.
double path1_min_distance_between_lines(rust::Slice<const int64_t> a_xs,
                                         rust::Slice<const int64_t> a_ys,
                                         rust::Slice<const int64_t> b_xs,
                                         rust::Slice<const int64_t> b_ys);
// `get_extents_rotated(const Points&, double)` (`:550-574`), returning
// `[min.x, min.y, max.x, max.y, defined (0/1)]`.
rust::Vec<int64_t> path1_get_extents_rotated(rust::Slice<const int64_t> xs,
                                              rust::Slice<const int64_t> ys, double angle);

// Path 2: MultiPoint.hpp's own header-only inline methods and free functions, driven through the
// same real `Slic3r::Polygon`.
rust::Vec<int64_t> path2_front_back(rust::Slice<const int64_t> xs, rust::Slice<const int64_t> ys);
rust::Vec<int64_t> path2_closest_point(rust::Slice<const int64_t> xs,
                                        rust::Slice<const int64_t> ys, int64_t px, int64_t py);
double path2_distance_to(rust::Slice<const int64_t> xs, rust::Slice<const int64_t> ys, int64_t px,
                          int64_t py);
bool path2_is_valid(rust::Slice<const int64_t> xs, rust::Slice<const int64_t> ys);
double path2_length_free(rust::Slice<const int64_t> xs, rust::Slice<const int64_t> ys);
double path2_area_free(rust::Slice<const int64_t> xs, rust::Slice<const int64_t> ys);

} // namespace multi_point
} // namespace proof_units
} // namespace slic3r_ffi
