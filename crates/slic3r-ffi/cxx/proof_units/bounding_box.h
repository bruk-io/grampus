#pragma once

// Declarations only (see cxx/clipper2_shim.h's comment for why this crate always splits the
// cxx-generated header from the hand-written declarations file). `cxx_build::bridges` generates
// this header's counterpart at <crate-name>/src/proof_units/bounding_box.rs.h on the include path
// it sets up, and `cxx/proof_units/bounding_box.cpp` is the one translation unit that defines the
// functions below.
#include "slic3r-ffi/src/proof_units/bounding_box.rs.h"

namespace slic3r_ffi {
namespace proof_units {
namespace bounding_box {

// Path 1: `BoundingBox.cpp`'s own, non-template methods -- externally linked symbols in the
// archive `just oracle-build` produces.
rust::Vec<int64_t> path1_rotated(int64_t min_x, int64_t min_y, int64_t max_x, int64_t max_y,
                                  double angle);
rust::Vec<int64_t> path1_rotated_about(int64_t min_x, int64_t min_y, int64_t max_x, int64_t max_y,
                                        int64_t cx, int64_t cy, double angle);
rust::Vec<int64_t> path1_scaled(int64_t min_x, int64_t min_y, int64_t max_x, int64_t max_y,
                                 bool defined, double factor);
rust::Vec<int64_t> path1_align_to_grid(int64_t min_x, int64_t min_y, bool defined,
                                        int64_t cell_size);

// Path 2: `BoundingBox.hpp`'s own header-only `BoundingBoxBase<Point, Points>` surface, driven
// directly through a real vendored `Slic3r::BoundingBox` instantiated in this translation unit.
rust::Vec<int64_t> path2_merge_point(int64_t min_x, int64_t min_y, int64_t max_x, int64_t max_y,
                                      bool defined, int64_t px, int64_t py);
rust::Vec<int64_t> path2_merge_points(rust::Slice<const int64_t> xs,
                                       rust::Slice<const int64_t> ys);
rust::Vec<int64_t> path2_merge_bb(int64_t min_x, int64_t min_y, int64_t max_x, int64_t max_y,
                                   bool defined, int64_t omin_x, int64_t omin_y, int64_t omax_x,
                                   int64_t omax_y, bool odefined);
rust::Vec<int64_t> path2_scale(int64_t min_x, int64_t min_y, int64_t max_x, int64_t max_y,
                                double factor);
rust::Vec<int64_t> path2_size(int64_t min_x, int64_t min_y, int64_t max_x, int64_t max_y);
double path2_radius(int64_t min_x, int64_t min_y, int64_t max_x, int64_t max_y);
double path2_area(int64_t min_x, int64_t min_y, int64_t max_x, int64_t max_y);
rust::Vec<int64_t> path2_center(int64_t min_x, int64_t min_y, int64_t max_x, int64_t max_y);
rust::Vec<int64_t> path2_translate_xy(int64_t min_x, int64_t min_y, int64_t max_x, int64_t max_y,
                                       double tx, double ty);
rust::Vec<int64_t> path2_translate_point(int64_t min_x, int64_t min_y, int64_t max_x,
                                          int64_t max_y, int64_t vx, int64_t vy);
rust::Vec<int64_t> path2_offset(int64_t min_x, int64_t min_y, int64_t max_x, int64_t max_y,
                                 double delta);
bool path2_contains_point(int64_t min_x, int64_t min_y, int64_t max_x, int64_t max_y, int64_t px,
                           int64_t py);
bool path2_contains_bbox(int64_t min_x, int64_t min_y, int64_t max_x, int64_t max_y,
                          int64_t omin_x, int64_t omin_y, int64_t omax_x, int64_t omax_y);
bool path2_overlap(int64_t min_x, int64_t min_y, int64_t max_x, int64_t max_y, int64_t omin_x,
                    int64_t omin_y, int64_t omax_x, int64_t omax_y);
rust::Vec<int64_t> path2_index(int64_t min_x, int64_t min_y, int64_t max_x, int64_t max_y,
                                int32_t idx);
bool path2_eq(int64_t min_x, int64_t min_y, int64_t max_x, int64_t max_y, int64_t omin_x,
              int64_t omin_y, int64_t omax_x, int64_t omax_y);

} // namespace bounding_box
} // namespace proof_units
} // namespace slic3r_ffi
