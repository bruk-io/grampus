//! Port of `Arachne/utils/SparseLineGrid.hpp` (task `72-foundation-arachne-grids`, plan step 20,
//! `foundation-23` row of `port/foundation.tsv`): a `SparseGrid` whose elements are indexed by
//! the grid cells their line segment passes through.
//!
//! **Lands `drafted`, not `ported`, and lands no `mod` line -- for the same reason
//! `sparse_grid.rs`'s own module doc states.** `SparseLineGrid<ElemT, Locator>` derives from
//! `SparseGrid<ElemT>` (`:28`), whose own `crate::square_grid::SquareGrid` field does not exist
//! yet (see `sparse_grid.rs`); until `square_grid.rs` lands, this module cannot resolve either,
//! so no `mod` line reaches it and `just port-status` derives this row `drafted`. **Named as a
//! finding in this task's pull request**, alongside `sparse_grid.rs`'s own.

use slic3r_point::point::Point;

use crate::sparse_grid::SparseGrid;

/// `template<class ElemT, class Locator> class SparseLineGrid : public SparseGrid<ElemT>`
/// (`:28`). `Locator` is the vendored template's own functor bound: "must have
/// `std::pair<Point, Point> operator()(const ElemT &elem) const`" (`:24-25`) -- a `Fn(&Elem) ->
/// (Point, Point)`, the idiomatic Rust rendering of a `const`, by-value-returning callable
/// (`docs/PORTING.md`'s idiom map, "A template" -> "with the same bound").
pub struct SparseLineGrid<Elem, Locator>
where
    Locator: Fn(&Elem) -> (Point, Point),
{
    base: SparseGrid<Elem>,
    /// `Locator m_locator;` (`:49`, `protected`).
    locator: Locator,
}

impl<Elem, Locator> SparseLineGrid<Elem, Locator>
where
    Locator: Fn(&Elem) -> (Point, Point),
{
    /// `SparseLineGrid<ElemT, Locator>::SparseLineGrid(coord_t, size_t, float)` (`:52-53`):
    /// forwards to the `SparseGrid<ElemT>` base constructor verbatim, exactly as the vendored
    /// member-initialiser list does.
    pub fn new(cell_size: i64, elem_reserve: usize, max_load_factor: f32, locator: Locator) -> Self {
        SparseLineGrid {
            base: SparseGrid::new(cell_size, elem_reserve, max_load_factor),
            locator,
        }
    }

    /// `SparseLineGrid<ElemT, Locator>::insert(const Elem &elem)` (`:56-68`). The vendored body
    /// is a workaround (its own comment, `:62-63`) for a lambda not being able to reach a
    /// protected base member directly, built with `std::bind` over a second lambda; both are
    /// collapsed here into the one closure Rust's own borrow rules allow directly -- the
    /// faithful behaviour (emplace `elem.clone()` into every grid cell the line passes through)
    /// without reproducing a C++-only workaround that has no Rust counterpart to port
    /// (`docs/PORTING.md`, "Faithfulness, not idiom": the simplest translation that keeps the
    /// meaning, not the C++-specific plumbing around it).
    pub fn insert(&mut self, elem: Elem)
    where
        Elem: Clone,
    {
        let line = (self.locator)(&elem);
        // `SquareGrid::processLineCells` (inherited; `SquareGrid.hpp`'s own declaration) only
        // enumerates grid coordinates -- it never touches `m_grid` itself -- so this collects
        // the visited cells first, then inserts, rather than needing the vendored lambda's
        // `std::bind`-over-`this` workaround for mutating a protected base member from inside a
        // borrowed closure (this module's own doc comment above).
        let mut cells: Vec<Point> = Vec::new();
        self.base
            .square_grid()
            .process_line_cells(line, |grid_loc: Point| {
                cells.push(grid_loc);
                true
            });
        for grid_loc in cells {
            self.base
                .grid_mut()
                .entry(grid_loc)
                .or_default()
                .push(elem.clone());
        }
    }
}

#[cfg(test)]
mod tests {
    // Unreachable until `crate::square_grid::SquareGrid` and `crate::sparse_grid::SparseGrid`'s
    // own `mod` lines land -- see this module's doc and this task's pull request. Left here, in
    // position, so the translation is reviewable now (task.md, "Write each `Sparse*` module's
    // `// PROOF: 1` test in its `#[cfg(test)]` position so the translation is reviewable").
    //
    // Proof path 1 (R19, manifest row `Arachne/utils/SparseLineGrid.hpp`): once reachable, named
    // as scope for the Phase A task that writes `square_grid.rs` (task.md, "Only linear_alg2_d
    // gets them here"): `slic3r_proof::proof_units::sparse_line_grid::path1_insert_and_get_nearby`.
    // PROOF: 1
    //
    // No `#[ignore]` here -- see `sparse_grid.rs`'s own `tests` module doc for why: this file
    // carries no `mod` line, so `cargo test --workspace` never compiles this test today, and an
    // `#[ignore]` would only risk silently skipping a real compile error later.
    #[test]
    fn sparse_line_grid_matches_the_oracle() {
        use super::*;

        let mut grid: SparseLineGrid<i64, _> = SparseLineGrid::new(1000, 0, 1.0, |elem: &i64| {
            (Point::new(0, 0), Point::new(*elem, 0))
        });
        grid.insert(5000);
        let actual = grid.base.get_nearby(Point::new(2000, 0), 1500);

        let oracle = slic3r_proof::proof_units::sparse_line_grid::path1_insert_and_get_nearby(
            1000, 0, 0, 5000, 0, 2000, 0, 1500,
        );
        assert_eq!(
            actual, oracle,
            "SparseLineGrid::insert/getNearby must match the oracle exactly"
        );
    }
}

// PORT STATUS
// source: src/libslic3r/Arachne/utils/SparseLineGrid.hpp
// confidence: medium
// tier: exact
// oracle: built
