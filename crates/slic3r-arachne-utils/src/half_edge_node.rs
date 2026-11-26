//! Port of `Arachne/utils/HalfEdgeNode.hpp` (task `73-foundation-arachne-indices`, plan step 20,
//! `foundation-22` row of `port/foundation.tsv`): the half-edge mesh's node type, one of the
//! three units this task lands `ported` (the fourth, `polyline_stitcher`, lands `drafted` --
//! see that module's own doc).
//!
//! **The four template parameters collapse to one.** The vendored class is
//! `template<typename node_data_t, typename edge_data_t, typename derived_node_t, typename
//! derived_edge_t> class HalfEdgeNode` (`:17-32`), but only two of the four are used inside the
//! class body at all: `node_data_t` (the `data` field) and `derived_node_t` (aliased `node_t`,
//! used only as `operator==`'s parameter type). `edge_data_t` is declared and never referenced
//! inside this class; `derived_edge_t` is aliased `edge_t` and used only for `incident_edge`'s
//! declared type, `edge_t*` -- which `port/ownership.tsv`'s `back-reference` row for this exact
//! field (`HalfEdgeNode.hpp:25`) already resolves to a plain `usize` index, with no `edge_t`
//! type needed at all to spell it. So after taking that cell verbatim, this struct needs only
//! one generic parameter, `NodeData`, for the field the vendored template actually stores a
//! caller-chosen type in. **Finding, named in this task's pull request**: dropping
//! `edge_data_t`/`derived_edge_t`/`derived_node_t` from the Rust signature is a judgement call
//! about a template with no in-scope instantiation yet (`HalfEdge`, `HalfEdgeGraph.hpp`,
//! `SkeletalTrapezoidationGraph.hpp` and `SkeletalTrapezoidation.hpp` -- `port/ownership.tsv`'s
//! own evidence column for this field -- are each a fan-out module, not this task's to write),
//! so there is no concrete caller yet to monomorphise against and check this choice against.
//!
//! **`operator==` is identity, not a field comparison, and has no `PartialEq` counterpart
//! here.** `bool operator==(const node_t& other) { return this == &other; }` (`:27-30`)
//! compares `this` (a `HalfEdgeNode*`, implicitly upcast from `&other`'s derived-type address)
//! against `&other` -- are the two the same object in memory, not whether their fields agree.
//! A `#[derive(PartialEq)]` here would silently do the opposite (compare `data`, `p` and
//! `incident_edge` by value), so none is derived; [`HalfEdgeNode::is_same_node`] below is the
//! named, pointer-identity method instead, taking `&Self` rather than the vendored `node_t`
//! (CRTP's derived-type parameter, which this port's one-type-param struct has collapsed away --
//! see above) since this port has no second, derived type for it to distinguish.
//!
//! **`incident_edge`'s default has no sentinel, and this is a gap this task reports rather than
//! invents an answer for.** `port/ownership.tsv`'s `back-reference` row for `:25` gives the
//! field's Rust type as `usize` verbatim (an index into the owning `HalfEdgeGraph`'s edge
//! collection), not `Option<usize>` -- unlike `## Decisions recorded on 2026-11-26`, decision 6
//! (an `owned` field the C++ can null), decision 6 is scoped to the `owned` class only and this
//! row's class is `back-reference`, so that decision does not reach it and this module invents
//! no `Option` wrapper the committed cell does not call for (`docs/PORTING.md`: "no module
//! invents a Rust type for a row whose cell is empty", and this cell is not empty -- it is
//! `usize`, taken verbatim). But `edge_t* incident_edge = nullptr;`'s default member initializer
//! *is* a real null, meaningfully distinct in the owning graph from any real edge index (every
//! node starts with none), and `0` is itself a valid edge index once one exists -- there is no
//! value [`HalfEdgeNode::new`] can give this field that is unambiguously "no edge yet" under a
//! plain `usize`. [`HalfEdgeNode::new`] below sets it to `0` as a placeholder with no claim that
//! `0` means "none"; the real sentinel convention is the owning `HalfEdgeGraph` fan-out module's
//! to establish, the same way `#60` (`docs/PORTING.md` decision 6) reported rather than invented
//! when it hit the equivalent gap for an `owned` field.
//!
//! **The forward-declared sibling `HalfEdge` (`:14-15`) has no body in this header** -- a
//! forward declaration a fan-out module eventually defines, not a type this hand-design task
//! writes a Rust counterpart for.

use slic3r_point::point::Point;

/// `template<...> class HalfEdgeNode` (`:17-32`) -- see the module doc for why this is one
/// generic parameter rather than the vendored four.
pub struct HalfEdgeNode<NodeData> {
    /// `node_data_t data;` (`:20`).
    pub data: NodeData,
    /// `Point p;` (`:21`).
    pub p: Point,
    /// `edge_t* incident_edge = nullptr;` (`:22`) -- see the module doc for why this is a plain
    /// `usize` with no `Option` wrapper, and for the unresolved "no edge yet" sentinel gap.
    pub incident_edge: usize,
}

impl<NodeData> HalfEdgeNode<NodeData> {
    /// `HalfEdgeNode(node_data_t data, Point p) : data(data), p(p) {}` (`:23-26`) --
    /// `incident_edge` is left at its in-class default, `nullptr`; see the module doc for why
    /// `0` here is a placeholder, not a sentinel this module defines the meaning of.
    pub fn new(data: NodeData, p: Point) -> Self {
        HalfEdgeNode {
            data,
            p,
            incident_edge: 0,
        }
    }

    /// `bool operator==(const node_t& other) { return this == &other; }` (`:27-30`) -- pointer
    /// identity of the node objects themselves, not a field comparison; see the module doc for
    /// why this is a named method rather than a `PartialEq` impl.
    pub fn is_same_node(&self, other: &Self) -> bool {
        std::ptr::eq(self, other)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_same_node_is_identity_not_value() {
        let a = HalfEdgeNode::new(1u32, Point::new(10, 20));
        let b = HalfEdgeNode::new(1u32, Point::new(10, 20));
        // Equal fields, different objects: `this == &other` is false for two distinct nodes
        // even when their data agrees, exactly the divergence from a value comparison the
        // module doc names.
        assert!(!a.is_same_node(&b));
        assert!(a.is_same_node(&a));
    }

    /// Proof path 1 (R19, manifest row `Arachne/utils/HalfEdgeNode.hpp`): the shim instantiates
    /// the vendored template over a concrete, FFI-plain node/edge data pair and reads back the
    /// `p` field and the default `incident_edge`'s nullness through the pointer-identity
    /// comparison `is_same_node` stands in for -- see `cxx/proof_units/half_edge_node.cpp` for
    /// what is actually instantiated and why the proof is necessarily thin (a template with no
    /// in-scope fan-out caller yet has no behaviour beyond its own constructor and field
    /// layout). One cell across this unit's single manifest row, so one `// PROOF: 1` marker.
    // PROOF: 1
    #[test]
    fn half_edge_node_matches_the_oracle() {
        let p = Point::new(123, -456);
        let actual_p = HalfEdgeNode::new(0u32, p).p;

        let oracle_p = slic3r_proof::proof_units::half_edge_node::path1_node_point(p.x(), p.y());
        assert_eq!(
            (actual_p.x(), actual_p.y()),
            (oracle_p[0], oracle_p[1]),
            "HalfEdgeNode's stored point must match the oracle exactly"
        );

        // `edge_t* incident_edge = nullptr;` (`:22`): the oracle's default-constructed node
        // reports no incident edge, mirrored here by this port's own placeholder `0` carrying no
        // claim of meaning -- see the module doc. Proven as "the oracle also starts with none",
        // not as "0 means none" on this port's side.
        let oracle_has_edge =
            slic3r_proof::proof_units::half_edge_node::path1_node_has_incident_edge(p.x(), p.y());
        assert!(
            !oracle_has_edge,
            "a freshly constructed HalfEdgeNode must start with no incident edge"
        );
    }
}

// PORT STATUS
// source: src/libslic3r/Arachne/utils/HalfEdgeNode.hpp
// confidence: low
// tier: exact
// oracle: built
