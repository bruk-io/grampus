#pragma once

// Declarations only, same pattern as cxx/clipper2_shim.h: cxx_build generates the struct
// definitions (CPoint, CEdgeCacheStats) and rust::Vec at slic3r-ffi/src/nest2d.rs.h on the
// include path it sets up.
//
// This bridge's own namespace is `slic3r_ffi::nest2d`, not `slic3r_ffi::libnest2d`: the vendored
// library's own top-level namespace is `::libnest2d` (`libnest2d.hpp`), and nesting an
// identically-named namespace inside `slic3r_ffi` would make every unqualified
// `libnest2d::Something` inside this bridge's .cc resolve to itself first (name lookup finds the
// innermost enclosing `libnest2d` before the global one), not to the vendored library --
// `nest2d` sidesteps that without needing `::libnest2d::` qualification everywhere.
#include "slic3r-ffi/src/nest2d.rs.h"

namespace slic3r_ffi {
namespace nest2d {

CEdgeCacheStats libnest2d_edge_cache_stats(const rust::Vec<CPoint> &contour);

} // namespace nest2d
} // namespace slic3r_ffi
