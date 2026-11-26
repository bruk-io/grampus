//! Port of `Arachne/utils/PolygonsSegmentIndex.hpp` (task `73-foundation-arachne-indices`, plan
//! step 20, `foundation-22` row of `port/foundation.tsv`): `PolygonsSegmentIndex`, the one
//! concrete (non-template) class in this task's unit set, publicly deriving
//! [`PolygonsPointIndex`](crate::polygons_point_index::PolygonsPointIndex).
//!
//! **Composition, not inheritance**, the same substitution `sparse_grid.rs`'s own module doc
//! states the reason for: Rust has no base-class inheritance, so the base is held as a field
//! (`base` below) rather than flattened in -- unlike `crates/slic3r-point`'s `Point : public
//! Vec2crd`, which flattens because that sibling foundation task hand-designed both types
//! together. Here the base, `PathsPointIndex<'a, Polygons>`, is this task's own sibling module
//! (`polygons_point_index.rs`), already finished, so this module reaches its methods through
//! the `base` field rather than re-declaring them -- a caller that needs `p()`, `advance()`,
//! `is_before()` or the other base methods reaches them as `seg.base.p()`, etc., exactly the way
//! a C++ caller would reach a public base member through the derived object, just spelled with
//! an explicit field instead of an implicit upcast.
//!
//! **No default constructor, for the same reason `polygons_point_index.rs` gives.**
//! `PolygonsSegmentIndex() : PolygonsPointIndex(){}` (`:14`) delegates straight to the base's own
//! zero-argument constructor, which that module does not port (`port/ownership.tsv`'s
//! `borrowed` row makes the base's `polygons` field a live `&'a Paths`, never null) -- see that
//! module's doc for the full reasoning. This task found no in-scope caller of either default
//! constructor (`Arachne/SkeletalTrapezoidation.cpp`, `Arachne/WallToolPaths.cpp`,
//! `Geometry/Voronoi*` are this header's only other referencing files, and each is a fan-out
//! module this task does not read for call sites). **Finding, named in this task's pull
//! request.**
//!
//! **The `boost::polygon::geometry_concept`/`segment_traits<PolygonsSegmentIndex>`
//! specialisations (`:30-46`) are not ported, with no marker site of its own** -- the same
//! treatment `crates/slic3r-polygon/src/polygon.rs`'s own module doc already gives the
//! equivalent `Slic3r::Polygon`/`Polygons` specialisations: template-customisation glue teaching
//! Boost.Polygon's generic algorithms how to read `PolygonsSegmentIndex` as a segment, with no
//! Rust item to carry over and no vendored line this would port (every in-scope Boost.Polygon
//! caller is itself a specific site the manifest tracks under its own file).

use slic3r_point::point::Point;
use slic3r_polygon::polygon::Polygons;

use crate::polygons_point_index::PathsPointIndex;

/// `class PolygonsSegmentIndex : public PolygonsPointIndex` (`:13-22`) -- see the module doc for
/// why the base is a field rather than a flattened set of members.
#[derive(Debug, Clone, Copy)]
pub struct PolygonsSegmentIndex<'a> {
    /// The `PolygonsPointIndex` base. `pub`, matching the vendored `public` inheritance: a
    /// caller reaches an inherited method (`p()`, `advance()`, `is_before()`, ...) through this
    /// field.
    pub base: PathsPointIndex<'a, Polygons>,
}

impl<'a> PolygonsSegmentIndex<'a> {
    /// `PolygonsSegmentIndex(const Polygons *polygons, unsigned int poly_idx, unsigned int
    /// point_idx) : PolygonsPointIndex(polygons, poly_idx, point_idx){}` (`:15`). The
    /// zero-argument constructor (`:14`) is not ported -- see the module doc.
    pub fn new(polygons: &'a Polygons, poly_idx: usize, point_idx: usize) -> Self {
        PolygonsSegmentIndex {
            base: PathsPointIndex::new(polygons, poly_idx, point_idx),
        }
    }

    /// `Point from() const { return PolygonsPointIndex::p(); }` (`:17`).
    pub fn from(&self) -> Point {
        self.base.p()
    }

    /// `Point to() const { return PolygonsSegmentIndex::next().p(); }` (`:19`) -- `next()` here
    /// resolves to the inherited `PolygonsPointIndex::next()` (no override in this class), so
    /// this calls straight through the base field.
    pub fn to(&self) -> Point {
        self.base.next().p()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use slic3r_polygon::polygon::Polygon;

    fn square() -> Polygons {
        vec![Polygon::from_points(vec![
            Point::new(0, 0),
            Point::new(100, 0),
            Point::new(100, 100),
            Point::new(0, 100),
        ])]
    }

    #[test]
    fn from_and_to_are_the_segment_starting_at_point_idx() {
        let polys = square();
        let seg = PolygonsSegmentIndex::new(&polys, 0, 1);
        assert_eq!(seg.from(), Point::new(100, 0));
        assert_eq!(seg.to(), Point::new(100, 100));
    }

    /// Proof path 1 (R19, manifest row `Arachne/utils/PolygonsSegmentIndex.hpp`): drives
    /// `Slic3r::Arachne::PolygonsSegmentIndex::from/to` over a real `Slic3r::Polygons` built
    /// from the same coordinates on both sides. One cell across this unit's single manifest
    /// row, so one `// PROOF: 1` marker.
    // PROOF: 1
    #[test]
    fn polygons_segment_index_matches_the_oracle() {
        let xs = vec![0i64, 100, 100, 0];
        let ys = vec![0i64, 0, 100, 100];
        let points: Vec<Point> = xs
            .iter()
            .zip(&ys)
            .map(|(&x, &y)| Point::new(x, y))
            .collect();
        let polys: Polygons = vec![Polygon::from_points(points)];

        let seg = PolygonsSegmentIndex::new(&polys, 0, 2);
        let actual_from = seg.from();
        let actual_to = seg.to();

        let oracle = slic3r_proof::proof_units::polygons_segment_index::path1_from_to(&xs, &ys, 2);
        assert_eq!(
            (
                actual_from.x(),
                actual_from.y(),
                actual_to.x(),
                actual_to.y()
            ),
            (oracle[0], oracle[1], oracle[2], oracle[3]),
            "PolygonsSegmentIndex::from/to must match the oracle exactly"
        );
    }
}

// PORT STATUS
// source: src/libslic3r/Arachne/utils/PolygonsSegmentIndex.hpp
// confidence: medium
// tier: exact
// oracle: built
