#pragma once

// Declarations only, same pattern as cxx/clipper2_shim.h: cxx_build generates the struct
// definitions (CSegment, CVoronoiStats) and rust::Vec at slic3r-ffi/src/voronoi.rs.h on the
// include path it sets up.
#include "slic3r-ffi/src/voronoi.rs.h"

namespace slic3r_ffi {
namespace voronoi {

CVoronoiStats voronoi_diagram_stats(const rust::Vec<CSegment> &segments);

} // namespace voronoi
} // namespace slic3r_ffi
