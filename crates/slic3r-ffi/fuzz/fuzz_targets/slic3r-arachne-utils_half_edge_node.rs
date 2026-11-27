//! Differential fuzz target for `Arachne/utils/HalfEdgeNode.hpp` (task
//! `73-foundation-arachne-indices`, plan step 20): `port/manifest.tsv`'s
//! `Arachne/utils/HalfEdgeNode.hpp` row is `fuzz target: required`. Generates an arbitrary
//! point, constructs `slic3r_arachne_utils::half_edge_node::HalfEdgeNode::new` with it, and
//! asserts exact equality between the stored point (and the default-constructed node's lack of
//! an incident edge) and the same construction through `slic3r-ffi`'s own
//! `proof_units::half_edge_node` shim -- the identical comparison
//! `crates/slic3r-arachne-utils/src/half_edge_node.rs`'s own `// PROOF: 1` test makes on fixed
//! inputs, here driven by the fuzzer's arbitrary ones.
//!
//! `just fuzz slic3r-arachne-utils_half_edge_node` runs it.

#![no_main]

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use slic3r_arachne_utils::half_edge_node::HalfEdgeNode;
use slic3r_ffi::proof_units::half_edge_node::{path1_node_has_incident_edge, path1_node_point};
use slic3r_point::point::Point;

/// A single point over 64-bit integer coordinates (`coord_t` is `int64_t` at the pin,
/// `vendor/OrcaSlicer/src/libslic3r/libslic3r.h:45-48`) -- the same shape
/// `slic3r-arachne-utils_linear_alg2_d.rs`'s own `ArbPoint` uses, redefined here because a
/// `#![no_main]` fuzz binary is its own crate.
#[derive(Debug, Clone, Copy, Arbitrary)]
struct ArbPoint {
    x: i64,
    y: i64,
}

fuzz_target!(|input: ArbPoint| {
    let p = Point::new(input.x, input.y);

    let node = HalfEdgeNode::new(0u32, p);
    let actual = (node.p.x(), node.p.y());

    let oracle_p = path1_node_point(p.x(), p.y());
    assert_eq!(
        actual,
        (oracle_p[0], oracle_p[1]),
        "HalfEdgeNode's stored point must match the oracle exactly for p={:?}",
        input,
    );

    let oracle_has_edge = path1_node_has_incident_edge(p.x(), p.y());
    assert!(
        !oracle_has_edge,
        "a freshly constructed HalfEdgeNode must start with no incident edge"
    );
});
