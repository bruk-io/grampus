#pragma once

// Declarations only, same pattern as cxx/clipper2_shim.h: cxx_build generates the struct
// definitions (CVertex, CFacet) and rust::Vec at slic3r-ffi/src/admesh.rs.h on the include
// path it sets up.
#include "slic3r-ffi/src/admesh.rs.h"

namespace slic3r_ffi {
namespace admesh {

float admesh_compute_volume(const rust::Vec<CFacet> &facets);

} // namespace admesh
} // namespace slic3r_ffi
