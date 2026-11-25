#pragma once

// Declarations only, same pattern as cxx/clipper2_shim.h: cxx_build generates the struct
// definitions (CPoint3, CHullStats) and rust::Vec at slic3r-ffi/src/qhull.rs.h on the include
// path it sets up.
#include "slic3r-ffi/src/qhull.rs.h"

namespace slic3r_ffi {
namespace qhull {

CHullStats qhull_convex_hull_stats(const rust::Vec<CPoint3> &points);

} // namespace qhull
} // namespace slic3r_ffi
