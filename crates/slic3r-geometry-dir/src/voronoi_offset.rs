//! Port of `src/libslic3r/Geometry/VoronoiOffset.cpp`/`VoronoiOffset.hpp` (task
//! `77-foundation-voronoi-offset`, plan step 20, `port/foundation.tsv`'s `foundation-26` row).
//! `VoronoiOffset.hpp` is `template-heavy: yes`; `VoronoiOffset.cpp` is `template-heavy: no` but
//! hand-designed alongside it in this same task because the two are one unit.
//!
//! // PORT NOTE: this module is an honest Phase A draft, not a ported module: no `mod
//! voronoi_offset;` line reaches it, it carries no `// PROOF:` marker, and `fuzz target: n/a`
//! for both its manifest rows regardless.
//!
//! **The same blocker [`crate::voronoi_utils`]'s module doc states in full.** Every public
//! function this unit's own header declares -- `reset_inside_outside_annotations`,
//! `annotate_inside_outside`, `signed_vertex_distances`, `edge_offset_contour_intersections`,
//! `skeleton_edges_rough` and both `offset` overloads -- takes a `const VD &vd` (or `VD &`) and
//! walks its vertices, edges and cells by `vd.vertices()`/`.edges()`/`.cells()` and
//! `cell.incident_edge()`/`edge->next()`/`edge->twin()`/`vertex0()`/`vertex1()`, reading and
//! writing each node's "color" (`.color()`/`set_*_category`) along the way. `crates/slic3r-ffi`'s
//! `voronoi` bridge exposes none of that (counts only), and extending it is outside this task's
//! own enumerated file list -- the one block that stands, stated in full in
//! [`crate::voronoi_utils`]'s module doc.
//!
//! Two claims an earlier draft of this doc made alongside it are **withdrawn**, for the reasons
//! that doc now gives in full: `docs/PORTING.md` decision 1's "never a libslic3r type" sentence
//! scopes the per-unit **proof shim** (`docs/PORTING.md:562-577`), not the R17 library bridges, so
//! reading it over them is this port's own inference and the escalation needed is a file-list
//! amendment alone; and the proof route itself is **open** (decision 1's `proof_units/` files are
//! new files, and its proof lives in exactly the `#[cfg(test)]`/`tests/` dev-dependency call this
//! crate can make). What this unit declines, measured rather than asserted, is deriving `ported`
//! from that route: a `#[cfg(test)]` proof needs a `mod` line, and
//! `port/port-status:863-881`'s `derive_row_status` then returns `ported` -- which for a
//! 1,646-line `.cpp` with 27 `// TODO(port):` markers still standing here, including all seven
//! public entry points, would be a false manifest status cell. That disposition is
//! `docs/PORTING.md` decisions 9 and 10's call, not a porter's. Per `docs/WORKFLOW.md`'s own
//! rule, this module stays an honest Phase A draft, unreached by any `mod` line, and does not use
//! `todo!()`/`unimplemented!()`.
//!
//! **What is genuinely ported below, with no topology dependency at all:** the three "color"
//! enums ([`VertexCategory`], [`EdgeCategory`], [`CellCategory`], plain data, no behaviour to
//! prove), [`edge_offset_no_intersection`]/[`edge_offset_has_intersection`] (a bare `isnan` check
//! on a `Vec2d`), the vendored `namespace detail`'s pure-arithmetic prefix
//! ([`first_circle_segment_intersection_parameter`], [`Intersections`] and
//! [`point_point_equal_distance_points`]/[`line_point_equal_distance_points`], `:31-210`), and
//! two of the four free functions that follow `detail::on_site` ([`point_point_dr_dl_thresholds`]
//! and [`point_point_skeleton_thresholds`], `:267-305` and `:372-401`) -- despite sitting between
//! two topology-dependent sections, neither touches a `cell`/`edge`/`vertex` at all; see each
//! function's own doc. Everything else -- `contour_point`/`vertex_point`/`*_category`/
//! `set_*_category` (`VoronoiOffset.hpp:21-102`), `detail::vertex_equal_to_point`/`dist_to_site`/
//! `on_site` (`VoronoiOffset.cpp:212-256`), `detail::point_segment_dr_dl_thresholds`/
//! `point_segment_skeleton_thresholds` (`:316-370`, `:403-449`, blocked on `Geometry::foot_pt`,
//! not topology -- see each function's own `// TODO(port):` below), the `debug::verify_*`
//! self-checks (`:457-648`, `#ifndef NDEBUG`), and the seven public entry points (`:650-1646`) --
//! needs a live diagram (the `point_segment_*` pair needs `foot_pt` instead) and is left as
//! `// TODO(port):` below, one marker per function naming its own vendored line range, per this
//! task's own pull request.

use slic3r_point::point::Vec2d;

// -------------------------------------------------------------------------------------------
// The three "color" enums (VoronoiOffset.hpp:34-75). Plain data: no behaviour of their own to
// prove, and no live diagram to attach them to yet (see the module doc).
// -------------------------------------------------------------------------------------------

/// "Color" stored inside the boost::polygon Voronoi vertex (`VoronoiOffset.hpp:35-47`).
/// `#[repr(u8)]` because the vendored enum is backed by `VD::vertex_type::color_type`
/// (`unsigned char`) and is round-tripped through it by `vertex_category`/`set_vertex_category`
/// (`:77-84`, both still `// TODO(port):` -- they need a live `vertex_type`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum VertexCategory {
    /// Voronoi vertex is on the input contour. `VD::vertex_type` stores coordinates in `double`,
    /// though the coordinates shall match exactly with the coordinates of the input contour when
    /// converted to `int32_t` (`:38-39`).
    OnContour,
    /// Vertex is inside the CCW input contour, holes are respected.
    Inside,
    /// Vertex is outside the CCW input contour, holes are respected.
    Outside,
    /// Not known yet.
    Unknown,
}

/// "Color" stored inside the boost::polygon Voronoi edge (`VoronoiOffset.hpp:49-62`). The
/// Voronoi edge as represented by boost::polygon's Voronoi module is really a half-edge; the
/// half-edges are classified based on the target vertex (`VD::vertex_type::vertex1()`).
/// `#[repr(u8)]`; see [`VertexCategory`]'s doc.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum EdgeCategory {
    /// This half-edge points onto the contour: this `VD::edge_type::vertex1().color()` is
    /// `OnContour`.
    PointsToContour,
    /// This half-edge points inside: this `VD::edge_type::vertex1().color()` is `Inside`.
    PointsInside,
    /// This half-edge points outside: this `VD::edge_type::vertex1().color()` is `Outside`.
    PointsOutside,
    /// Not known yet.
    Unknown,
}

/// "Color" stored inside the boost::polygon Voronoi cell (`VoronoiOffset.hpp:64-75`).
/// `#[repr(u8)]`; see [`VertexCategory`]'s doc.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum CellCategory {
    /// This Voronoi cell is split by an input segment to two halves, one is inside, the other is
    /// outside.
    Boundary,
    /// This Voronoi cell is completely inside.
    Inside,
    /// This Voronoi cell is completely outside.
    Outside,
    /// Not known yet.
    Unknown,
}

// TODO(port): src/libslic3r/Geometry/VoronoiOffset.hpp:21-29 -- `contour_point(const VD::cell_type
// &, const Line &)` (both const/non-const overloads) and `contour_point(const VD::cell_type &,
// const Lines &)` (both overloads); need `cell.source_category()`/`.source_index()`.
// TODO(port): src/libslic3r/Geometry/VoronoiOffset.hpp:31-32 -- `vertex_point` over both a
// `const VD::vertex_type &` and a `const VD::vertex_type *`; need `vertex.x()`/`.y()` (same
// dependency as `crate::voronoi_utils::to_point`).
// TODO(port): src/libslic3r/Geometry/VoronoiOffset.hpp:77-84 -- `vertex_category`/
// `set_vertex_category`; need a live `vertex_type` and its `.color()`/`color_type`.
// TODO(port): src/libslic3r/Geometry/VoronoiOffset.hpp:86-93 -- `edge_category`/
// `set_edge_category`; need a live `edge_type`.
// TODO(port): src/libslic3r/Geometry/VoronoiOffset.hpp:95-102 -- `cell_category`/
// `set_cell_category`; need a live `cell_type`.

// -------------------------------------------------------------------------------------------
// edge_offset_no_intersection / edge_offset_has_intersection (VoronoiOffset.hpp:115-118) -- a
// bare `isnan` check on a plain Vec2d, no topology.
// -------------------------------------------------------------------------------------------

/// `static inline bool edge_offset_no_intersection(const Vec2d &intersection_point) { return
/// std::isnan(intersection_point.x()); }` (`:115-116`).
pub fn edge_offset_no_intersection(intersection_point: Vec2d) -> bool {
    intersection_point.x().is_nan()
}

/// `static inline bool edge_offset_has_intersection(const Vec2d &intersection_point) { return !
/// edge_offset_no_intersection(intersection_point); }` (`:117-118`).
pub fn edge_offset_has_intersection(intersection_point: Vec2d) -> bool {
    !edge_offset_no_intersection(intersection_point)
}

// TODO(port): src/libslic3r/Geometry/VoronoiOffset.hpp:105 -- `reset_inside_outside_annotations(VD
// &)`; needs a live diagram's vertices/edges/cells (see the module doc).
// TODO(port): src/libslic3r/Geometry/VoronoiOffset.hpp:109 -- `annotate_inside_outside(VD &,
// const Lines &)`; same dependency, defined at `VoronoiOffset.cpp:660-975`.
// TODO(port): src/libslic3r/Geometry/VoronoiOffset.hpp:113 -- `signed_vertex_distances(const VD &,
// const Lines &)`; same dependency, defined at `VoronoiOffset.cpp:977-1018`.
// TODO(port): src/libslic3r/Geometry/VoronoiOffset.hpp:119-121 --
// `edge_offset_contour_intersections(const VD &, const Lines &, const std::vector<double> &,
// double)`; same dependency, defined at `VoronoiOffset.cpp:1020-1335`.
// TODO(port): src/libslic3r/Geometry/VoronoiOffset.hpp:123-126 -- `skeleton_edges_rough(const VD
// &, const Lines &, double)`; same dependency, defined at `VoronoiOffset.cpp:1543-1646`.
// TODO(port): src/libslic3r/Geometry/VoronoiOffset.hpp:128-133 -- `offset(const
// Geometry::VoronoiDiagram &, const Lines &, const std::vector<double> &, double, double)`; same
// dependency, defined at `VoronoiOffset.cpp:1337-1519`.
// TODO(port): src/libslic3r/Geometry/VoronoiOffset.hpp:140-144 -- `offset(const VD &, const Lines
// &, double, double)`; same dependency, defined at `VoronoiOffset.cpp:1521-1541`.

// -------------------------------------------------------------------------------------------
// namespace detail (VoronoiOffset.cpp:30-451) -- a named (not anonymous) namespace, so these
// carry external linkage at the pin; ported as private (non-`pub`) functions per the idiom map's
// "free function in an anonymous namespace, or a `static` free function" row, since none is
// `pub` in the vendored header either and nothing outside this translation unit calls them.
// `:31-210` (pure arithmetic) and two of the four functions at `:258-449` port below with no
// live diagram; `:212-256` and the other two of those four (`:316-370`, `:403-449`, blocked on
// `Geometry::foot_pt` instead) do not -- see the module doc and each function's own doc/TODO.
// `namespace debug` (`:453-648`, `#ifndef NDEBUG`) is a separate, sibling namespace, not part of
// `detail`; its five `verify_*` self-checks are TODO(port)'d separately below.
// -------------------------------------------------------------------------------------------

/// Intersect a circle with a ray, return the parameter of the intersection that lies on the
/// segment. Currently used for unbounded Voronoi edges only (`VoronoiOffset.cpp:31-69`).
///
/// Every vendored `assert(...)` in this body is a debug-only self-check (the start point inside,
/// the end point outside the circle; the discriminant and the found root in range) and is not
/// ported, matching every other module's own precedent for a debug-only `assert`.
fn first_circle_segment_intersection_parameter(center: Vec2d, r: f64, pt: Vec2d, v: Vec2d) -> f64 {
    let d = pt - center;
    let a = vec2d_dot(v, v);
    let b = 2. * vec2d_dot(d, v); // PORT SRC: src/libslic3r/Geometry/VoronoiOffset.cpp:45
    let c = vec2d_dot(d, d) - r * r;
    let u = b * b - 4. * a * c; // PORT SRC: src/libslic3r/Geometry/VoronoiOffset.cpp:48

    // `if (u <= 0)` (`:51`) spells this comparison's zero as a bare integer, not `0.` -- `0f64`
    // below (normalised text `0`, R16's exempt set) matches that spelling, per
    // `docs/PORTING.md` decision 8 of 2026-11-26. Bound to a `let` (rather than written as a
    // trailing comment on the `if` itself) so `cargo fmt` cannot relocate the marker off the
    // line it covers.
    let u_le_zero = u <= 0f64; // PORT SRC: src/libslic3r/Geometry/VoronoiOffset.cpp:51
    if u_le_zero {
        // Degenerate to a single closest point.
        let t = -b / (2. * a); // PORT SRC: src/libslic3r/Geometry/VoronoiOffset.cpp:53
        t.clamp(0., 1.) // PORT SRC: src/libslic3r/Geometry/VoronoiOffset.cpp:55
    } else {
        let u = u.sqrt();
        let t0 = (-b - u) / (2. * a); // PORT SRC: src/libslic3r/Geometry/VoronoiOffset.cpp:59
        let t1 = (-b + u) / (2. * a); // PORT SRC: src/libslic3r/Geometry/VoronoiOffset.cpp:60

        // One of the intersections shall be found inside the segment. Each comparison is bound
        // to a `let` for the same `cargo fmt` reason as `u_le_zero` above.
        let t1_lt_zero = t1 < 0.; // PORT SRC: src/libslic3r/Geometry/VoronoiOffset.cpp:63
        let t0_gt_one = t0 > 1.; // PORT SRC: src/libslic3r/Geometry/VoronoiOffset.cpp:65
        let t0_gt_zero = t0 > 0.; // PORT SRC: src/libslic3r/Geometry/VoronoiOffset.cpp:67
        if t1_lt_zero {
            0. // PORT SRC: src/libslic3r/Geometry/VoronoiOffset.cpp:64
        } else if t0_gt_one {
            1. // PORT SRC: src/libslic3r/Geometry/VoronoiOffset.cpp:66
        } else if t0_gt_zero {
            t0
        } else {
            t1
        }
    }
}

/// `struct Intersections` (`VoronoiOffset.cpp:71-75`): up to two points, with `count` saying how
/// many of `pts` are populated.
#[derive(Debug, Clone, Copy)]
struct Intersections {
    count: i32,
    pts: [Vec2d; 2],
}

/// `v1.dot(v2)` for a plain `Vec2d` pair; a local helper, the same choice `slic3r-geometry`'s
/// `geometry.rs` and `crate::voronoi_utils` make for their own single-instantiation uses of
/// `dot`/`perp`/`cross2`.
fn vec2d_dot(v1: Vec2d, v2: Vec2d) -> f64 {
    v1.x() * v2.x() + v1.y() * v2.y()
}

/// `cross2(v1, v2)` (`Point.hpp`'s free `cross2`, `return v1.x() * v2.y() - v1.y() * v2.x();`) at
/// its one `Vec2d` instantiation this unit needs; see [`vec2d_dot`]'s doc for why this is a
/// local helper.
fn vec2d_cross2(v1: Vec2d, v2: Vec2d) -> f64 {
    v1.x() * v2.y() - v1.y() * v2.x()
}

/// Return maximum two points that are at distance `d` from both `pt1` and `pt2`
/// (`VoronoiOffset.cpp:78-126`).
///
/// Every vendored `assert(...)` that only re-verifies the result (`:121-124`) is a debug-only
/// self-check and is not ported.
fn point_point_equal_distance_points(
    pt1: slic3r_point::point::Point,
    pt2: slic3r_point::point::Point,
    d: f64,
) -> Intersections {
    // Calculate the two intersection points.
    // With the help of Python package sympy:
    //      res = solve([(x - cx)**2 + (y - cy)**2 - d**2, x**2 + y**2 - d**2], [x, y])
    //      ccode(cse((res[0][0], res[0][1], res[1][0], res[1][1])))
    // where cx, cy is the center of pt1 relative to pt2,
    // d is distance from the line and the point (0, 0).
    // The result is then shifted to pt2.
    let mut cx = (pt1.x() - pt2.x()) as f64;
    let mut cy = (pt1.y() - pt2.y()) as f64;
    let cl = cx * cx + cy * cy;
    let discr = 4. * d * d - cl; // PORT SRC: src/libslic3r/Geometry/VoronoiOffset.cpp:90

    // Bound to a `let` (rather than written as a trailing comment on the `if` itself) so
    // `cargo fmt` cannot relocate the marker off the line it covers.
    let discr_lt_zero = discr < 0.; // PORT SRC: src/libslic3r/Geometry/VoronoiOffset.cpp:91
    if discr_lt_zero {
        // No intersection point found, the two circles are too far away.
        // PORT NOTE: src/libslic3r/Geometry/VoronoiOffset.cpp:93 -- `Vec2d()`/`Vec2d()` are
        // default-constructed, carrying no literal of their own. Eigen's default constructor
        // leaves the storage indeterminate (`EIGEN_INITIALIZE_MATRICES_BY_ZERO` is defined
        // nowhere in this build); `Vec2d::default()` substitutes a defined zero for that
        // indeterminate value -- a divergence, not an equivalence, but unobservable here because
        // every caller checks `count` before ever reading `pts`.
        return Intersections {
            count: 0,
            pts: [Vec2d::default(), Vec2d::default()],
        };
    }
    // Avoid division by zero if a gets too small.
    let xy_swapped = cx.abs() < cy.abs();
    if xy_swapped {
        std::mem::swap(&mut cx, &mut cy);
    }
    // `u = 0;` (`:103`) spells this value's zero as a bare integer, not `0.` -- `0f64` below
    // (normalised text `0`, R16's exempt set) matches that spelling, per `docs/PORTING.md`
    // decision 8 of 2026-11-26.
    let discr_eq_zero = discr == 0.; // PORT SRC: src/libslic3r/Geometry/VoronoiOffset.cpp:101
    let (cnt, u) = if discr_eq_zero {
        (1, 0f64) // PORT SRC: src/libslic3r/Geometry/VoronoiOffset.cpp:103
    } else {
        (2, 0.5 * cx * (cl * discr).sqrt() / cl) // PORT SRC: src/libslic3r/Geometry/VoronoiOffset.cpp:106
    };
    let v = 0.5 * cy - u; // PORT SRC: src/libslic3r/Geometry/VoronoiOffset.cpp:108
    let w = 2. * cy; // PORT SRC: src/libslic3r/Geometry/VoronoiOffset.cpp:109
    let e = 0.5 / cx; // PORT SRC: src/libslic3r/Geometry/VoronoiOffset.cpp:110
    let f = 0.5 * cy + u; // PORT SRC: src/libslic3r/Geometry/VoronoiOffset.cpp:111
    let mut pts = [
        Vec2d::new(-e * (v * w - cl), v),
        Vec2d::new(-e * (w * f - cl), f),
    ];
    if xy_swapped {
        pts[0] = Vec2d::new(pts[0].y(), pts[0].x());
        pts[1] = Vec2d::new(pts[1].y(), pts[1].x());
    }
    let pt2_d = Vec2d::new(pt2.x() as f64, pt2.y() as f64);
    pts[0] = pts[0] + pt2_d;
    pts[1] = pts[1] + pt2_d;

    Intersections { count: cnt, pts }
}

/// Return maximum two points that are at distance `d` from both the line and point
/// (`VoronoiOffset.cpp:129-210`).
///
/// Every vendored `assert(...)` that only re-verifies an intermediate or the result (`:131,148,
/// 150,160,205-208`) is a debug-only self-check and is not ported; the `#ifndef NDEBUG` block at
/// `:152-162` (the foot-point/center-point cross-check) is entirely one such self-check and is
/// dropped whole, not just its individual `assert`.
///
/// `Geometry::foot_pt`/`Geometry::ray_point_distance` (`:155,205-206`, inside the dropped
/// debug-only block and in the ported `assert`s this function's own doc already drops) are not
/// referenced by the ported body below for that reason; neither is a `// TODO(port):` sited on
/// them -- they are debug-only self-check call sites, not computation this function's return
/// value depends on.
fn line_point_equal_distance_points(
    line: slic3r_line::line::Line,
    ipt: slic3r_point::point::Point,
    d: f64,
) -> Intersections {
    // Calculating two points of distance "d" to a ray and a point.
    let pt = Vec2d::new(ipt.x() as f64, ipt.y() as f64);
    let mut lv = Vec2d::new(
        (line.b.x() - line.a.x()) as f64,
        (line.b.y() - line.a.y()) as f64,
    );
    let l2 = lv.x() * lv.x() + lv.y() * lv.y();
    let lpv = Vec2d::new((line.a.x() - ipt.x()) as f64, (line.a.y() - ipt.y()) as f64);
    let mut c = vec2d_cross2(lpv, lv);
    // `if (c < 0)` (`:139`) spells this comparison's zero as a bare integer, not `0.` --
    // `0f64` below (normalised text `0`, R16's exempt set) matches that spelling, per
    // `docs/PORTING.md` decision 8 of 2026-11-26. Bound to a `let` (rather than written as a
    // trailing comment on the `if` itself) so `cargo fmt` cannot relocate the marker off the
    // line it covers.
    let c_lt_zero = c < 0f64; // PORT SRC: src/libslic3r/Geometry/VoronoiOffset.cpp:139
    if c_lt_zero {
        lv = Vec2d::new(-lv.x(), -lv.y());
        c = -c;
    }

    // Line equation (ax + by + c - d * sqrt(l2)).
    let mut a = -lv.y();
    let mut b = lv.x();

    let dscaled = d * l2.sqrt();
    let s = c * (2. * dscaled - c); // PORT SRC: src/libslic3r/Geometry/VoronoiOffset.cpp:173

    // Bound to a `let` (rather than written as a trailing comment on the `if` itself) so
    // `cargo fmt` cannot relocate the marker off the line it covers.
    let s_lt_zero = s < 0.; // PORT SRC: src/libslic3r/Geometry/VoronoiOffset.cpp:174
    if s_lt_zero {
        // Distance of pt from line is bigger than 2 * d.
        // PORT NOTE: src/libslic3r/Geometry/VoronoiOffset.cpp:176 -- `Vec2d()`/`Vec2d()` (the
        // implicit `Intersections { 0 }` aggregate-init) are default-constructed, carrying no
        // literal of their own. Eigen's default constructor leaves the storage indeterminate
        // (`EIGEN_INITIALIZE_MATRICES_BY_ZERO` is defined nowhere in this build);
        // `Vec2d::default()` substitutes a defined zero for that indeterminate value -- a
        // divergence, not an equivalence, but unobservable here because every caller checks
        // `count` before ever reading `pts`.
        return Intersections {
            count: 0,
            pts: [Vec2d::default(), Vec2d::default()],
        };
    }
    // Avoid division by zero if a gets too small.
    let xy_swapped = a.abs() < b.abs();
    if xy_swapped {
        std::mem::swap(&mut a, &mut b);
    }
    let s_eq_zero = s == 0.; // PORT SRC: src/libslic3r/Geometry/VoronoiOffset.cpp:183
    let (cnt, u) = if s_eq_zero {
        // Distance of pt from line is 2 * d.
        (1, 0.) // PORT SRC: src/libslic3r/Geometry/VoronoiOffset.cpp:186
    } else {
        // Distance of pt from line is smaller than 2 * d.
        (2, a * s.sqrt() / l2)
    };
    let e = dscaled - c;
    let f = b * e / l2;
    let g = f - u;
    let h = f + u;
    let mut pts = [
        Vec2d::new((-b * g + e) / a, g),
        Vec2d::new((-b * h + e) / a, h),
    ];
    if xy_swapped {
        pts[0] = Vec2d::new(pts[0].y(), pts[0].x());
        pts[1] = Vec2d::new(pts[1].y(), pts[1].x());
    }
    pts[0] = pts[0] + pt;
    pts[1] = pts[1] + pt;

    Intersections { count: cnt, pts }
}

// TODO(port): src/libslic3r/Geometry/VoronoiOffset.cpp:212-236 -- `detail::vertex_equal_to_point`
// (the generic `VertexType` template, instantiated at `Vec2d` by `on_site` at `:249`, and the
// `const VD::vertex_type *` overload at `:235-236`); the generic form is pure ULP comparison
// (`boost::polygon::detail::ulp_comparison<double>`) but its only in-scope caller needs a live
// `cell_type` regardless (see the next marker), so it is left whole rather than ported standalone
// with no caller to prove it against.
// TODO(port): src/libslic3r/Geometry/VoronoiOffset.cpp:238-244 -- `detail::dist_to_site(const
// Lines &, const VD::cell_type &, const Vec2d &)`; needs `cell.source_index()`/
// `.contains_point()`/`.source_category()`.
// TODO(port): src/libslic3r/Geometry/VoronoiOffset.cpp:246-256 -- `detail::on_site`; same
// dependency as `dist_to_site`, plus `cell.incident_edge()`.

// -------------------------------------------------------------------------------------------
// Two of the four free functions that follow `on_site` (VoronoiOffset.cpp:258-449) -- despite
// sitting between topology-dependent code on both sides, neither of these two touches a
// `cell`/`edge`/`vertex` anywhere in its vendored body (checked: no `VD::`, `cell.`, `edge->`,
// `vertex0`/`vertex1`, `incident_edge` or `.color(` in `:267-305` or `:372-401`). The other two,
// `point_segment_dr_dl_thresholds`/`point_segment_skeleton_thresholds`, are genuinely blocked --
// not by topology, but by `Geometry::foot_pt<Vec2d>`, which is itself still
// `// TODO(port): src/libslic3r/Geometry.hpp:186-217` at `crates/slic3r-geometry/src/geometry.rs:
// 115` -- so they stay `// TODO(port):` below rather than being ported with a stub in place of
// `foot_pt`.
// -------------------------------------------------------------------------------------------

/// Local helper: the Euclidean norm of a plain `Vec2d`, via [`vec2d_dot`]; see that function's
/// doc for why this unit keeps its own vector helpers rather than reaching for a shared one.
fn vec2d_norm(v: Vec2d) -> f64 {
    vec2d_dot(v, v).sqrt()
}

/// `v.normalized()` for a plain `Vec2d`; duplicated from `crate::voronoi_utils`'s own
/// `vec2d_normalized` rather than shared, the same per-module-local choice [`vec2d_dot`]'s doc
/// states. Carries Eigen's own zero-norm guard (`z > 0 ? n / sqrt(z) : n`,
/// `Eigen/src/Core/Dot.h:92-102` under the dependency superbuild); see `crate::voronoi_utils`'s
/// copy for the full citation.
// PORT NOTE: src/libslic3r/Geometry/VoronoiOffset.cpp:286 -- `.normalized()` is Eigen's own
// zero-guarded normalize, not a plain `v / v.norm()`.
fn vec2d_normalized(v: Vec2d) -> Vec2d {
    let squared_norm = v.x() * v.x() + v.y() * v.y();
    if squared_norm > 0f64 {
        let norm = squared_norm.sqrt();
        Vec2d::new(v.x() / norm, v.y() / norm)
    } else {
        v
    }
}

/// For a Voronoi segment bisecting two point sites `pt1_site`/`pt2_site`, the two points along
/// the segment (clamped to its own endpoints `voronoi_point1`/`voronoi_point2`) where the dr/dl
/// measure crosses `threshold_tan_alpha_half` (`VoronoiOffset.cpp:267-305`). Pure `Point`/`Vec2d`
/// arithmetic -- see the block comment above for why this one ports with no live diagram while
/// its sibling `point_segment_dr_dl_thresholds` does not. Has a live in-file caller at `:1609`
/// (inside `skeleton_edges_rough`, itself still `// TODO(port):` below because it walks a live
/// `VD` to reach this function).
///
/// Every vendored `assert`/debug-only branch in this body's range is absent (there is none): the
/// body below is a line-for-line translation of `:267-305`.
fn point_point_dr_dl_thresholds(
    pt1_site: slic3r_point::point::Point,
    pt2_site: slic3r_point::point::Point,
    voronoi_point1: Vec2d,
    voronoi_point2: Vec2d,
    threshold_tan_alpha_half: f64,
) -> (Vec2d, Vec2d) {
    let dir_y = Vec2d::new(
        (pt2_site.x() - pt1_site.x()) as f64,
        (pt2_site.y() - pt1_site.y()) as f64,
    );
    let mut dir_x = vec2d_normalized(Vec2d::new(-dir_y.y(), dir_y.x()));
    let cntr = Vec2d::new(
        0.5 * (pt1_site.x() as f64 + pt2_site.x() as f64), // PORT SRC: src/libslic3r/Geometry/VoronoiOffset.cpp:287
        0.5 * (pt1_site.y() as f64 + pt2_site.y() as f64), // PORT SRC: src/libslic3r/Geometry/VoronoiOffset.cpp:287
    );
    let mut t1 = vec2d_dot(voronoi_point1 - cntr, dir_x);
    let mut t2 = vec2d_dot(voronoi_point2 - cntr, dir_x);
    if t1 > t2 {
        t1 = -t1;
        t2 = -t2;
        dir_x = -dir_x;
    }
    let x = 0.5 * vec2d_norm(dir_y) * threshold_tan_alpha_half; // PORT SRC: src/libslic3r/Geometry/VoronoiOffset.cpp:295
    let nan = f64::NAN;
    let mut out = (Vec2d::new(nan, nan), Vec2d::new(nan, nan));
    let intervals_overlap = t2 > -x && t1 < x;
    if intervals_overlap {
        // Intervals overlap.
        dir_x = Vec2d::new(dir_x.x() * x, dir_x.y() * x);
        out.0 = if t1 < -x {
            cntr - dir_x
        } else {
            voronoi_point1
        };
        out.1 = if t2 > x { cntr + dir_x } else { voronoi_point2 };
    }
    out
}

// TODO(port): src/libslic3r/Geometry/VoronoiOffset.cpp:316-370 -- `detail::
// point_segment_dr_dl_thresholds`; pure `Point`/`Line`/`Vec2d` arithmetic apart from one call to
// `Geometry::foot_pt<Vec2d>` (`:334`), which is itself `// TODO(port):` at
// `crates/slic3r-geometry/src/geometry.rs:115` (`src/libslic3r/Geometry.hpp:186-217`) -- blocked
// on that, not on diagram topology.

/// For a Voronoi segment bisecting two point sites, the two points along it where the dr/dl
/// measure crosses `tan_alpha_half` (`VoronoiOffset.cpp:372-401`) -- the same shape as
/// [`point_point_dr_dl_thresholds`] with a different threshold parameter name at the pin; kept as
/// its own function rather than merged with it, matching the vendored source having two.
fn point_point_skeleton_thresholds(
    pt1_site: slic3r_point::point::Point,
    pt2_site: slic3r_point::point::Point,
    voronoi_point1: Vec2d,
    voronoi_point2: Vec2d,
    tan_alpha_half: f64,
) -> (Vec2d, Vec2d) {
    let dir_y = Vec2d::new(
        (pt2_site.x() - pt1_site.x()) as f64,
        (pt2_site.y() - pt1_site.y()) as f64,
    );
    let mut dir_x = vec2d_normalized(Vec2d::new(-dir_y.y(), dir_y.x()));
    let cntr = Vec2d::new(
        0.5 * (pt1_site.x() as f64 + pt2_site.x() as f64), // PORT SRC: src/libslic3r/Geometry/VoronoiOffset.cpp:383
        0.5 * (pt1_site.y() as f64 + pt2_site.y() as f64), // PORT SRC: src/libslic3r/Geometry/VoronoiOffset.cpp:383
    );
    let mut t1 = vec2d_dot(voronoi_point1 - cntr, dir_x);
    let mut t2 = vec2d_dot(voronoi_point2 - cntr, dir_x);
    if t1 > t2 {
        t1 = -t1;
        t2 = -t2;
        dir_x = -dir_x;
    }
    let x = 0.5 * vec2d_norm(dir_y) * tan_alpha_half; // PORT SRC: src/libslic3r/Geometry/VoronoiOffset.cpp:391
    let nan = f64::NAN;
    let mut out = (Vec2d::new(nan, nan), Vec2d::new(nan, nan));
    let intervals_overlap = t2 > -x && t1 < x;
    if intervals_overlap {
        // Intervals overlap.
        dir_x = Vec2d::new(dir_x.x() * x, dir_x.y() * x);
        out.0 = if t1 < -x {
            cntr - dir_x
        } else {
            voronoi_point1
        };
        out.1 = if t2 > x { cntr + dir_x } else { voronoi_point2 };
    }
    out
}

// TODO(port): src/libslic3r/Geometry/VoronoiOffset.cpp:403-449 -- `detail::
// point_segment_skeleton_thresholds`; pure `Point`/`Line`/`Vec2d` arithmetic apart from one call
// to `Geometry::foot_pt<Vec2d>` (`:412`), same blocker as `point_segment_dr_dl_thresholds` above.
// TODO(port): src/libslic3r/Geometry/VoronoiOffset.cpp:457-648 -- `debug::
// verify_twin_halfedges_successive`/`verify_inside_outside_annotations`/
// `verify_vertices_on_contour`/`verify_signed_distances`/`verify_offset_intersection_points`
// (`namespace debug`, `#ifndef NDEBUG` at `:453`, not `namespace detail`, which closes at
// `:451`); internal self-checks called only from inside `assert(...)` in the seven public
// functions below (themselves all `// TODO(port):`, see the block above the `namespace detail`
// section) -- they read `vd.vertices()`/`.edges()`/`.cells()` directly and need the same live
// diagram.

// PORT STATUS
// source: src/libslic3r/Geometry/VoronoiOffset.cpp
// source: src/libslic3r/Geometry/VoronoiOffset.hpp
// confidence: low
// tier: exact
// oracle: built
