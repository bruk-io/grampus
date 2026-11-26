#pragma once

// Declarations only (see cxx/clipper2_shim.h's comment for why this crate always splits the
// cxx-generated header from the hand-written declarations file). `cxx_build::bridges` generates
// this header's counterpart at <crate-name>/src/proof_units/half_edge_node.rs.h on the include
// path it sets up, and `cxx/proof_units/half_edge_node.cpp` is the one translation unit that
// defines the functions below.
#include "slic3r-ffi/src/proof_units/half_edge_node.rs.h"

namespace slic3r_ffi {
namespace proof_units {
namespace half_edge_node {

rust::Vec<int64_t> path1_node_point(int64_t px, int64_t py);
bool path1_node_has_incident_edge(int64_t px, int64_t py);

} // namespace half_edge_node
} // namespace proof_units
} // namespace slic3r_ffi
