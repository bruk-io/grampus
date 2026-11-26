//! Port of `Arachne/utils/SparsePointGrid.hpp` (task `72-foundation-arachne-grids`, plan step
//! 20, `foundation-23` row of `port/foundation.tsv`): a `SparseGrid` whose elements are indexed
//! by one grid cell, the one their own location falls in.
//!
//! **Lands `drafted`, not `ported`, and lands no `mod` line -- for the same reason
//! `sparse_grid.rs`'s own module doc states.** `SparsePointGrid<ElemT, Locator>` derives from
//! `SparseGrid<ElemT>` (`:26`), whose own `crate::square_grid::SquareGrid` field does not exist
//! yet; until `square_grid.rs` lands, this module cannot resolve either, so no `mod` line
//! reaches it and `just port-status` derives this row `drafted`. **Named as a finding in this
//! task's pull request**, alongside `sparse_grid.rs`'s and `sparse_line_grid.rs`'s own.

use slic3r_point::point::Point;

use crate::sparse_grid::SparseGrid;

/// `template<class ElemT, class Locator> class SparsePointGrid : public SparseGrid<ElemT>`
/// (`:26`, **public** inheritance -- see [`SparsePointGrid::get_nearby`] below). `Locator` is
/// the vendored template's own functor bound: "must have `Point
/// operator()(const ElemT &elem) const`" (`:23-24`) -- a `Fn(&Elem) -> Point`
/// (`docs/PORTING.md`'s idiom map, "A template" -> "with the same bound").
pub struct SparsePointGrid<Elem, Locator>
where
    Locator: Fn(&Elem) -> Point,
{
    base: SparseGrid<Elem>,
    /// `Locator m_locator;` (`:50`, `protected`).
    locator: Locator,
}

impl<Elem, Locator> SparsePointGrid<Elem, Locator>
where
    Locator: Fn(&Elem) -> Point,
{
    /// `SparsePointGrid<ElemT, Locator>::SparsePointGrid(coord_t, size_t, float)` (`:53-54`):
    /// forwards to the `SparseGrid<ElemT>` base constructor verbatim.
    pub fn new(
        cell_size: i64,
        elem_reserve: usize,
        max_load_factor: f32,
        locator: Locator,
    ) -> Self {
        SparsePointGrid {
            base: SparseGrid::new(cell_size, elem_reserve, max_load_factor),
            locator,
        }
    }

    /// `SparsePointGrid<ElemT, Locator>::insert(const Elem &elem)` (`:57-63`): locate the
    /// element, convert its location to a grid coordinate through the inherited
    /// `SquareGrid::toGridPoint`, and emplace into `m_grid` at that one cell. `loc.cast<int64_t>()`
    /// (`:60`) is a no-op here (`Point`'s own fields are already `int64_t`), as it is at every
    /// other such call site this task's units carry.
    pub fn insert(&mut self, elem: Elem) {
        let loc = (self.locator)(&elem);
        let grid_loc = self.base.square_grid().to_grid_point(loc);
        self.base.grid_mut().entry(grid_loc).or_default().push(elem);
    }

    /// `SparseGrid<ElemT>::begin()/end()` (`SparseGrid.hpp:48-51`) are **public** members of the
    /// base, so public inheritance (`:26`) puts them on `SparsePointGrid`'s own surface -- and
    /// `Arachne/WallToolPaths.cpp:874` uses them, range-for'ing a `SparsePointGrid` directly
    /// (`:869`, `using GridT = SparsePointGrid<LineLoc, Locator>`) and reading `pair.second`.
    /// Rust inherits nothing and `base` is private, so that caller needs this explicit
    /// delegation to [`SparseGrid::iter`] -- the same reason [`SparsePointGrid::get_nearby`]
    /// below exists. The order it exposes, and the `// FIDELITY(port):` marker recording it, are
    /// `SparseGrid::iter`'s own; this forwards and adds nothing.
    pub fn iter(&self) -> impl Iterator<Item = (Point, &Elem)> {
        self.base.iter()
    }
}

impl<Elem, Locator> SparsePointGrid<Elem, Locator>
where
    Elem: Clone,
    Locator: Fn(&Elem) -> Point,
{
    /// `SparsePointGrid<ElemT, Locator> : public SparseGrid<ElemT>` (`:26`) is **public**
    /// inheritance, so `SparseGrid<ElemT>::getNearby` (`SparseGrid.hpp:68`) *is* part of
    /// `SparsePointGrid`'s own public surface, even though this class declares no query method
    /// of its own (only `insert`) -- `Arachne/WallToolPaths.cpp:878` calls
    /// `grid.getNearby(loc_here, searching_radius)` on one directly. A thin pass-through, not a
    /// method this type declares of its own; this unit's own proof test (below) and fuzz target
    /// drive the insert/query round-trip through it.
    pub fn get_nearby(&self, query_pt: Point, radius: i64) -> Vec<Elem> {
        self.base.get_nearby(query_pt, radius)
    }
}

#[cfg(test)]
mod tests {
    // Unreachable until `crate::square_grid::SquareGrid` and `crate::sparse_grid::SparseGrid`'s
    // own `mod` lines land -- see this module's doc and this task's pull request. Left here, in
    // position, so the translation is reviewable now (task.md, "Write each `Sparse*` module's
    // `// PROOF: 1` test in its `#[cfg(test)]` position so the translation is reviewable").
    //
    // Proof path 1 (R19, manifest row `Arachne/utils/SparsePointGrid.hpp`): once reachable,
    // named as scope for the Phase A task that writes `square_grid.rs` (task.md, "Only
    // linear_alg2_d gets them here"):
    // `slic3r_proof::proof_units::sparse_point_grid::path1_insert_and_get_nearby`.
    // PROOF: 1
    //
    // No `#[ignore]` here -- see `sparse_grid.rs`'s own `tests` module doc for why: this file
    // carries no `mod` line, so `cargo test --workspace` never compiles this test today, and an
    // `#[ignore]` would only risk silently skipping a real compile error later.
    #[test]
    fn sparse_point_grid_matches_the_oracle() {
        use super::*;

        let mut grid: SparsePointGrid<i64, _> =
            SparsePointGrid::new(1000, 0, 1.0, |elem: &i64| Point::new(*elem, 0));
        grid.insert(5000);
        let actual = grid.get_nearby(Point::new(5100, 0), 500);

        let oracle = slic3r_proof::proof_units::sparse_point_grid::path1_insert_and_get_nearby(
            1000, 0, 5000, 0, 5100, 0, 500,
        );
        assert_eq!(
            actual, oracle,
            "SparsePointGrid::insert/getNearby must match the oracle exactly"
        );
    }
}

// PORT STATUS
// source: src/libslic3r/Arachne/utils/SparsePointGrid.hpp
// confidence: medium
// tier: exact
// oracle: built
