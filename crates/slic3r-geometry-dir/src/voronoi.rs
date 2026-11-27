//! Port of `src/libslic3r/Geometry/Voronoi.cpp`/`Voronoi.hpp` (task `76-foundation-voronoi-curves`,
//! plan step 20, `port/foundation.tsv`'s `foundation-25` row).
//!
//! // PORT NOTE: this module is an honest Phase A draft, not a ported module, and is the one
//! stated exception among this step's 33 tasks (`intent/libslic3r-rust-port/tasks/
//! 76-foundation-voronoi-curves/task.md`, "The mutual reference with `foundation-26`"): no
//! `mod voronoi;` line reaches it, it carries no `// PROOF:` marker and no fuzz target here,
//! although both its manifest rows say `fuzz target: required`. `Geometry/VoronoiUtils.hpp`
//! (`crate::voronoi_utils`, task `77-foundation-voronoi-offset`) and this file reference each
//! other at the pin -- `VoronoiUtils.hpp:8,20,21,35,36,47,49,51,53,60` names this module's
//! `VoronoiDiagram::edge_type`/`vertex_type`/`cell_type`, and this file's own
//! `VoronoiDiagram::detect_known_voronoi_cell_issues`/`detect_known_voronoi_edge_issues`/
//! `try_to_repair_degenerated_voronoi_diagram_by_rotation` call back into `VoronoiUtils` -- and
//! the crate cut keeps both in `slic3r-geometry-dir`, so this is the one unit of all 33
//! foundation tasks that cannot be ported standalone. `77-foundation-voronoi-offset` adds
//! `voronoi_utils.rs`, the `mod voronoi;` line, this module's `// PROOF: 1` marker and its
//! `required` fuzz target. The five call sites into `VoronoiUtils` below are spelled
//! `crate::voronoi_utils::...`, as the port will spell them once that module exists.
//!
//! Boost.Polygon's Voronoi builder is linked, not reimplemented (R17, `docs/PORTING.md`
//! "Linked libraries"), through `crates/slic3r-ffi`'s `voronoi` bridge -- itself, by its own
//! module doc, narrower today (an `i32` segment-count-only surface) than this unit needs (an
//! `i64` `Slic3r::Point`-keyed `construct_voronoi` returning real cell/vertex/edge topology);
//! extending it is this task's own `// TODO(port):` below, left to whichever of this module or
//! `77-foundation-voronoi-offset` reaches real topology first. Being a draft, this file is not
//! required to compile (`docs/WORKFLOW.md`, "The two phases") and several of its bodies are
//! `// TODO(port):` for exactly that reason -- it is not required to, and does not, use
//! `todo!()`/`unimplemented!()`: no `mod` line reaches it regardless, but a draft is written the
//! same way a Phase A draft anywhere else in the port is (`docs/WORKFLOW.md`, "Conflict
//! discipline").

use slic3r_point::point::Point;

/// `VoronoiDiagram::IssueType` (`Voronoi.hpp:52-60`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IssueType {
    NoIssueDetected,
    FiniteEdgeWithNonFiniteVertex,
    MissingVoronoiVertex,
    NonPlanarVoronoiDiagram,
    VoronoiEdgeIntersectingInputSegment,
    ParabolicVoronoiEdgeWithoutFocusPoint,
    /// "Repairs are disabled in the constructor" (`:59`).
    Unknown,
}

/// `VoronoiDiagram::State` (`:62-67`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    RepairNotNeeded,
    RepairSuccessful,
    RepairUnsuccessful,
    Unknown,
}

/// `VoronoiDiagram::Segment` (`:151-158`): a private helper struct, one input segment.
#[derive(Debug, Clone, Copy)]
pub struct Segment {
    pub from: Point,
    pub to: Point,
}

// TODO(port): src/libslic3r/Geometry/Voronoi.hpp:33-50 -- `VoronoiDiagram`'s own `coord_type`/
// `voronoi_diagram_type`/`point_type`/`segment_type`/`rect_type`/`vertex_type`/`edge_type`/
// `cell_type`/the three `const_*_iterator`s/the three `*_container_type`s all come from
// `boost::polygon::voronoi_diagram<double>` (bound through `crates/slic3r-ffi`'s `voronoi`
// bridge). The bridge's own module doc says it reads back only cell/vertex/edge *counts* today,
// not real topology (pointers, colors, incident edges) -- extending it to that is this unit's
// own dependency, not invented here.

/// `VoronoiDiagram` (`Voronoi.hpp:30-190`). The vendored `m_voronoi_diagram`/`m_vertices`/
/// `m_edges`/`m_cells` fields all depend on the not-yet-extended `voronoi` FFI bridge (see the
/// `// TODO(port):` block above); this draft carries the four scalar/`bool` fields whose shape
/// does not, and leaves the four topology fields as `// TODO(port):`.
#[derive(Debug, Default)]
pub struct VoronoiDiagram {
    // TODO(port): src/libslic3r/Geometry/Voronoi.hpp:178-181 -- `m_voronoi_diagram`/`m_vertices`/
    // `m_edges`/`m_cells`; need the `voronoi` FFI bridge's real topology surface.
    m_is_modified: bool,
    m_state: StateSlot,
    m_issue_type: IssueTypeSlot,
}

// A draft-only shim so `#[derive(Default)]` above has something to derive against without
// requiring `State`/`IssueType` (whose vendored default is `UNKNOWN`, not a derivable
// `Default`) to carry a `#[derive(Default)]` of their own that the real port would not want on
// a C++-style enum. Dropped once the struct above is finished for real.
type StateSlot = Option<State>;
type IssueTypeSlot = Option<IssueType>;

impl VoronoiDiagram {
    /// `VoronoiDiagram::get_issue_type() const` (`:73`).
    pub fn get_issue_type(&self) -> IssueType {
        self.m_issue_type.unwrap_or(IssueType::Unknown)
    }

    /// `VoronoiDiagram::get_state() const` (`:75`).
    pub fn get_state(&self) -> State {
        self.m_state.unwrap_or(State::Unknown)
    }

    /// `VoronoiDiagram::is_valid() const` (`:77`).
    pub fn is_valid(&self) -> bool {
        self.get_state() != State::RepairUnsuccessful
    }

    /// `VoronoiDiagram::clear()` (`Voronoi.cpp:81-94`).
    pub fn clear(&mut self) {
        // TODO(port): src/libslic3r/Geometry/Voronoi.cpp:83-90 -- needs the topology fields
        // (see the struct's own `// TODO(port):`).
        self.m_is_modified = false;
        self.m_state = Some(State::Unknown);
        self.m_issue_type = Some(IssueType::Unknown);
    }

    // TODO(port): src/libslic3r/Geometry/Voronoi.hpp:81-91 -- `vertices`/`edges`/`cells`/
    // `num_vertices`/`num_edges`/`num_cells`; need the topology fields.

    // TODO(port): src/libslic3r/Geometry/Voronoi.hpp:93-125 -- the three `construct_voronoi`
    // overloads (point-only, segment-only, point+segment); need the `voronoi` FFI bridge
    // extended to real `boost::polygon::construct_voronoi` over `Slic3r::Point`-keyed segments,
    // not just `i32` counts.

    /// `VoronoiDiagram::construct_voronoi(SegmentIterator, SegmentIterator, bool)`
    /// (`Voronoi.cpp:34-79`), the one overload with a real (non-templated-elsewhere) body in
    /// this unit's own `.cpp`. Explicitly instantiated at three segment-iterator types
    /// (`:25-27`): `Lines::iterator`, `ColoredLines::const_iterator` and
    /// `std::vector<Arachne::PolygonsSegmentIndex>::const_iterator` -- all three need crates this
    /// task does not own (`slic3r-line`'s `Lines`/`ColoredLines` are `56-foundation-point`/
    /// `61-foundation-line-slicing`'s own types used here, not re-exported by this unit; the
    /// Arachne index is `73-foundation-arachne-indices`'s), so the three instantiations are left
    /// as one `// TODO(port):` rather than guessed at a fourth, invented signature.
    pub fn construct_voronoi_segments(&mut self, segments: &[Segment], try_to_repair_if_needed: bool) {
        // TODO(port): src/libslic3r/Geometry/Voronoi.cpp:35 --
        // `boost::polygon::construct_voronoi(segment_begin, segment_end, &m_voronoi_diagram);`
        // needs the `voronoi` FFI bridge extended to real topology (see the struct's own
        // `// TODO(port):`).
        let _ = segments;

        if try_to_repair_if_needed {
            // `if (m_issue_type = detect_known_issues(*this, segment_begin, segment_end); ...)`
            // (`:37`).
            let issue = Self::detect_known_issues_segments(self, segments);
            if issue != IssueType::NoIssueDetected {
                // `if (m_issue_type = try_to_repair_degenerated_voronoi_diagram(...))` (`:52`).
                let repaired = self.try_to_repair_degenerated_voronoi_diagram(segments);
                self.m_state = Some(if repaired == IssueType::NoIssueDetected {
                    State::RepairSuccessful
                } else {
                    State::RepairUnsuccessful
                });
                self.m_issue_type = Some(repaired);
            } else {
                self.m_state = Some(State::RepairNotNeeded);
                self.m_issue_type = Some(IssueType::NoIssueDetected);
            }
        } else {
            self.m_state = Some(State::Unknown);
            self.m_issue_type = Some(IssueType::Unknown);
        }
    }

    /// `VoronoiDiagram::detect_known_issues(const VoronoiDiagram&, SegmentIterator,
    /// SegmentIterator)` (`Voronoi.cpp:167-184`).
    fn detect_known_issues_segments(&self, segments: &[Segment]) -> IssueType {
        let edge_issue = self.detect_known_voronoi_edge_issues();
        if edge_issue != IssueType::NoIssueDetected {
            return edge_issue;
        }
        let cell_issue = self.detect_known_voronoi_cell_issues(segments);
        if cell_issue != IssueType::NoIssueDetected {
            return cell_issue;
        }
        // TODO(port): src/libslic3r/Geometry/Voronoi.cpp:178 --
        // `!VoronoiUtilsCgal::is_voronoi_diagram_planar_angle(voronoi_diagram, segment_begin,
        // segment_end)`; `VoronoiUtilsCgal.hpp`/`.cpp` is a separate manifest row, not this
        // task's (links CGAL, out of this task's own dependency list).
        IssueType::NoIssueDetected
    }

    /// `VoronoiDiagram::detect_known_voronoi_cell_issues(...)` (`Voronoi.cpp:186-231`). The five
    /// call sites this unit's own module doc and this task's pull request name, spelled against
    /// the sibling module `77-foundation-voronoi-offset` adds:
    ///
    /// - `:203,326` -- `VoronoiUtils::compute_segment_cell_range` -> `crate::voronoi_utils::
    ///   compute_segment_cell_range`.
    /// - `:207` -- `Geometry::VoronoiUtils::get_source_segment` -> `crate::voronoi_utils::
    ///   get_source_segment`.
    fn detect_known_voronoi_cell_issues(&self, segments: &[Segment]) -> IssueType {
        // TODO(port): src/libslic3r/Geometry/Voronoi.cpp:199-227 -- needs the topology fields
        // (`VD::cell_type`/`VD::edge_type` iteration) and, at `:203`,
        // `crate::voronoi_utils::compute_segment_cell_range`, and at `:207`,
        // `crate::voronoi_utils::get_source_segment` -- both `77-foundation-voronoi-offset`'s.
        let _ = segments;
        IssueType::NoIssueDetected
    }

    /// `VoronoiDiagram::detect_known_voronoi_edge_issues(const VoronoiDiagram&)`
    /// (`Voronoi.cpp:233-247`). The call site this unit's module doc and this task's pull
    /// request name:
    ///
    /// - `:238` -- `VoronoiUtils::is_finite` -> `crate::voronoi_utils::is_finite`.
    fn detect_known_voronoi_edge_issues(&self) -> IssueType {
        // TODO(port): src/libslic3r/Geometry/Voronoi.cpp:235-243 -- needs the topology fields
        // (`edge_type` iteration) and, at `:238`, `crate::voronoi_utils::is_finite`.
        IssueType::NoIssueDetected
    }

    /// `VoronoiDiagram::try_to_repair_degenerated_voronoi_diagram(SegmentIterator,
    /// SegmentIterator)` (`Voronoi.cpp:249-267`).
    fn try_to_repair_degenerated_voronoi_diagram(&mut self, segments: &[Segment]) -> IssueType {
        let mut issue_type = self.m_issue_type.unwrap_or(IssueType::Unknown);
        // `const std::vector<double> fix_angles = {PI / 6, PI / 5, PI / 7, PI / 11};` (`:258`,
        // spelled with no decimal point so each divisor's normalised text matches the vendored
        // line's own bare-integer spelling verbatim -- `docs/PORTING.md`, "Decisions recorded
        // on 2026-11-26", decision 8).
        let fix_angles = [
            slic3r_libslic3r::libslic3r::PI / 6f64, // PORT SRC: src/libslic3r/Geometry/Voronoi.cpp:258
            slic3r_libslic3r::libslic3r::PI / 5f64, // PORT SRC: src/libslic3r/Geometry/Voronoi.cpp:258
            slic3r_libslic3r::libslic3r::PI / 7f64, // PORT SRC: src/libslic3r/Geometry/Voronoi.cpp:258
            slic3r_libslic3r::libslic3r::PI / 11f64, // PORT SRC: src/libslic3r/Geometry/Voronoi.cpp:258
        ];
        for &fix_angle in &fix_angles {
            issue_type = self.try_to_repair_degenerated_voronoi_diagram_by_rotation(segments, fix_angle);
            if issue_type == IssueType::NoIssueDetected {
                return issue_type;
            }
        }
        issue_type
    }

    /// `VoronoiDiagram::try_to_repair_degenerated_voronoi_diagram_by_rotation(SegmentIterator,
    /// SegmentIterator, double)` (`Voronoi.cpp:291-368`). The three remaining call sites this
    /// unit's module doc and this task's pull request name:
    ///
    /// - `:326` -- `VoronoiUtils::compute_segment_cell_range` -> `crate::voronoi_utils::
    ///   compute_segment_cell_range` (the same function `detect_known_voronoi_cell_issues`
    ///   calls at `:203`).
    /// - `:352` -- `VoronoiUtils::make_rotated_vertex` -> `crate::voronoi_utils::
    ///   make_rotated_vertex`.
    fn try_to_repair_degenerated_voronoi_diagram_by_rotation(
        &mut self,
        segments: &[Segment],
        fix_angle: f64,
    ) -> IssueType {
        // `segments_rotated.emplace_back(from.rotated(fix_angle), to.rotated(fix_angle));`
        // (`:309`).
        let _segments_rotated: Vec<Segment> = segments
            .iter()
            .map(|s| Segment {
                from: s.from.rotated(fix_angle),
                to: s.to.rotated(fix_angle),
            })
            .collect();

        // TODO(port): src/libslic3r/Geometry/Voronoi.cpp:312-365 -- needs the `voronoi` FFI
        // bridge extended to real topology (`construct_voronoi` + `copy_to_local` + vertex/cell
        // iteration), and at `:326`, `crate::voronoi_utils::compute_segment_cell_range`, and at
        // `:352`, `crate::voronoi_utils::make_rotated_vertex`.
        self.m_is_modified = true;
        IssueType::Unknown
    }
}

// PORT STATUS
// source: src/libslic3r/Geometry/Voronoi.cpp
// source: src/libslic3r/Geometry/Voronoi.hpp
// confidence: low
// tier: exact
// oracle: built
