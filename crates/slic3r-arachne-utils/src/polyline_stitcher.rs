//! Port of `Arachne/utils/PolylineStitcher.cpp`/`.hpp` (task `73-foundation-arachne-indices`,
//! plan step 20, `foundation-22` row of `port/foundation.tsv`): `PolylineStitcher<Paths, Path,
//! Junction>::stitch`, the one unit in this task with a real algorithm in it.
//!
//! **Lands `drafted`, not `ported`, and lands no `mod` line -- blocked twice over, each named
//! as a finding in this task's pull request:**
//!
//! 1. `PolylineStitcher.cpp:6` includes `Arachne/utils/ExtrusionLine.hpp`
//!    (`crates/slic3r-arachne-utils/src/extrusion_line.rs`, manifest row `template-heavy: no`
//!    -- a fan-out module, in this same crate, that this task may not write
//!    (`docs/WORKFLOW.md` "Conflict discipline"; task.md, "The fan-out module `polyline_stitcher`
//!    needs"). `PolylineStitcher` is instantiated over `ExtrusionLine` at the pin
//!    (`PolylineStitcher.cpp:18-24`), so this is structural, not a call inside a function body
//!    the shared `// TODO(port):` escape would reach.
//! 2. `PolylineStitcher.hpp:16` includes `Arachne/utils/SparsePointGrid.hpp`
//!    (`crates/slic3r-arachne-utils/src/sparse_point_grid.rs`, task
//!    `72-foundation-arachne-grids`), which that task itself leaves `drafted` with no `mod`
//!    line, for the same reason `sparse_grid.rs`'s own module doc gives
//!    (`crate::square_grid::SquareGrid` does not exist yet).
//!
//! **A third, found by this task: `SparsePointGrid` (task `72-foundation-arachne-grids`)
//! exposes only `get_nearby` (collect every nearby element), not the early-exit
//! `processNearby` form `stitch`'s own closure needs** (`PolylineStitcher.hpp:120-168`,
//! `grid.processNearby(from, max_stitch_distance, [...](const PathsPointIndex<Paths>& nearby)
//! -> bool {...})`: the closure updates `closest`/`closest_distance` on every candidate it is
//! shown, *then* returns `false` to stop the walk once `dist < snap_distance` (`:160-163`) --
//! stopping early changes which candidates are even visited, and so can change which one ends
//! up `closest` relative to visiting every candidate and keeping the global minimum. Collapsing
//! this to `get_nearby`'s own full-collection-then-scan shape would silently diverge from the
//! oracle on exactly the inputs where the early exit matters, which is the "workaround that
//! needs a paragraph of justification is wrong code" shape `docs/WORKFLOW.md`'s rejection rule
//! names -- so this module calls the early-exit form `grid.process_nearby(...)` as if it
//! existed, rather than substituting `get_nearby` and asserting equivalence that does not hold
//! in general. **Finding, named in this task's pull request**, for whichever task next unblocks
//! this module: `SparsePointGrid` needs its own `process_nearby`, delegating to the
//! `SparseGrid::process_nearby` its own public inheritance already exposes on paper
//! (`sparse_point_grid.rs`'s module doc, `get_nearby`'s own doc comment, states the identical
//! "public inheritance puts the base's query method on this type's own surface" reasoning for
//! `get_nearby`; `process_nearby` is the same base method's early-exit sibling,
//! `SparseGrid.hpp:116-120`).
//!
//! **The vendored three-parameter template collapses to one.** `Paths` is instantiated as
//! `Polygons` (`= std::vector<Polygon>`) and `VariableWidthLines` at the pin
//! (`PolylineStitcher.cpp:18-41`), both literally `std::vector<Path>` for their own `Path` --
//! there is no in-scope instantiation where `Paths` is anything but a vector of `Path` -- so
//! this module's `stitch` is generic over `Path` alone, with `Paths` written out as `Vec<Path>`
//! rather than carried as a second type parameter with no distinct value. `Junction` (the
//! vendored template's third parameter, `Polygon`'s own `Point`, `ExtrusionLine`'s own
//! `ExtrusionJunction`) stays as [`StitchPath::Junction`], an associated type, since it
//! genuinely varies independently of `Path` across the two instantiations. **Finding, named in
//! this task's pull request**, for the same reason `half_edge_node.rs`'s parameter collapse is.
//!
//! **`canReverse`/`canConnect`/`isOdd` (`PolylineStitcher.hpp:214-224`) are per-instantiation
//! template specialisations defined only in the `.cpp`** (`PolylineStitcher.cpp:18-41`, two
//! pairs, one per instantiation) **with no generic body of their own** -- the header declares
//! them, never defines them generically, so there is no single Rust body to write once; each
//! concrete `Path` must supply its own. [`StitchPath`] below carries them as required methods
//! for exactly that reason, rather than a default implementation this module would have to
//! invent. This module implements [`StitchPath`] for `Polygon` (`canReverse`: `true`;
//! `canConnect`: `true`; `isOdd`: `false`, straight from `PolylineStitcher.cpp:27-39`) since
//! `Polygon` already exists in this crate; the `ExtrusionLine` instantiation
//! (`PolylineStitcher.cpp:19-25`) is named as scope for the task that writes
//! `extrusion_line.rs`, alongside the `mod` line and the `// PROOF: 1` marker task.md already
//! asks that task to carry.

use slic3r_point::point::Point;
use slic3r_polygon::polygon::Polygon;

use crate::polygons_point_index::PathsPointIndex;

/// A path element that can be read back as a plain [`Point`] -- `Polygon`'s own `Point`,
/// `ExtrusionLine`'s `ExtrusionJunction` (fan-out, not this crate's to implement this for yet).
/// Stands in for `make_point`'s template parameter at `PolylineStitcher.hpp:85,117,134,163,195`
/// (`nearby.p().template cast<int64_t>()`, `make_point(chain.back())`, ...).
pub trait StitchJunction: Copy {
    fn as_point(&self) -> Point;
}

impl StitchJunction for Point {
    fn as_point(&self) -> Point {
        *self
    }
}

/// Hand-designed bound for `PolylineStitcher<Paths, Path, Junction>::stitch`'s own `Path` --
/// see the module doc for why `Paths` itself collapses to `Vec<Self>` and why
/// `can_reverse`/`can_connect`/`is_odd` are required rather than defaulted.
pub trait StitchPath: Clone {
    type Junction: StitchJunction;

    /// `line.size()` at every call site in `stitch` (`:69,99,109,...`).
    fn junction_count(&self) -> usize;
    /// `line[idx]`, read-only.
    fn junction_at(&self, idx: usize) -> Self::Junction;
    /// `chain.reverse()` (`:91`).
    fn reverse(&mut self);
    /// `chain.polylineLength()` (`:94`).
    fn polyline_length(&self) -> i64;
    /// `chain.insert(chain.end(), start_pos, (*closest.polygons)[closest.poly_idx].end())`
    /// (`:177`): append `other`'s junctions from `start_idx` (inclusive) to its end, in order.
    fn extend_from_front(&mut self, other: &Self, start_idx: usize);
    /// `chain.insert(chain.end(), start_pos, (*closest.polygons)[closest.poly_idx].rend())`
    /// (`:186`): append `other`'s junctions from `start_idx_from_end` (inclusive, counted from
    /// the end) back to its front, in reverse order.
    fn extend_from_back(&mut self, other: &Self, start_idx_from_end: usize);

    /// `static bool canReverse(const PathsPointIndex<Paths> &polyline);` (`:214-215`) -- no
    /// generic body; see the module doc.
    fn can_reverse(polyline: &PathsPointIndex<'_, Vec<Self>>) -> bool;
    /// `static bool canConnect(const Path &a, const Path &b);` (`:217-220`).
    fn can_connect(a: &Self, b: &Self) -> bool;
    /// `static bool isOdd(const Path &line);` (`:222`).
    fn is_odd(line: &Self) -> bool;
}

/// `template<> bool PolylineStitcher<Polygons, Polygon, Point>::canReverse(...)` /
/// `canConnect` / `isOdd` (`PolylineStitcher.cpp:27,33,39`): the plain-polygon instantiation
/// always allows reversal and connection, and no polygon is "odd".
impl StitchPath for Polygon {
    type Junction = Point;

    fn junction_count(&self) -> usize {
        self.points.len()
    }
    fn junction_at(&self, idx: usize) -> Point {
        self.points[idx]
    }
    fn reverse(&mut self) {
        self.points.reverse();
    }
    /// `polylineLength()`: `Polygon` has no ported method of this name in
    /// `crates/slic3r-polygon/src/polygon.rs` (that crate's own `length()` is the *closed*-ring
    /// perimeter, `MultiPoint::length()` plus the closing edge -- a different vendored method,
    /// `Polygon.hpp`'s own `length()`, not this one); summed directly here as the open-polyline
    /// length `MultiPoint::length()` already computes for a non-closed walk, since this crate
    /// may not add a dependency to reach that trait method (see
    /// `crates/slic3r-arachne-utils/src/polygons_point_index.rs`'s module doc, same
    /// `slic3r-multipoint` dependency gap).
    fn polyline_length(&self) -> i64 {
        let mut total: i64 = 0;
        for i in 1..self.points.len() {
            let a = self.points[i - 1];
            let b = self.points[i];
            // PORT SRC: src/libslic3r/Arachne/utils/PolylineStitcher.hpp:94
            let dx = b.x().wrapping_sub(a.x());
            let dy = b.y().wrapping_sub(a.y());
            total = total.wrapping_add(((dx * dx + dy * dy) as f64).sqrt() as i64);
        }
        total
    }
    fn extend_from_front(&mut self, other: &Self, start_idx: usize) {
        self.points.extend_from_slice(&other.points[start_idx..]);
    }
    fn extend_from_back(&mut self, other: &Self, start_idx_from_end: usize) {
        let take = other.points.len() - start_idx_from_end;
        self.points
            .extend(other.points[..=start_idx_from_end].iter().rev().take(take + 1));
    }

    // PORT SRC: src/libslic3r/Arachne/utils/PolylineStitcher.cpp:28
    fn can_reverse(_polyline: &PathsPointIndex<'_, Vec<Self>>) -> bool {
        true
    }
    // PORT SRC: src/libslic3r/Arachne/utils/PolylineStitcher.cpp:34
    fn can_connect(_a: &Self, _b: &Self) -> bool {
        true
    }
    // PORT SRC: src/libslic3r/Arachne/utils/PolylineStitcher.cpp:40
    fn is_odd(_line: &Self) -> bool {
        false
    }
}

/// `PolylineStitcher<Paths, Path, Junction>::stitch(...)` (`:34-209`). See the module doc for
/// why this is unreachable and for the generic collapse from the vendored three parameters to
/// one.
pub fn stitch<Path: StitchPath>(
    lines: &[Path],
    result_lines: &mut Vec<Path>,
    result_polygons: &mut Vec<Path>,
    max_stitch_distance: i64,
    snap_distance: i64,
) {
    // `if (lines.empty()) return;` (`:36-37`).
    if lines.is_empty() {
        return;
    }

    let lines_vec: Vec<Path> = lines.to_vec();

    // `SparsePointGrid<PathsPointIndex<Paths>, PathsPointIndexLocator<Paths>> grid(...)`
    // (`:39`) -- see the module doc, finding 2: `SparsePointGrid` does not yet expose the
    // early-exit `processNearby` this closure needs, so `grid` below is written against the
    // method it should have, not the one it does.
    // TODO(port): crates/slic3r-arachne-utils/src/sparse_point_grid.rs -- add a
    // `process_nearby` delegating to `SparseGrid::process_nearby` (`SparseGrid.hpp:116-120`),
    // the early-exit sibling of the `get_nearby` it already exposes; see this module's doc,
    // finding 2, for why `get_nearby` is not an equivalent substitute here.
    let mut grid = crate::sparse_point_grid::SparsePointGrid::new(
        max_stitch_distance,
        lines_vec.len() * 2,
        // PORT SRC: src/libslic3r/Arachne/utils/SparsePointGrid.hpp:38
        1.0,
        |idx: &PathsPointIndex<'_, Vec<Path>>| idx.p(),
    );

    // `for (size_t line_idx = 0; line_idx < lines.size(); line_idx++) { ... grid.insert(...); }`
    // (`:42-47`).
    for line_idx in 0..lines_vec.len() {
        let line = &lines_vec[line_idx];
        grid.insert(PathsPointIndex::new(&lines_vec, line_idx, 0));
        grid.insert(PathsPointIndex::new(
            &lines_vec,
            line_idx,
            line.junction_count() - 1,
        ));
    }

    let mut processed = vec![false; lines_vec.len()];

    for line_idx in 0..lines_vec.len() {
        if processed[line_idx] {
            continue;
        }
        processed[line_idx] = true;
        let line = &lines_vec[line_idx];
        let mut should_close = Path::is_odd(line);

        let mut chain = line.clone();
        let mut closest_is_closing_polygon = false;

        for go_in_reverse_direction in [false, true] {
            if go_in_reverse_direction {
                chain.reverse();
            }
            let mut chain_length = chain.polyline_length();

            loop {
                let from = chain.junction_at(chain.junction_count() - 1).as_point();

                let mut closest: Option<PathsPointIndex<'_, Vec<Path>>> = None;
                let mut closest_distance = i64::MAX;

                // `grid.processNearby(from, max_stitch_distance, [...] -> bool {...})`
                // (`:120-168`) -- see the module doc, finding 2, for `process_nearby` vs.
                // `get_nearby`.
                grid.process_nearby(from, max_stitch_distance, |nearby: &PathsPointIndex<
                    '_,
                    Vec<Path>,
                >| {
                    let mut is_closing_segment = false;
                    let np = nearby.p();
                    // PORT SRC: src/libslic3r/Arachne/utils/PolylineStitcher.hpp:131
                    let dx = np.x().wrapping_sub(from.x());
                    let dy = np.y().wrapping_sub(from.y());
                    let mut dist = ((dx * dx + dy * dy) as f64).sqrt() as i64;
                    if dist > max_stitch_distance {
                        return true; // keep looking
                    }
                    let chain_front = chain.junction_at(0).as_point();
                    let fdx = np.x().wrapping_sub(chain_front.x());
                    let fdy = np.y().wrapping_sub(chain_front.y());
                    if (fdx * fdx + fdy * fdy) < snap_distance * snap_distance {
                        // PORT SRC: src/libslic3r/Arachne/utils/PolylineStitcher.hpp:117
                        if chain_length + dist < 3 * max_stitch_distance || chain.junction_count() <= 2
                        {
                            return true; // look for a better next line
                        }
                        is_closing_segment = true;
                        if !should_close {
                            // PORT SRC: src/libslic3r/Arachne/utils/PolylineStitcher.hpp:125
                            dist += slic3r_point::point::scaled_coord(0.01f64);
                        } else {
                            // PORT SRC: src/libslic3r/Arachne/utils/PolylineStitcher.hpp:131
                            dist -= slic3r_point::point::scaled_coord(0.01f64);
                        }
                    } else if processed[nearby.poly_idx] {
                        return true; // keep looking for a connection
                    }
                    let mut nearby_would_be_reversed = nearby.point_idx != 0;
                    nearby_would_be_reversed = nearby_would_be_reversed != go_in_reverse_direction;
                    if !Path::can_reverse(nearby) && nearby_would_be_reversed {
                        return true;
                    }
                    if !Path::can_connect(&chain, &lines_vec[nearby.poly_idx]) {
                        return true;
                    }
                    if dist < closest_distance {
                        closest_distance = dist;
                        closest = Some(*nearby);
                        closest_is_closing_polygon = is_closing_segment;
                    }
                    if dist < snap_distance {
                        return false; // stop looking for alternatives
                    }
                    true
                });

                let Some(closest_idx) = closest else {
                    break;
                };
                if closest_is_closing_polygon {
                    break;
                }

                let closest_p = closest_idx.p();
                // PORT SRC: src/libslic3r/Arachne/utils/PolylineStitcher.hpp:171
                let sdx = chain
                    .junction_at(chain.junction_count() - 1)
                    .as_point()
                    .x()
                    .wrapping_sub(closest_p.x());
                let sdy = chain
                    .junction_at(chain.junction_count() - 1)
                    .as_point()
                    .y()
                    .wrapping_sub(closest_p.y());
                let segment_dist = ((sdx * sdx + sdy * sdy) as f64).sqrt() as i64;
                debug_assert!(
                    // PORT SRC: src/libslic3r/Arachne/utils/PolylineStitcher.hpp:171
                    segment_dist <= max_stitch_distance + slic3r_point::point::scaled_coord(0.01f64)
                );
                let old_len = chain.junction_count();
                let closest_line = &lines_vec[closest_idx.poly_idx];
                if closest_idx.point_idx == 0 {
                    let start = if segment_dist < snap_distance { 1 } else { 0 };
                    chain.extend_from_front(closest_line, start);
                } else {
                    let start_from_end = if segment_dist < snap_distance { 1 } else { 0 };
                    chain.extend_from_back(closest_line, start_from_end);
                }
                for i in old_len..chain.junction_count() {
                    let a = chain.junction_at(i - 1).as_point();
                    let b = chain.junction_at(i).as_point();
                    let dx = b.x().wrapping_sub(a.x());
                    let dy = b.y().wrapping_sub(a.y());
                    chain_length =
                        chain_length.wrapping_add(((dx * dx + dy * dy) as f64).sqrt() as i64);
                }
                should_close = should_close && !Path::is_odd(closest_line);
                debug_assert!(!processed[closest_idx.poly_idx]);
                processed[closest_idx.poly_idx] = true;
            }

            if closest_is_closing_polygon {
                if go_in_reverse_direction {
                    chain.reverse();
                }
                break;
            }
        }

        if closest_is_closing_polygon {
            result_polygons.push(chain);
        } else {
            let ppi_here = PathsPointIndex::new(&lines_vec, line_idx, 0);
            if !Path::can_reverse(&ppi_here) {
                chain.reverse();
            }
            result_lines.push(chain);
        }
    }
}

#[cfg(test)]
mod tests {
    // Unreachable until `extrusion_line.rs` and `sparse_point_grid.rs`'s own `mod` lines land
    // (and a `process_nearby` method on `SparsePointGrid`, this module's own finding) -- see
    // the module doc. Left here, in position, so the translation is reviewable now, per
    // task.md: "Write its `// PROOF: 1` test in its `#[cfg(test)]` position so the translation
    // is reviewable, and say in the pull request that it is unreachable until the `mod` line
    // lands."
    //
    // Proof path 1 (R19, manifest rows `Arachne/utils/PolylineStitcher.cpp` and `.hpp`): once
    // reachable, drives `Slic3r::Arachne::PolylineStitcher<Polygons, Polygon,
    // Point>::stitch` over a pair of open polylines that should join end to end, through a
    // shim this task does not add (see the module doc) -- named as scope for the task that
    // unblocks this module.
    // PROOF: 1
    #[test]
    fn polyline_stitcher_matches_the_oracle() {
        use super::*;

        let a = Polygon::from_points(vec![Point::new(0, 0), Point::new(100, 0)]);
        let b = Polygon::from_points(vec![Point::new(100, 0), Point::new(200, 0)]);
        let lines = vec![a, b];
        let mut result_lines = Vec::new();
        let mut result_polygons = Vec::new();

        stitch(
            &lines,
            &mut result_lines,
            &mut result_polygons,
            slic3r_point::point::scaled_coord(0.1f64),
            slic3r_point::point::scaled_coord(0.01f64),
        );

        let oracle = slic3r_proof::proof_units::polyline_stitcher::path1_stitch_two_open_lines(
            0, 0, 100, 0, 100, 0, 200, 0,
        );
        assert_eq!(
            result_lines.len() + result_polygons.len(),
            oracle.len() as usize,
            "PolylineStitcher::stitch's output count must match the oracle exactly"
        );
    }
}

// PORT STATUS
// source: src/libslic3r/Arachne/utils/PolylineStitcher.cpp
// source: src/libslic3r/Arachne/utils/PolylineStitcher.hpp
// confidence: low
// tier: exact
// oracle: built
