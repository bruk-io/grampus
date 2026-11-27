#pragma once

// Declarations only (see cxx/clipper2_shim.h's comment for why this crate always splits the
// cxx-generated header from the hand-written declarations file). `cxx_build::bridges` generates
// this header's counterpart at <crate-name>/src/proof_units/geometry.rs.h on the include path
// it sets up, and `cxx/proof_units/geometry.cpp` is the one translation unit that defines the
// functions below.
//
// Every function below calls a plain, externally-linked `Slic3r::Geometry::` free function (or
// the header-only inline `orient`/`is_ccw`/`segments_intersect`) directly against
// `liblibslic3r.a` -- the same mechanism proof path 1 uses. `port/manifest.tsv`'s `Geometry.cpp`/
// `Geometry.hpp` rows both carry proof path `3`, but this unit has no in-scope, provable
// internal-linkage helper to reach that way: see `crates/slic3r-geometry/src/geometry.rs`'s
// `// PROOF: 3` test for why, and this task's pull request body for the decision. Nothing here
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
bool path_directions_parallel(double angle1, double angle2, double max_diff);
bool path_directions_perpendicular(double angle1, double angle2, double max_diff);
double path_rad2deg_dir(double angle);
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
