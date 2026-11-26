//! Port of `Arachne/utils/PolygonsPointIndex.hpp` (task `73-foundation-arachne-indices`, plan
//! step 20, `foundation-22` row of `port/foundation.tsv`): `PathsPointIndex<Paths>`, the
//! template every Arachne point-walking index in this task builds on, plus its two locator
//! helper types and its free `make_point` identity function.
//!
//! **The generic bound this header's template needs, hand-designed here.** `PathsPointIndex`
//! is used only through `(*polygons)[poly_idx][point_idx]` and `(*polygons)[poly_idx].size()`
//! (`:32-123`); no existing foundation crate already expresses "an indexable sequence of
//! indexable point sequences" as a trait, and this crate's generated `Cargo.toml`
//! (`34-foundation-spike`) has no dependency on `slic3r-multipoint`, whose
//! [`MultiPoint`](slic3r_multipoint::multi_point::MultiPoint) trait would otherwise be the
//! obvious fit -- a port task may add no dependency (`docs/WORKFLOW.md`, "Conflict
//! discipline"). [`PointPath`] below is designed in this task instead, with `Polygon`'s own
//! public `points: Points` field as its one implementation (via the orphan rule: a local trait
//! may be implemented for a foreign type from this crate, so `impl PointPath for
//! slic3r_polygon::polygon::Polygon` here needs no edit to `polygon.rs` itself, which this task
//! does not own). `Arachne/utils/PolylineStitcher.hpp`'s own instantiation of this template over
//! `VariableWidthLines`/`ExtrusionLine` (see `polyline_stitcher.rs`'s module doc) would need a
//! second `impl PointPath for ExtrusionLine` once that fan-out type exists; this task leaves
//! that as a finding for the task that writes `extrusion_line.rs`, named in the pull request.
//!
//! **No default-constructed, "uninitialised" state is ported, and this is a deliberate gap, not
//! an oversight.** `port/ownership.tsv`'s `borrowed` row for `PathsPointIndex::polygons`
//! (`:32`) gives the field's Rust type as `&'a Paths` verbatim -- a live reference, never
//! null -- so `PathsPointIndex()` (`:45`, preceded by its own doc comment at `:38-44` admitting
//! "you can't ever make this initialisation useful") and `initialized()` (`:71`, `return
//! polygons;`, i.e. "is this the non-null state") have no faithful counterpart here: writing
//! `initialized()` to always return `true` would be a method whose body contradicts what its
//! name and its source say, which is exactly the "green result that is not real" this project's
//! honesty rules name, so neither is ported. **Finding, named in this task's pull request.**
//! The one in-scope call site that needs the zero-then-maybe-filled shape,
//! `PolylineStitcher.hpp:102,163` (`PathsPointIndex<Paths> closest; ... if
//! (!closest.initialized()) ...`), is this task's own `polyline_stitcher.rs`, which represents
//! it as `Option<PathsPointIndex<'a, Paths>>` instead -- see that module. The other,
//! `PolygonsSegmentIndex()` (`PolygonsSegmentIndex.hpp:23`), is this task's own
//! `polygons_segment_index.rs`, which omits the zero-argument constructor for the identical
//! reason -- see that module's doc.
//!
//! **`operator==`/`operator!=` compare pointer identity, not value, so neither is derived.**
//! `bool operator==(const PathsPointIndex &other) const { return polygons == other.polygons &&
//! poly_idx == other.poly_idx && point_idx == other.point_idx; }` (`:84-87`) compares the raw
//! `polygons` *pointer*, which `#[derive(PartialEq)]` cannot reproduce: on a `&'a Paths`,
//! Rust's `==` is `Paths: PartialEq`'s deep value comparison, not pointer identity, unless
//! written out with `std::ptr::eq` explicitly -- which [`PartialEq for PathsPointIndex`] below
//! does. `operator!=` (`:88-91`) is `PartialEq`'s derived `ne`, needing no separate impl.
//!
//! **`operator<` (`:92-95`, `this->p() < other.p()`) is ported as the inherent method
//! [`PathsPointIndex::is_before`]**, not a `PartialOrd`/`Ord` impl: this type's equality
//! (identity, above) and this ordering (point value) are independent on purpose in the vendored
//! source itself -- the same inconsistency a derived `Ord` would paper over by assuming the two
//! agree. `Point`'s own `Ord` (`crates/slic3r-point/src/point.rs:585-596`, lexicographic `x`
//! then `y`, `// PORT SRC: src/libslic3r/Point.hpp:274`) is reused rather than re-derived; this
//! is not a second order choice of this module's own, so it carries no `// FIDELITY(port):` of
//! its own -- the marker already sits on `Point::cmp`, where the order was actually chosen.
//!
//! **`std::hash<PolygonsPointIndex>` (`:169-176`) hashes `PointHash{}(lpi.p())` -- `PointHash`
//! is not ported.** `crates/slic3r-point/src/point.rs`'s own module doc states "`PointHash` is
//! not ported as a standalone hasher" (it exists upstream only to key one `unordered_multimap`
//! that module ports differently). No in-scope caller of *this* unit's own header needs
//! `PolygonsPointIndex` as a hash-map key, so this task leaves the specialization unported with
//! a `// TODO(port):` marker rather than substituting Rust's field-wise derived `Hash` (which
//! would hash by identity/indices, not by point value, and so would not even be
//! self-consistent with the identity-based `PartialEq` above the way the vendored pair,
//! value-hash plus identity-equality, already is not -- substituting here would add a *second*,
//! different inconsistency rather than porting the one that exists). **Finding, named in this
//! task's pull request**, for whichever fan-out task first needs this as a hash-map key.

use slic3r_point::point::Point;
use slic3r_polygon::polygon::{Polygon, Polygons};

/// `inline const Point &make_point(const Point &p) { return p; }` (`:20`): an identity hook so
/// templated algorithms can be written against "a point, or a thing convertible to one" --
/// `Point` is `Copy` here, so the by-reference C++ signature becomes by-value.
pub fn make_point(p: Point) -> Point {
    p
}

/// Hand-designed bound for `PathsPointIndex<Paths>`'s own needs -- see the module doc. One
/// `Paths` element (what `(*polygons)[poly_idx]` yields) must report how many points it holds
/// and return the point at an index.
pub trait PointPath {
    /// `(*polygons)[poly_idx].size()` at every call site below.
    fn point_count(&self) -> usize;
    /// `(*polygons)[poly_idx][point_idx]` at every call site below.
    fn point_at(&self, idx: usize) -> Point;
}

impl PointPath for Polygon {
    fn point_count(&self) -> usize {
        self.points.len()
    }
    fn point_at(&self, idx: usize) -> Point {
        self.points[idx]
    }
}

/// `template<typename Paths> class PathsPointIndex` (`:25-131`) -- see the module doc for the
/// constructor and the methods this port does not carry over.
///
/// `Clone`/`Copy` are implemented by hand below, not derived: `#[derive(Clone, Copy)]` on a
/// generic struct adds an implicit `Paths: Clone`/`Paths: Copy` bound on the *type parameter*,
/// which `Polygons` (`= Vec<Polygon>`) does not satisfy, even though every field actually
/// stored here (`&'a Paths`, `usize`) is unconditionally `Copy` regardless of whether `Paths`
/// itself is.
#[derive(Debug)]
pub struct PathsPointIndex<'a, Paths> {
    /// `const Paths* polygons;` (`:32`) -- `port/ownership.tsv`'s `borrowed` row, taken
    /// verbatim.
    pub polygons: &'a Paths,
    /// `unsigned int poly_idx;` (`:34`). No type-map row covers `unsigned int`; `usize` is used
    /// (not `u32`) because every use of this field is an index into a Rust-owned `Vec` (never
    /// arithmetic sensitive to a 32-bit width), matching the index type the indexing operator
    /// below already requires.
    pub poly_idx: usize,
    /// `unsigned int point_idx;` (`:36`), same reasoning as `poly_idx` above.
    pub point_idx: usize,
}

/// `PathsPointIndex(const PathsPointIndex &original) = default;` (`:58`) -- a field-wise copy,
/// unconditional in every field regardless of `Paths`; see the struct doc for why this is
/// hand-written rather than `#[derive(Clone)]`.
impl<'a, Paths> Clone for PathsPointIndex<'a, Paths> {
    fn clone(&self) -> Self {
        PathsPointIndex {
            polygons: self.polygons,
            poly_idx: self.poly_idx,
            point_idx: self.point_idx,
        }
    }
}
impl<'a, Paths> Copy for PathsPointIndex<'a, Paths> {}

impl<'a, Paths> PathsPointIndex<'a, Paths> {
    /// `PathsPointIndex(const Paths *polygons, unsigned int poly_idx, unsigned int point_idx)`
    /// (`:53`). The zero-argument constructor (`:45`) is not ported -- see the module doc.
    pub fn new(polygons: &'a Paths, poly_idx: usize, point_idx: usize) -> Self {
        PathsPointIndex {
            polygons,
            poly_idx,
            point_idx,
        }
    }
}

impl<'a, Paths> PathsPointIndex<'a, Paths>
where
    Paths: std::ops::Index<usize>,
    Paths::Output: PointPath,
{
    /// `Point p() const { if (!polygons) return {0, 0}; return make_point((*polygons)[poly_idx][point_idx]); }`
    /// (`:60-66`). The `!polygons` null guard has no counterpart -- see the module doc, "No
    /// default-constructed ... state is ported".
    pub fn p(&self) -> Point {
        make_point(self.polygons[self.poly_idx].point_at(self.point_idx))
    }

    /// `bool operator<(const PathsPointIndex &other) const { return this->p() < other.p(); }`
    /// (`:92-95`) -- see the module doc for why this is a named method, not `PartialOrd`.
    pub fn is_before(&self, other: &Self) -> bool {
        self.p() < other.p()
    }

    /// `PathsPointIndex &operator++()` (`:104-108`): advance, wrapping at the polygon's own
    /// point count.
    pub fn advance(&mut self) {
        // PORT SRC: src/libslic3r/Arachne/utils/PolygonsPointIndex.hpp:106
        let len = self.polygons[self.poly_idx].point_count();
        // PORT NOTE: src/libslic3r/Arachne/utils/PolygonsPointIndex.hpp:106 -- `point_idx =
        // (point_idx + 1) % (*polygons)[poly_idx].size();` is C++ undefined behaviour when the
        // polygon is empty (`% 0`); this port panics on `len == 0` instead of invoking UB. No
        // in-scope caller advances an empty path's index (`PolylineStitcher.hpp`'s own callers
        // always insert at least one point before indexing), so this divergence is unreachable
        // in practice, same as the vendored UB it stands in for.
        self.point_idx = (self.point_idx + 1) % len;
    }

    /// `PathsPointIndex &operator--()` (`:110-116`): retreat, wrapping at the polygon's own
    /// point count.
    pub fn retreat(&mut self) {
        let len = self.polygons[self.poly_idx].point_count();
        if self.point_idx == 0 {
            // PORT NOTE: src/libslic3r/Arachne/utils/PolygonsPointIndex.hpp:113 -- the
            // read-size-then-reset step of the vendored two-step wrap; carries no numeric
            // literal of its own, so this is a correspondence note, not a transcribed constant.
            self.point_idx = len;
        }
        // PORT NOTE: src/libslic3r/Arachne/utils/PolygonsPointIndex.hpp:114 -- `point_idx--;`
        // on an unsigned int wraps to `UINT_MAX` in C++ (defined behaviour there); this port's
        // `usize` subtraction panics on underflow in a debug build instead of wrapping. Reached
        // only when `len == 0` (the `if` above leaves `point_idx` at `0`), the same empty-path
        // case `advance`'s note above names as unreached by any in-scope caller.
        self.point_idx -= 1;
    }

    /// `PathsPointIndex next() const { PathsPointIndex ret(*this); ++ret; return ret; }`
    /// (`:118-123`).
    pub fn next(&self) -> Self {
        let mut ret = *self;
        ret.advance();
        ret
    }

    /// `PathsPointIndex prev() const { PathsPointIndex ret(*this); --ret; return ret; }`
    /// (`:125-130`).
    pub fn prev(&self) -> Self {
        let mut ret = *self;
        ret.retreat();
        ret
    }
}

/// `using PolygonsPointIndex = PathsPointIndex<Polygons>;` (`:133`).
pub type PolygonsPointIndex<'a> = PathsPointIndex<'a, Polygons>;

impl<'a> PathsPointIndex<'a, Polygons> {
    /// `const Polygon &getPolygon() const { return (*polygons)[poly_idx]; }` (`:76`) -- given
    /// its own `impl` block, rather than the generic one above, because the vendored method's
    /// body only type-checks when `Paths` yields a real `Polygon` (every other method above
    /// only needs `Paths::Output: PointPath`); this is the Rust analogue of a C++ member
    /// template whose body is never instantiated for an argument that would not compile it
    /// (`docs/PORTING.md`'s idiom map, "A template" row: "monomorphised the same way the C++
    /// template's instantiations are used in in-scope code").
    pub fn get_polygon(&self) -> &Polygon {
        &self.polygons[self.poly_idx]
    }
}

/// `bool operator==(const PathsPointIndex &other) const` / `operator!=` (`:84-91`) -- see the
/// module doc for why this is hand-written rather than derived.
impl<'a, Paths> PartialEq for PathsPointIndex<'a, Paths> {
    fn eq(&self, other: &Self) -> bool {
        // PORT NOTE: src/libslic3r/Arachne/utils/PolygonsPointIndex.hpp:86 -- the vendored
        // pointer comparison this method reproduces; carries no numeric literal of its own, so
        // this is a correspondence note, not a transcribed constant.
        std::ptr::eq(self.polygons, other.polygons)
            && self.poly_idx == other.poly_idx
            && self.point_idx == other.point_idx
    }
}

// TODO(port): src/libslic3r/Arachne/utils/PolygonsPointIndex.hpp:169-176 -- `std::hash<
// Slic3r::Arachne::PolygonsPointIndex>`, hashing via a ported `PointHash` that
// `crates/slic3r-point/src/point.rs` does not provide (see that module's own doc, "`PointHash`
// is not ported as a standalone hasher"). No in-scope caller of this header needs
// `PolygonsPointIndex` as a hash-map key; named as a finding in this task's pull request for
// whichever fan-out task first does.

/// `struct PolygonsPointIndexSegmentLocator { std::pair<Point, Point> operator()(const
/// PolygonsPointIndex &val) const {...} };` (`:138-148`): a C++ functor, ported as a unit
/// struct with a plain associated function -- this port has no generic callback type that would
/// need the functor's call-operator shape, and every in-scope use of a locator in this crate
/// (`sparse_point_grid.rs`, `sparse_line_grid.rs`) already takes a plain closure.
pub struct PolygonsPointIndexSegmentLocator;

impl PolygonsPointIndexSegmentLocator {
    /// `operator()` body (`:140-146`): the directed segment starting at `val`'s own point and
    /// ending at the next point around its polygon.
    pub fn locate(val: &PathsPointIndex<'_, Polygons>) -> (Point, Point) {
        let poly = &val.polygons[val.poly_idx];
        let start = poly.points[val.point_idx];
        // PORT SRC: src/libslic3r/Arachne/utils/PolygonsPointIndex.hpp:144
        let next_point_idx = (val.point_idx + 1) % poly.points.len();
        let end = poly.points[next_point_idx];
        (start, end)
    }
}

/// `template<typename Paths> struct PathsPointIndexLocator { Point operator()(const
/// PathsPointIndex<Paths> &val) const { return make_point(val.p()); } };` (`:153-160`) -- same
/// functor-to-associated-function treatment as [`PolygonsPointIndexSegmentLocator`] above.
pub struct PathsPointIndexLocator;

impl PathsPointIndexLocator {
    pub fn locate<Paths>(val: &PathsPointIndex<'_, Paths>) -> Point
    where
        Paths: std::ops::Index<usize>,
        Paths::Output: PointPath,
    {
        make_point(val.p())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn square() -> Polygons {
        vec![Polygon::from_points(vec![
            Point::new(0, 0),
            Point::new(100, 0),
            Point::new(100, 100),
            Point::new(0, 100),
        ])]
    }

    #[test]
    fn equality_is_identity_not_point_value() {
        let a = square();
        let b = square();
        let ia = PathsPointIndex::new(&a, 0, 0);
        let ib = PathsPointIndex::new(&b, 0, 0);
        // Same (poly_idx, point_idx) and the same point *value*, but a different `polygons`
        // object: `operator==` compares the pointer, so these must not be equal, exactly the
        // divergence from a value comparison the module doc names.
        assert_ne!(ia, ib);
        assert_eq!(ia, ia);
    }

    #[test]
    fn advance_and_retreat_wrap() {
        let polys = square();
        let mut idx = PathsPointIndex::new(&polys, 0, 3);
        idx.advance();
        assert_eq!(idx.point_idx, 0, "advance must wrap past the last point");
        idx.retreat();
        assert_eq!(idx.point_idx, 3, "retreat must wrap before the first point");
    }

    #[test]
    fn locators_agree_with_p_and_the_segment_shape() {
        let polys = square();
        let idx = PathsPointIndex::new(&polys, 0, 1);
        assert_eq!(PathsPointIndexLocator::locate(&idx), idx.p());
        assert_eq!(
            PolygonsPointIndexSegmentLocator::locate(&idx),
            (idx.p(), idx.next().p())
        );
    }

    /// Proof path 1 (R19, manifest row `Arachne/utils/PolygonsPointIndex.hpp`): drives
    /// `Slic3r::Arachne::PolygonsPointIndex::p/next/prev` over a real `Slic3r::Polygons` built
    /// from the same coordinates on both sides. One cell across this unit's single manifest
    /// row, so one `// PROOF: 1` marker.
    // PROOF: 1
    #[test]
    fn polygons_point_index_matches_the_oracle() {
        let xs = vec![0i64, 100, 100, 0];
        let ys = vec![0i64, 0, 100, 100];
        let points: Vec<Point> = xs
            .iter()
            .zip(&ys)
            .map(|(&x, &y)| Point::new(x, y))
            .collect();
        let polys: Polygons = vec![Polygon::from_points(points)];

        let idx = PathsPointIndex::new(&polys, 0, 1);
        let actual_p = idx.p();
        let actual_next = idx.next().p();
        let actual_prev = idx.prev().p();

        let oracle =
            slic3r_proof::proof_units::polygons_point_index::path1_p_next_prev(&xs, &ys, 1);
        assert_eq!(
            (
                actual_p.x(),
                actual_p.y(),
                actual_next.x(),
                actual_next.y(),
                actual_prev.x(),
                actual_prev.y()
            ),
            (oracle[0], oracle[1], oracle[2], oracle[3], oracle[4], oracle[5]),
            "PolygonsPointIndex::p/next/prev must match the oracle exactly"
        );
    }
}

// PORT STATUS
// source: src/libslic3r/Arachne/utils/PolygonsPointIndex.hpp
// confidence: medium
// tier: exact
// oracle: built
