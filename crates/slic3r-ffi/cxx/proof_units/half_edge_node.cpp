// Proof shims for the `HalfEdgeNode.hpp` unit (R19, proof path 1, task
// `73-foundation-arachne-indices`), compiled onto `crates/slic3r-ffi`'s shared
// `cxx_build::Build` -- the one sanctioned route a ported unit reaches the oracle by
// (`docs/PORTING.md`, "Decisions recorded on 2026-11-25").
//
// `HalfEdgeNode<node_data_t, edge_data_t, derived_node_t, derived_edge_t>` (`:17-32`) is a
// four-parameter template with no in-scope instantiation yet (`HalfEdge`, `HalfEdgeGraph.hpp`
// and the two `SkeletalTrapezoidation*` files that use it are each a fan-out module, not this
// task's). `TestNode` below instantiates it over a plain `int64_t` for every parameter --
// `node_data_t` is read back below, `edge_data_t`/`derived_node_t`/`derived_edge_t` are each
// unused inside the class body except as `edge_t`/`node_t`'s own aliases (see
// `half_edge_node.rs`'s own module doc), so any FFI-plain type satisfies them. This proof is
// therefore necessarily thin -- it exercises the constructor and the field layout the vendored
// class actually has, not a caller's use of it, because this unit has no in-scope caller yet.
#include "half_edge_node.h"

#include "Arachne/utils/HalfEdgeNode.hpp"
#include "Point.hpp"

namespace slic3r_ffi {
namespace proof_units {
namespace half_edge_node {

using TestNode = Slic3r::Arachne::HalfEdgeNode<int64_t, int64_t, int64_t, int64_t>;

rust::Vec<int64_t> path1_node_point(int64_t px, int64_t py) {
    TestNode node(0, Slic3r::Point(px, py));
    rust::Vec<int64_t> out;
    out.push_back(static_cast<int64_t>(node.p.x()));
    out.push_back(static_cast<int64_t>(node.p.y()));
    return out;
}

bool path1_node_has_incident_edge(int64_t px, int64_t py) {
    TestNode node(0, Slic3r::Point(px, py));
    return node.incident_edge != nullptr;
}

} // namespace half_edge_node
} // namespace proof_units
} // namespace slic3r_ffi
