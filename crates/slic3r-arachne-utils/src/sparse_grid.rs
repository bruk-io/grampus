//! Port of `Arachne/utils/SparseGrid.hpp` (task `72-foundation-arachne-grids`, plan step 20,
//! `foundation-23` row of `port/foundation.tsv`): the sparse-grid base template every other
//! `Arachne/utils/` grid in this crate derives from.
//!
//! **Lands `drafted`, not `ported`, and lands no `mod` line.** `SparseGrid<ElemT>` derives from
//! `SquareGrid` (`SparseGrid.hpp:27`), whose own manifest row is `template-heavy: no` -- a
//! fan-out row, in this same crate, this task does not write (task.md, "The fan-out base class
//! three of these four modules need": `Arachne/utils/SquareGrid.hpp`/`.cpp`, target module
//! `crates/slic3r-arachne-utils/src/square_grid.rs`). Writing `square_grid.rs` from this branch
//! would be exactly the collision `docs/WORKFLOW.md` "Conflict discipline" exists to prevent,
//! and would overwrite `34-foundation-spike`'s own corrected `template-heavy` column. Until a
//! later (Phase A/B) task writes it, `crate::square_grid::SquareGrid` below does not exist, so
//! no `mod` line reaches this file: `cargo build --workspace` never compiles it and
//! `cargo fmt --check` never visits it (`docs/WORKFLOW.md`, "The two phases"), and
//! `just port-status` derives this row `drafted`. **Named as a finding in this task's pull
//! request**, together with the `mod` line, the `// PROOF: 1` marker and the `required` fuzz
//! target the Phase A task that writes `square_grid.rs` should add.
//!
//! **Composition, not inheritance.** Rust has no base-class inheritance; `crates/slic3r-point`'s
//! own `Point : public Vec2crd` (`SparseGrid`'s sibling foundation task, `56-foundation-point`)
//! flattens its base's two fields directly, because that task hand-designed *both* `Point` and
//! `Vec2crd` together. This task cannot do the same for `SquareGrid`, which it is forbidden to
//! write, so `SparseGrid` instead holds one as a field (`square_grid` below) and reaches its
//! methods through it -- the ordinary Rust translation of non-virtual inheritance when the base
//! is owned elsewhere.
//!
//! **`GridMap` (`:34`, `std::unordered_multimap<GridPoint, Elem, PointHash>`).** `GridPoint` is
//! `SquareGrid::GridPoint`, itself `Point` (`SquareGrid.hpp:38`, `using GridPoint = Point;`) --
//! not re-aliased here, since that alias belongs to `square_grid.rs`'s own unit; this module
//! uses `slic3r_point::point::Point` directly. `std::unordered_multimap` is not one of the five
//! hash-container shapes `docs/PORTING.md`'s "Insertion-ordered containers" rule converts to
//! `indexmap::IndexMap`/`IndexSet` (`std::unordered_map`, `std::unordered_set`,
//! `ankerl::unordered_dense::map`, `tbb::concurrent_unordered_{map,set}`) -- and
//! `port/reports/order-containers.md` carries no row for this site either, since its audit
//! covers only those five shapes. So this site is outside that rule's reach, and the plain
//! `HashMap` shape the type map's silence leaves it at is used: one bucket (`Vec<Elem>`) per
//! `GridPoint`, which is also the only way to keep more than one element per key at all, since
//! Rust's std has no multimap.
//!
//! **Finding, named in this task's pull request.** `docs/PORTING.md`'s own "Insertion-ordered
//! containers" section states that `34-foundation-spike` already added `indexmap` as a workspace
//! dependency; measured directly against the committed tree (not assumed): no crate's
//! `Cargo.toml` and no `[workspace.dependencies]` table in the root `Cargo.toml` carries it, so
//! even a module that *did* want `IndexMap` has no sanctioned way to depend on it today, and a
//! port task may add none (`docs/WORKFLOW.md`, "Conflict discipline"). Immaterial to this
//! module's own HashMap choice above (`unordered_multimap` is not one of the five shapes that
//! rule reaches regardless), but a real gap the next task that does need `IndexMap`/`IndexSet`
//! will hit.

use std::collections::HashMap;

use slic3r_point::point::Point;

/// `template<class ElemT> class SparseGrid : public SquareGrid` (`SparseGrid.hpp:27`): an
/// abstract base with no insertion method of its own (`SparsePointGrid`/`SparseLineGrid` add
/// theirs) -- generic over `Elem`, the same bound the vendored template carries
/// (`docs/PORTING.md`'s idiom map, "A template" -> "A generic function or type with the same
/// bound").
pub struct SparseGrid<Elem> {
    /// The `SquareGrid` base (`SparseGrid.hpp:27`), held by composition -- see the module doc.
    square_grid: crate::square_grid::SquareGrid,
    /// `GridMap m_grid;` (`:90`, `protected`), `std::unordered_multimap<GridPoint, Elem,
    /// PointHash>` (`:34`) -- see the module doc for why this is `HashMap<Point, Vec<Elem>>`.
    grid: HashMap<Point, Vec<Elem>>,
}

impl<Elem> SparseGrid<Elem> {
    /// `SparseGrid<ElemT>::SparseGrid(coord_t cell_size, size_t elem_reserve, float
    /// max_load_factor)` (`:94-99`). `max_load_factor` is an `std::unordered_multimap` tuning
    /// knob (rehash threshold) with no `HashMap` counterpart; kept as a parameter for a
    /// faithful signature, acted on only where `HashMap` has an equivalent (`elem_reserve`, via
    /// `HashMap::with_capacity`).
    // TODO(port): src/libslic3r/Arachne/utils/SquareGrid.hpp -- `SquareGrid(coord_t)`'s own
    // constructor (`SquareGrid.cpp`, template-heavy: no, not this task's to write); this calls
    // the name and signature `square_grid.rs`'s Phase A/B task is expected to give it, named as
    // scope for that task in this task's pull request.
    pub fn new(cell_size: i64, elem_reserve: usize, max_load_factor: f32) -> Self {
        let _ = max_load_factor; // no HashMap equivalent -- see the doc above.
        SparseGrid {
            square_grid: crate::square_grid::SquareGrid::new(cell_size),
            grid: if elem_reserve != 0 {
                HashMap::with_capacity(elem_reserve)
            } else {
                HashMap::new()
            },
        }
    }

    /// `SparseGrid<ElemT>::getNearby(const Point &query_pt, coord_t radius) const`
    /// (`:122-130`): collects every element `processNearby` below visits, in visiting order.
    pub fn get_nearby(&self, query_pt: Point, radius: i64) -> Vec<Elem>
    where
        Elem: Clone,
    {
        let mut ret = Vec::new();
        self.process_nearby(query_pt, radius, |elem: &Elem| {
            ret.push(elem.clone());
            true
        });
        ret
    }

    /// `SparseGrid<ElemT>::processNearby(const Point &query_pt, coord_t radius, const
    /// std::function<bool(const ElemT &)> &process_func) const` (`:117-120`): delegates to the
    /// `SquareGrid` base's own `processNearby` (cell-coordinate iteration), binding each visited
    /// cell to [`process_from_cell`] below -- the same closure-over-`this` shape the vendored
    /// lambda at `:118` uses.
    pub fn process_nearby(
        &self,
        query_pt: Point,
        radius: i64,
        mut process_func: impl FnMut(&Elem) -> bool,
    ) -> bool {
        self.square_grid
            .process_nearby(query_pt, radius, |grid_pt: Point| {
                self.process_from_cell(grid_pt, &mut process_func)
            })
    }

    /// `SparseGrid<ElemT>::processFromCell(const GridPoint &grid_pt, const
    /// std::function<bool(const Elem &)> &process_func) const` (`:106-111`, `protected`):
    /// `equal_range` over the multimap's one key's bucket -- the faithful translation for this
    /// `HashMap<Point, Vec<Elem>>` shape is a lookup by key followed by an in-order walk of its
    /// bucket.
    fn process_from_cell(
        &self,
        grid_pt: Point,
        process_func: &mut impl FnMut(&Elem) -> bool,
    ) -> bool {
        if let Some(bucket) = self.grid.get(&grid_pt) {
            for elem in bucket {
                if !process_func(elem) {
                    return false;
                }
            }
        }
        true
    }

    /// `iterator begin() { return m_grid.begin(); }` / `end()` and their `const` overloads
    /// (`:45-48`): no single `GridMap::iterator` counterpart exists for `HashMap<Point,
    /// Vec<Elem>>`'s two-level shape, so this is a flattened iterator over `(GridPoint, &Elem)`
    /// pairs -- the same pairs a multimap iterator yields, in this map's (not the multimap's)
    /// order.
    pub fn iter(&self) -> impl Iterator<Item = (Point, &Elem)> {
        self.grid
            .iter()
            .flat_map(|(grid_pt, bucket)| bucket.iter().map(move |elem| (*grid_pt, elem)))
    }

    /// `protected: GridMap m_grid;` (`:90`), reached by `SparseLineGrid`/`SparsePointGrid`'s own
    /// `insert` (`SparseGrid<ElemT>::m_grid.emplace(...)`). `pub(crate)`, not `pub`, is Rust's
    /// closest analogue to C++ `protected` here: visible to this crate's own derived modules,
    /// not to a caller outside it.
    pub(crate) fn grid_mut(&mut self) -> &mut HashMap<Point, Vec<Elem>> {
        &mut self.grid
    }

    /// The `SquareGrid` base, for `SparseLineGrid`/`SparsePointGrid`'s own `insert`, which calls
    /// `SparseGrid<ElemT>::processLineCells`/`toGridPoint` -- both inherited from `SquareGrid`,
    /// not declared on `SparseGrid` itself.
    pub(crate) fn square_grid(&self) -> &crate::square_grid::SquareGrid {
        &self.square_grid
    }
}

#[cfg(test)]
mod tests {
    // Unreachable until `crate::square_grid::SquareGrid` exists and a `mod square_grid;` line
    // (and this unit's own `mod sparse_grid;` line) land -- see the module doc and this task's
    // pull request. Left here, in position, so the translation is reviewable now rather than
    // written only when the module compiles (task.md, "The fan-out base class...").
    //
    // Proof path 1 (R19, manifest row `Arachne/utils/SparseGrid.hpp`): once reachable, this
    // calls through a shim this task does *not* add (task.md: "Only linear_alg2_d gets them
    // here"), named as scope for the Phase A task that writes `square_grid.rs`:
    // `slic3r_proof::proof_units::sparse_grid::path1_get_nearby`, over
    // `crates/slic3r-ffi/cxx/proof_units/sparse_grid.{h,cpp}` and
    // `crates/slic3r-ffi/src/proof_units/sparse_grid.rs`, instantiating `SparseGrid<ElemT>`
    // over a concrete, FFI-plain `ElemT` such as `int64_t` on the C++ side.
    // PROOF: 1
    //
    // No `#[ignore]` here: this file carries no `mod` line (see the module doc above), so
    // `cargo test --workspace` never compiles this test at all today -- an `#[ignore]` would do
    // nothing now and would silently skip a real compile error later, the moment a `mod` line
    // does reach this file before `square_grid.rs`'s own API actually exists, which is exactly
    // the "a green result that is not real" shape this project's honesty rules name. Loud and
    // unreachable is preferred over quiet and unreachable.
    #[test]
    fn sparse_grid_matches_the_oracle() {
        use super::*;

        let mut grid: SparseGrid<i64> = SparseGrid::new(1000, 0, 1.0);
        grid.grid_mut()
            .entry(Point::new(0, 0))
            .or_default()
            .push(42);
        let actual = grid.get_nearby(Point::new(100, 100), 2000);

        let oracle = slic3r_proof::proof_units::sparse_grid::path1_get_nearby(1000, 0, 0, 42, 100, 100, 2000);
        assert_eq!(actual, oracle, "SparseGrid::getNearby must match the oracle exactly");
    }
}

// PORT STATUS
// source: src/libslic3r/Arachne/utils/SparseGrid.hpp
// confidence: medium
// tier: exact
// oracle: built
