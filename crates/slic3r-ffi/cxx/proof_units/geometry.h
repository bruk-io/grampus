#pragma once

// Declarations only (see cxx/clipper2_shim.h's comment for why this crate always splits the
// cxx-generated header from the hand-written declarations file). `cxx_build::bridges` generates
// this header's counterpart at <crate-name>/src/proof_units/geometry.rs.h on the include path
// it sets up, and `cxx/proof_units/geometry.cpp` is the one translation unit that defines the
// functions below.
//
// Every function below calls `Slic3r::Geometry::` code the shim reaches through this file's own
// `#include "Geometry.hpp"` (via `geometry.cpp`), measured (`nm -g` on the oracle's own
// `Geometry.cpp.o`) into three groups, none of which needs `#include`-ing `Geometry.cpp` whole:
//   - `orient`/`is_ccw` are `static inline` in the header -- true internal linkage, so this file
//     compiling `Geometry.hpp` fresh IS proof path 3's mechanism for these two.
//   - `ray_ray_intersection`/`segment_segment_intersection`/`segments_intersect`/
//     `is_rotation_ninety_degrees`/`deg2rad`/`angle_to_0_2PI`/`to_range_pi_pi` are plain `inline`
//     or template header functions with no symbol in `liblibslic3r.a` (nothing else in this
//     build's object files instantiates/ODR-uses them) -- also reached only by recompiling the
//     header here, the same mechanism in substance.
//   - `directions_parallel`/`directions_perpendicular`/`rad2deg_dir`/`linint`/`arrange`/
//     `is_point_inside_polygon_corner` resolve to a real `T` symbol in `liblibslic3r.a`, the same
//     mechanism proof path 1 uses.
// `port/manifest.tsv`'s `Geometry.cpp`/`Geometry.hpp` rows both carry proof path `3`, but this
// unit has no in-scope, provable internal-linkage helper that needs the *whole* `.cpp`
// `#include`d (`Geometry.cpp`'s own four `static` helpers are all `Transform3d`-based and belong
// to this unit's deferred surface): see `crates/slic3r-geometry/src/geometry.rs`'s `// PROOF: 3`
// test for the full account, and this task's pull request body for the decision. Nothing here
// `#include`s `Geometry.cpp`, so none of `cxx/proof_units/clipper_z_utils.cpp`'s duplicate-symbol
// analysis applies to this file.
#include "slic3r-ffi/src/proof_units/geometry.rs.h"

namespace slic3r_ffi {
namespace proof_units {
namespace geometry {

int32_t path_orient(int64_t ax, int64_t ay, int64_t bx, int64_t by, int64_t cx, int64_t cy);
bool path_is_ccw(rust::Slice<const int64_t> xs, rust::Slice<const int64_t> ys);
bool path_segments_intersect(int64_t ip1x, int64_t ip1y, int64_t ip2x, int64_t ip2y, int64_t jp1x,
                              int64_t jp1y, int64_t jp2x, int64_t jp2y);
// Returns `[ok (0/1), x, y]` -- the C++ `bool` return and the `Vec2d&` out-parameter flattened
// into one vector, the same shape `path_arrange` below uses for its own out-parameter.
rust::Vec<double> path_ray_ray_intersection(double p1x, double p1y, double v1x, double v1y,
                                             double p2x, double p2y, double v2x, double v2y);
rust::Vec<double> path_segment_segment_intersection(double p1x, double p1y, double v1x,
                                                     double v1y, double p2x, double p2y,
                                                     double v2x, double v2y);
bool path_directions_parallel(double angle1, double angle2, double max_diff);
bool path_directions_perpendicular(double angle1, double angle2, double max_diff);
double path_rad2deg_dir(double angle);
double path_deg2rad(double angle);
double path_angle_to_0_2pi(double angle);
double path_to_range_pi_pi(double angle);
double path_linint(double value, double oldmin, double oldmax, double newmin, double newmax);
// Returns `[ok (0/1), x0, y0, x1, y1, ...]`. `bb_defined` selects whether the bed bounding box
// argument is passed at all -- `cxx` has no `Option<&T>` across the bridge.
rust::Vec<double> path_arrange(size_t total_parts, double part_w, double part_h, double dist,
                                bool bb_defined, double bb_min_x, double bb_min_y,
                                double bb_max_x, double bb_max_y);
bool path_is_rotation_ninety_degrees(double a);
bool path_is_point_inside_polygon_corner(int64_t ax, int64_t ay, int64_t bx, int64_t by,
                                          int64_t cx, int64_t cy, int64_t qx, int64_t qy);

} // namespace geometry
} // namespace proof_units
} // namespace slic3r_ffi
