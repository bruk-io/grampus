// Proof shims for the `HalfEdgeNode.hpp` unit (R19, proof path 1, task
// `73-foundation-arachne-indices`), compiled onto `crates/slic3r-ffi`'s shared
// `cxx_build::Build` -- the one sanctioned route a ported unit reaches the oracle by
// (`docs/PORTING.md`, "Decisions recorded on 2026-11-25").
//
// `HalfEdgeNode<node_data_t, edge_data_t, derived_node_t, derived_edge_t>` (`:17-35`) has one
// concrete in-scope instantiation, `SkeletalTrapezoidationGraph.hpp:59`'s `STHalfEdgeNode :
// public HalfEdgeNode<SkeletalTrapezoidationJoint, SkeletalTrapezoidationEdge, STHalfEdgeNode,
// STHalfEdge>` -- but that type belongs to a fan-out module this task does not write (see
// `half_edge_node.rs`'s own module doc), so `TestNode` below instead instantiates the template
// over a plain `int64_t` for every parameter. `edge_data_t`/`derived_node_t`/`derived_edge_t`
// are each unused inside the class body except as `edge_t`/`node_t`'s own aliases, so any
// FFI-plain type satisfies them; `node_data_t` is passed to the constructor but never read back
// by this shim (`path1_node_point` and `path1_node_has_incident_edge` below read `p` and
// `incident_edge`, never `data`). This proof is therefore necessarily thin -- it exercises the
// constructor and the field layout the vendored class actually has, and cannot exercise
// `operator==`'s pointer identity at all (`node_t = int64_t` here, so `this == &other` would
// compare against an unrelated scalar type and is never instantiated), not a caller's use of
// the class, because the instantiation that exists is a fan-out module's, not this task's.
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
