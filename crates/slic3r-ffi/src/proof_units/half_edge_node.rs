//! Proof path 1 (R19, task `73-foundation-arachne-indices`) for the `HalfEdgeNode.hpp` unit.
//! See `cxx/proof_units/half_edge_node.cpp` for why this proof is necessarily thin: the
//! vendored class has no in-scope caller yet, so the shim instantiates it over a plain
//! `int64_t` stand-in and reads back only the constructor's own field layout.

#[cxx::bridge(namespace = "slic3r_ffi::proof_units::half_edge_node")]
mod ffi {
    // SAFETY: this is cxx's own unsafe-extern declaration block, not a hand-written unsafe
    // expression (see src/clipper2.rs's identical note) -- it carries no body, and the only
    // unsafety is in the bindings `cxx` generates from it.
    unsafe extern "C++" {
        include!("cxx/proof_units/half_edge_node.h");

        /// `HalfEdgeNode(0, Slic3r::Point(px, py)).p`, returning `[x, y]`.
        fn path1_node_point(px: i64, py: i64) -> Vec<i64>;
        /// `HalfEdgeNode(0, Slic3r::Point(px, py)).incident_edge != nullptr`.
        fn path1_node_has_incident_edge(px: i64, py: i64) -> bool;
    }
}

pub use ffi::{path1_node_has_incident_edge, path1_node_point};
