//! Port of `src/libslic3r/Polygon.hpp`/`Polygon.cpp` (task `62-foundation-polygon`, plan step 20,
//! `port/foundation.tsv` row `foundation-11`): `Polygon`, the closed-ring subtype of
//! `crates/slic3r-multipoint`'s [`MultiPoint`](slic3r_multipoint::multi_point::MultiPoint) trait
//! (this task's own sibling unit, built alongside this one), and its free-function surface.
//!
//! **`Polygon` implements [`MultiPoint`](slic3r_multipoint::multi_point::MultiPoint)**, giving
//! it every default method that trait carries (`scale`, `translate`, `rotate*`, `find_point*`,
//! `bounding_box`, `has_duplicate_points`, `remove_duplicate_points`, ...) with no code of its
//! own. Several Polygon-specific methods below share a name with one of that trait's default
//! methods -- `length`, `is_valid`, `intersection`, `first_intersection`, `intersections` -- and
//! are written as **inherent** methods on `Polygon` rather than inside the `impl MultiPoint for
//! Polygon` block: Rust resolves an inherent method ahead of a trait method of the same name on
//! an unqualified call (`poly.length()`), which is exactly the C++ *name-hiding* relationship a
//! derived class's own non-virtual member has over its base's same-named member
//! (`Polygon::length()` hides `MultiPoint::length()`; neither is declared `virtual`). Reaching
//! the hidden trait method explicitly still works the same way the C++ qualified call does --
//! `<Polygon as MultiPoint>::length(&poly)`, this port's `poly.MultiPoint::length()` -- and
//! `crates/slic3r-ffi/cxx/proof_units/multi_point.cpp`'s own shims do exactly that to prove the
//! base's methods independently (see that file's header comment).
//!
//! **Deferred for the same reason `BoundingBox.rs`/`Point.rs`/`multi_point.rs` each defer their
//! own cross-crate methods -- needs a type from a crate this task does not own, or a dependency
//! edge the cut graph forbids, and this task's own rule forbids inventing one.**
//! `slic3r-polygon`'s own `Cargo.toml` (committed by `34-foundation-spike`) depends on
//! `slic3r-boundingbox`, `slic3r-exception`, `slic3r-libslic3r`, `slic3r-line`,
//! `slic3r-multipoint`, `slic3r-point` -- **no `slic3r-polyline`, no Clipper binding** (the cut
//! graph runs `slic3r-polygon` -> `slic3r-multipoint`, at crate levels 7 and 6; `Polyline.hpp`'s
//! own row is `slic3r-polyline` at level 8, *above* this crate, so the two share an include-graph
//! cycle the cut's feedback-arc order breaks by making this unit come first with no edge back).
//! Deferred, each at its own `// TODO(port):` site below, named in this task's pull request:
//!
//! - Everything returning or taking a `Polyline`/`Polylines`: `split_at_vertex`,
//!   `split_at_index`, `split_at_first_point`, `equally_spaced_points` (`Polygon.hpp:56-61`,
//!   `Polygon.cpp:40-63`), `to_polyline(const Polygon&)`, `to_polylines(const Polygons&)` (both
//!   overloads), `to_polygons(const Polylines&)` (`Polygon.hpp:228-270`) -- `to_polygons(const
//!   VecOfPoints&)`/`to_polygons(VecOfPoints&&)` (`:272-288`) need no `Polyline` at all (built
//!   straight from `Points`) and **are** ported below.
//! - Everything needing a Clipper binding (`ClipperUtils.hpp`, `slic3r-clipperutils`, crate
//!   level 12, far above this crate and no dependency edge to it): `Polygon::simplify`
//!   (`Polygon.cpp:120-135`, `simplify_polygons`), `polygons_simplify` (`:685-706`, `union_`),
//!   `Polygon::overlaps`/the free `overlaps(Polygons&, Polygons&)` (`Polygon.hpp:87`,
//!   `Polygon.cpp:229-240,728-735`, `intersection_pl`/`to_polylines`).
//! - `Polygon::transform(const Transform3d&)` (`Polygon.hpp:99`, `Polygon.cpp:409-433`): needs
//!   the deferred `Transform3d` type `crates/slic3r-point/src/point.rs`'s own module doc already
//!   names as `// TODO(port): src/libslic3r/Point.hpp:83-96`.
//! - `Polygon::split_at_vertex`'s own `throw Slic3r::InvalidArgument(...)` path is moot while the
//!   function itself is deferred; `slic3r-exception` stays an unused dependency of this crate
//!   until a future task fills this gap.
//!
//! **Not ported, with no marker site of its own: the Boost.Polygon `geometry_concept`/
//! `polygon_traits`/`polygon_mutable_traits`/`polygon_set_traits`/`polygon_set_mutable_traits`
//! specialisations for `Slic3r::Polygon`/`Polygons`** (`Polygon.hpp:316-395`). These are C++
//! template-customisation glue with no Rust item to carry over -- they teach Boost.Polygon's own
//! generic algorithms how to read/write `Slic3r::Polygon`, and this port has no Boost.Polygon
//! generic-algorithm caller of its own to need an equivalent adapter for (every in-scope
//! Boost.Polygon use is itself a specific caller the manifest tracks under its own file). Noted
//! here per the module doc convention, not marked `// TODO(port):` -- there is no vendored line
//! this would port, only a mechanism this port does not have.
//!
//! `// PROOF: 3` proves both rows together (one cell: both `Polygon.cpp` and `Polygon.hpp` are
//! `proof path: 3`) through `crates/slic3r-ffi/cxx/proof_units/polygon.cpp`'s `#include
//! <Polygon.cpp>` -- see that file for the duplicate-symbol measurement this `#include` needed
//! before it was written, and `cxx/proof_units/multi_point.cpp` (this task's sibling file) for
//! why the ordinary, externally-linked `Polygon`/`MultiPoint` methods are instead proven through
//! a plain `#include "Polygon.hpp"` shim with no such `#include <Polygon.cpp>` of their own --
//! `point_in_polygon` (`Polygon.cpp:738-766`, file-scope `static`, used by [`contains`] below) is
//! the one symbol in this unit with no external linkage to call any other way.

use slic3r_boundingbox::bounding_box::BoundingBox;
use slic3r_line::line::{Line, Lines};
use slic3r_multipoint::multi_point::{self, MultiPoint};
use slic3r_point::point::{Point, Points, Vec2d};

// -------------------------------------------------------------------------------------------
// Polygon (Polygon.hpp:30-103, Polygon.cpp)
// -------------------------------------------------------------------------------------------

/// `class Polygon : public MultiPoint { public: ... };` (`Polygon.hpp:30-103`). The inherited
/// `points` field is this struct's only field, exactly as it is `MultiPoint`'s only field
/// upstream -- see the module doc for why this implements the
/// [`MultiPoint`](slic3r_multipoint::multi_point::MultiPoint) trait rather than wrapping it.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Polygon {
    pub points: Points,
}

/// `using Polygons = std::vector<Polygon, PointsAllocator<Polygon>>;` (`:22`); see
/// `crates/slic3r-point/src/point.rs`'s `Points` doc for why the custom allocator needs no
/// counterpart.
pub type Polygons = Vec<Polygon>;
/// `using PolygonPtrs = std::vector<Polygon*, PointsAllocator<Polygon*>>;` (`:23`); see
/// `crates/slic3r-point/src/point.rs`'s `PointPtrs` doc for the same borrowed-container
/// treatment.
pub type PolygonPtrs<'a> = Vec<&'a mut Polygon>;
/// `using ConstPolygonPtrs = std::vector<const Polygon*, PointsAllocator<const Polygon*>>;`
/// (`:24`).
pub type ConstPolygonPtrs<'a> = Vec<&'a Polygon>;

/// `inline bool operator==(const Polygon &lhs, const Polygon &rhs) { return lhs.points ==
/// rhs.points; }` (`:105`) / `operator!=` (`:106`): `#[derive(PartialEq)]` above, over the one
/// field, is exactly this comparison; Rust derives `!=` from it automatically.
impl Eq for Polygon {}

impl std::ops::Index<usize> for Polygon {
    type Output = Point;
    /// `const Point& operator[](Points::size_type idx) const` (`:49`).
    fn index(&self, idx: usize) -> &Point {
        &self.points[idx]
    }
}
impl std::ops::IndexMut<usize> for Polygon {
    /// `Point& operator[](Points::size_type idx)` (`:48`).
    fn index_mut(&mut self, idx: usize) -> &mut Point {
        &mut self.points[idx]
    }
}

impl Polygon {
    /// `Polygon() = default;` (`:33`).
    pub fn new() -> Self {
        Polygon {
            points: Points::new(),
        }
    }

    /// `explicit Polygon(const Points &points)` (`:34`).
    pub fn from_points(points: Points) -> Self {
        Polygon { points }
    }

    /// `static Polygon new_scale(const std::vector<Vec2d> &points)` (`:38-44`).
    pub fn new_scale(points: &[Vec2d]) -> Self {
        let mut pgn = Polygon::new();
        pgn.points.reserve(points.len());
        for pt in points {
            pgn.points.push(Point::new_scale(pt.x(), pt.y()));
        }
        pgn
    }

    /// `double length() const` (`Polygon.cpp:24-33`): sums every consecutive edge **plus** the
    /// closing edge back to the first point, computed directly rather than through
    /// [`lines`](Polygon::lines) -- a different traversal from `MultiPoint::length()`'s own
    /// (which sums over `this->lines()`), ported separately per the module doc. Hides
    /// `MultiPoint::length()`.
    pub fn length(&self) -> f64 {
        // `0f64`: the vendored `double l = 0;` (`Polygon.cpp:26`) spells its own literal as the
        // bare integer `0`; a Rust `f64` binding needs a float-typed literal, and `0f64`
        // normalises to the same exempt `"0"` text the vendored literal already is
        // (`docs/PORTING.md` decision 8's `f64`-suffix technique).
        let mut l = 0f64;
        if self.points.len() > 1 {
            l += multi_point_diff_norm(self.points[self.points.len() - 1], self.points[0]);
            for i in 1..self.points.len() {
                l += multi_point_diff_norm(self.points[i], self.points[i - 1]);
            }
        }
        l
    }

    /// `bool is_valid() const { return this->points.size() >= 3; }` (`:69`). Hides
    /// `MultiPoint::is_valid()` (`>= 2`).
    pub fn is_valid(&self) -> bool {
        // PORT SRC: src/libslic3r/Polygon.hpp:69
        self.points.len() >= 3
    }

    /// `static double area(const Points &points)` (`Polygon.cpp:65-77`): the shoelace formula,
    /// `0.5 * sum(cross2(p_prev, p_cur))`.
    pub fn area_of(points: &Points) -> f64 {
        // PORT SRC: src/libslic3r/Polygon.cpp:67
        let mut a = 0.;
        // PORT SRC: src/libslic3r/Polygon.cpp:68
        if points.len() >= 3 {
            let mut p1 = (
                points[points.len() - 1].x() as f64,
                points[points.len() - 1].y() as f64,
            );
            for p in points {
                let p2 = (p.x() as f64, p.y() as f64);
                a += p1.0 * p2.1 - p1.1 * p2.0;
                p1 = p2;
            }
        }
        // PORT SRC: src/libslic3r/Polygon.cpp:76
        0.5 * a
    }

    /// `double area() const { return Polygon::area(points); }` (`:79-82`).
    pub fn area(&self) -> f64 {
        Polygon::area_of(&self.points)
    }

    /// `bool is_counter_clockwise() const { return this->area() >= 0.; }` (`:84-87`).
    pub fn is_counter_clockwise(&self) -> bool {
        // PORT SRC: src/libslic3r/Polygon.cpp:86
        self.area() >= 0.
    }

    /// `bool is_clockwise() const { return !this->is_counter_clockwise(); }` (`:89-92`).
    pub fn is_clockwise(&self) -> bool {
        !self.is_counter_clockwise()
    }

    /// `bool make_counter_clockwise()` (`:94-101`).
    pub fn make_counter_clockwise(&mut self) -> bool {
        if !self.is_counter_clockwise() {
            MultiPoint::reverse(self);
            true
        } else {
            false
        }
    }

    /// `bool make_clockwise()` (`:103-110`).
    pub fn make_clockwise(&mut self) -> bool {
        if self.is_counter_clockwise() {
            MultiPoint::reverse(self);
            true
        } else {
            false
        }
    }

    /// `void douglas_peucker(double tolerance)` (`:112-118`): calls the base's own static
    /// `MultiPoint::_douglas_peucker`, [`multi_point::douglas_peucker`], directly -- a C++
    /// `static` member call, never virtual, so it is a plain function call here too (no trait
    /// involved).
    pub fn douglas_peucker(&mut self, tolerance: f64) {
        self.points.push(self.points[0]);
        let mut p = multi_point::douglas_peucker(&self.points, tolerance);
        p.pop();
        self.points = p;
    }

    // TODO(port): src/libslic3r/Polygon.cpp:120-135 -- `Polygons simplify(double tolerance)
    // const`, needs `simplify_polygons` (`ClipperUtils.hpp`, `slic3r-clipperutils`, a crate this
    // task does not depend on -- see the module doc).

    /// `void triangulate_convex(Polygons* polygons) const` (`:138-150`): only valid on convex
    /// polygons (the vendored comment's own caveat, not checked here either).
    pub fn triangulate_convex(&self, polygons: &mut Polygons) {
        for i in 2..self.points.len() {
            let mut p = Polygon {
                // PORT SRC: src/libslic3r/Polygon.cpp:142
                points: Points::with_capacity(3),
            };
            p.points.push(self.points[0]);
            p.points.push(self.points[i - 1]);
            p.points.push(self.points[i]);
            // `0f64`: the vendored `if (p.area() > 0)` (`:148`) spells its own literal as the
            // bare integer `0`; see [`Polygon::length`]'s doc for why this stays exempt.
            if p.area() > 0f64 {
                polygons.push(p);
            }
        }
    }

    /// `Point centroid() const` (`Polygon.cpp:154-169`).
    pub fn centroid(&self) -> Point {
        // PORT SRC: src/libslic3r/Polygon.cpp:156
        let mut area_sum = 0.;
        // PORT SRC: src/libslic3r/Polygon.cpp:157
        let mut c = Vec2d::new(0., 0.);
        // PORT SRC: src/libslic3r/Polygon.cpp:158
        if self.points.len() >= 3 {
            let mut p1 = (
                self.points[self.points.len() - 1].x() as f64,
                self.points[self.points.len() - 1].y() as f64,
            );
            for p in &self.points {
                let p2 = (p.x() as f64, p.y() as f64);
                let a = p1.0 * p2.1 - p1.1 * p2.0;
                area_sum += a;
                c = Vec2d::new(c.x() + (p1.0 + p2.0) * a, c.y() + (p1.1 + p2.1) * a);
                p1 = p2;
            }
        }
        // PORT SRC: src/libslic3r/Polygon.cpp:168
        Point::from_f64_rounded(c.x() / (3. * area_sum), c.y() / (3. * area_sum))
    }

    /// `bool intersection(const Line &line, Point *intersection) const` (`Polygon.cpp:171-181`):
    /// iterates edges directly (the closing edge first, then every consecutive pair), rather
    /// than through [`lines`](Polygon::lines) -- a different traversal from
    /// `MultiPoint::intersection`'s own, ported separately per the module doc. Hides
    /// `MultiPoint::intersection`.
    pub fn intersection(&self, line: &Line) -> Option<Point> {
        if self.points.len() < 2 {
            return None;
        }
        let closing = Line::new(self.points[0], self.points[self.points.len() - 1]);
        if let Some(p) = closing.intersection(line) {
            return Some(p);
        }
        for i in 1..self.points.len() {
            let edge = Line::new(self.points[i - 1], self.points[i]);
            if let Some(p) = edge.intersection(line) {
                return Some(p);
            }
        }
        None
    }

    /// `bool first_intersection(const Line& line, Point* intersection) const`
    /// (`Polygon.cpp:183-210`): the nearest intersection by *squared* distance to `line.a`
    /// (`MultiPoint::first_intersection`'s own uses the plain, non-squared distance -- a genuine
    /// difference between the two vendored bodies, not a port simplification). Hides
    /// `MultiPoint::first_intersection`.
    pub fn first_intersection(&self, line: &Line) -> Option<Point> {
        if self.points.len() < 2 {
            return None;
        }
        let mut found: Option<Point> = None;
        // PORT SRC: src/libslic3r/Polygon.cpp:189
        let mut dmin = 0.;
        let mut a = self.points[self.points.len() - 1];
        for i in 0..self.points.len() {
            let b = self.points[i];
            let edge = Line::new(a, b);
            if let Some(ip) = edge.intersection(line) {
                let d = multi_point_diff_squared_norm(line.a, ip);
                match found {
                    None => {
                        dmin = d;
                        found = Some(ip);
                    }
                    Some(_) => {
                        if d < dmin {
                            dmin = d;
                            found = Some(ip);
                        }
                    }
                }
            }
            a = b;
        }
        found
    }

    /// `bool intersections(const Line &line, Points* intersections) const`
    /// (`Polygon.cpp:212-227`); see `MultiPoint::intersections`'s own doc
    /// (`crates/slic3r-multipoint/src/multi_point.rs`) for why this returns just the newly found
    /// points rather than appending to a caller-owned vector. Hides `MultiPoint::intersections`.
    pub fn intersections(&self, line: &Line) -> Points {
        let mut out = Points::new();
        if self.points.len() < 2 {
            return out;
        }
        let mut a = self.points[self.points.len() - 1];
        for i in 0..self.points.len() {
            let b = self.points[i];
            let edge = Line::new(a, b);
            if let Some(p) = edge.intersection(line) {
                out.push(p);
            }
            a = b;
        }
        out
    }

    /// `bool overlaps(const Polygons& other) const` (`:87`) is deferred: see the module doc
    // TODO(port): src/libslic3r/Polygon.cpp:229-240 -- `bool overlaps(const Polygons& other)
    // const`, needs `intersection_pl`/`to_polylines` (`ClipperUtils.hpp`, a crate this task does
    // not depend on -- see the module doc).

    /// `Points convex_points(double angle_threshold) const` (`:323-326`), through
    /// `filter_convex_concave_points_by_angle_threshold` (`:287-321`) and
    /// `filter_points_by_vectors` (`:248-268`): kept positive cross products.
    pub fn convex_points(&self, angle_threshold: f64) -> Points {
        filter_convex_concave_points_by_angle_threshold(&self.points, angle_threshold, |v1, v2| {
            // PORT SRC: src/libslic3r/Polygon.cpp:325
            cross2(v1, v2) > 0.
        })
    }

    /// `Points concave_points(double angle_threshold) const` (`:328-331`); see
    /// [`convex_points`](Polygon::convex_points)'s doc. Kept negative cross products.
    pub fn concave_points(&self, angle_threshold: f64) -> Points {
        filter_convex_concave_points_by_angle_threshold(&self.points, angle_threshold, |v1, v2| {
            // PORT SRC: src/libslic3r/Polygon.cpp:330
            cross2(v1, v2) < 0.
        })
    }

    /// `Point point_projection(const Point &point) const` (`Polygon.cpp:334-369`): the closest
    /// point on the polygon's own boundary (vertices and edge-interior feet), not merely the
    /// closest vertex.
    pub fn point_projection(&self, point: Point) -> Point {
        let mut proj = point;
        let mut dmin = f64::MAX;
        if !self.points.is_empty() {
            let n = self.points.len();
            for i in 0..n {
                let pt0 = self.points[i];
                let pt1 = self.points[if i + 1 == n { 0 } else { i + 1 }];
                let d = multi_point_diff_norm(point, pt0);
                if d < dmin {
                    dmin = d;
                    proj = pt0;
                }
                let d = multi_point_diff_norm(point, pt1);
                if d < dmin {
                    dmin = d;
                    proj = pt1;
                }
                let v1x = (pt1.x() - pt0.x()) as f64;
                let v1y = (pt1.y() - pt0.y()) as f64;
                let div = v1x * v1x + v1y * v1y;
                // PORT SRC: src/libslic3r/Polygon.cpp:354
                if div > 0. {
                    let v2x = (point.x() - pt0.x()) as f64;
                    let v2y = (point.y() - pt0.y()) as f64;
                    let t = (v1x * v2x + v1y * v2y) / div;
                    // PORT SRC: src/libslic3r/Polygon.cpp:357
                    if t > 0. && t < 1. {
                        let foot = Point::new(
                            // PORT SRC: src/libslic3r/Polygon.cpp:358
                            (pt0.x() as f64 + t * v1x + 0.5).floor() as i64,
                            // PORT SRC: src/libslic3r/Polygon.cpp:358
                            (pt0.y() as f64 + t * v1y + 0.5).floor() as i64,
                        );
                        let d = multi_point_diff_norm(point, foot);
                        if d < dmin {
                            dmin = d;
                            proj = foot;
                        }
                    }
                }
            }
        }
        proj
    }

    /// `bool on_boundary(const Point &point, double eps) const` (`:75-76`, inline).
    pub fn on_boundary(&self, point: Point, eps: f64) -> bool {
        multi_point_diff_squared_norm(self.point_projection(point), point) < eps * eps
    }

    /// `bool contains(const Point &point) const { return Slic3r::contains(*this, point, true); }`
    /// (`:73`, inline).
    pub fn contains(&self, point: Point) -> bool {
        contains(self, point, true)
    }

    /// `std::vector<float> parameter_by_length() const` (`Polygon.cpp:371-379`): cumulative edge
    /// length from `points[0]`, one entry longer than `points` (the closing edge's own length is
    /// the last entry).
    pub fn parameter_by_length(&self) -> Vec<f32> {
        let n = self.points.len();
        // PORT SRC: external: the vendored `std::vector<float> lengths(points.size()+1, 0.);`
        // (`Polygon.cpp:374`) spells its own literal `0.` -- a trailing-dot float with no digit
        // after the point -- and Rust's grammar forbids a type suffix directly after a
        // trailing-dot float (`0.f32` does not parse; `0.0f32`/`0f32` are the only valid
        // spellings, and neither's normalised text is `"0."`), so no Rust spelling of this
        // value can match the cited line's text (`docs/PORTING.md` decision 8's closing
        // paragraph).
        let mut lengths = vec![0.0f32; n + 1];
        for i in 1..n {
            let d = self.points[i] - self.points[i - 1];
            lengths[i] = lengths[i - 1] + ((d.x() as f32).powi(2) + (d.y() as f32).powi(2)).sqrt();
        }
        // PORT NOTE: src/libslic3r/Polygon.cpp:377 -- the vendored `lengths.back() =
        // lengths[lengths.size()-2] + ...` reads `lengths[(size_t)-1]` (an out-of-bounds,
        // wrapped `size_t`) when `points` is empty, since `lengths.size()` is `1` there; this
        // `if n >= 1` guard is a disclosed divergence for that one degenerate input, returning
        // the harmless `[0.0]` this port's own bounds-checked `Vec` would otherwise panic on,
        // rather than reproducing the vendored out-of-bounds read. No in-scope caller reaches
        // `parameter_by_length`/`densify` on an empty `Polygon`.
        if n >= 1 {
            let d = self.points[0] - self.points[n - 1];
            lengths[n] = lengths[n - 1] + ((d.x() as f32).powi(2) + (d.y() as f32).powi(2)).sqrt();
        }
        lengths
    }

    /// `void densify(float min_length, std::vector<float>* lengths = nullptr)`
    /// (`Polygon.cpp:381-407`): inserts a point whenever a parametrised segment exceeds
    /// `min_length`, splitting it proportionally.
    pub fn densify(&mut self, min_length: f32, lengths_in: Option<&mut Vec<f32>>) {
        let mut lengths_local;
        let lengths: &mut Vec<f32> = match lengths_in {
            Some(l) => l,
            None => {
                lengths_local = self.parameter_by_length();
                &mut lengths_local
            }
        };
        debug_assert_eq!(self.points.len(), lengths.len() - 1);
        let mut j = 1usize;
        while j <= self.points.len() {
            let last = j == self.points.len();
            let i = if last { 0 } else { j };
            if lengths[j] - lengths[j - 1] > min_length {
                let diff = self.points[i] - self.points[j - 1];
                let diff_len = lengths[j] - lengths[j - 1];
                let r = min_length / diff_len;
                // `Point(r*diff[0], r*diff[1])` (`Polygon.cpp:401`) calls the rounding
                // `Point(double, double)` constructor (`r*diff[0]` promotes to `float` by the
                // usual arithmetic conversions, then widens losslessly to `double` for the
                // call) -- `.round()`, not a truncating `as i64`, which `Point::new`'s own
                // two-`i64`-argument form would give.
                let new_pt = self.points[j - 1]
                    + Point::from_f64_rounded(
                        (r * diff.x() as f32) as f64,
                        (r * diff.y() as f32) as f64,
                    );
                self.points.insert(j, new_pt);
                lengths.insert(j, lengths[j - 1] + min_length);
            }
            j += 1;
        }
    }

    // TODO(port): src/libslic3r/Polygon.cpp:409-433 -- `Polygon transform(const Transform3d&)
    // const`, needs the deferred `Transform3d` type (`crates/slic3r-point/src/point.rs`'s own
    // `// TODO(port): src/libslic3r/Point.hpp:83-96`).

    // TODO(port): src/libslic3r/Polygon.hpp:56-61, Polygon.cpp:40-63 -- `split_at_vertex`,
    // `split_at_index`, `split_at_first_point`, `equally_spaced_points`, each returning or built
    // from a `Polyline` (`slic3r-polyline`, a crate this task does not depend on -- see the
    // module doc).
}

impl MultiPoint for Polygon {
    fn points(&self) -> &Points {
        &self.points
    }
    fn points_mut(&mut self) -> &mut Points {
        &mut self.points
    }
    /// `const Point& last_point() const { return this->points.front(); }` (`:52`).
    fn last_point(&self) -> Point {
        self.points[0]
    }
    /// `Lines lines() const { return to_lines(*this); }` (`:55`, `Polygon.cpp:35-38`), through
    /// [`to_lines`]: every consecutive pair, plus the closing edge back to the first point.
    fn lines(&self) -> Lines {
        to_lines(self)
    }
}

/// `(a - b).cast<double>().norm()`, this module's own copy of the helper
/// `crates/slic3r-multipoint/src/multi_point.rs` keeps privately (that module's own copy is not
/// `pub`, so this crate cannot reuse it -- see this task's rule against adding a dependency's
/// surface it does not already export).
fn multi_point_diff_norm(a: Point, b: Point) -> f64 {
    multi_point_diff_squared_norm(a, b).sqrt()
}

/// `(a - b).cast<double>().squaredNorm()`; see [`multi_point_diff_norm`]'s doc.
fn multi_point_diff_squared_norm(a: Point, b: Point) -> f64 {
    let d = a - b;
    let dx = d.x() as f64;
    let dy = d.y() as f64;
    dx * dx + dy * dy
}

/// `inline typename Derived::Scalar cross2(...)` (`Point.hpp:121-126`) narrowed to `f64`, this
/// module's own copy since `crates/slic3r-point/src/point.rs`'s own `cross2_f64` is private to
/// that module (its own doc: "proven by proof path 3 where needed rather than by visibility").
fn cross2(v1: (f64, f64), v2: (f64, f64)) -> f64 {
    v1.0 * v2.1 - v1.1 * v2.0
}

/// `template<typename FilterFn> Points filter_points_by_vectors(const Points &poly, FilterFn
/// filter)` (`Polygon.cpp:247-268`).
fn filter_points_by_vectors<F: Fn((f64, f64), (f64, f64)) -> bool>(
    poly: &Points,
    filter: F,
) -> Points {
    let mut p1 = poly[poly.len() - 1];
    let mut v1 = {
        let prev = poly[poly.len() - 2];
        ((p1.x() - prev.x()) as f64, (p1.y() - prev.y()) as f64)
    };
    let mut out = Points::new();
    for &p2 in poly {
        let v2 = ((p2.x() - p1.x()) as f64, (p2.y() - p1.y()) as f64);
        if filter(v1, v2) {
            out.push(p1);
        }
        v1 = v2;
        p1 = p2;
    }
    out
}

/// `template<typename ConvexConcaveFilterFn> Points
/// filter_convex_concave_points_by_angle_threshold(const Points &poly, double angle_threshold,
/// ConvexConcaveFilterFn convex_concave_filter)` (`Polygon.cpp:287-321`).
fn filter_convex_concave_points_by_angle_threshold<F: Fn((f64, f64), (f64, f64)) -> bool>(
    poly: &Points,
    angle_threshold: f64,
    convex_concave_filter: F,
) -> Points {
    // PORT SRC: src/libslic3r/Polygon.cpp:291
    debug_assert!(angle_threshold >= 0.);
    debug_assert!(angle_threshold < std::f64::consts::PI);
    if angle_threshold > slic3r_libslic3r::libslic3r::EPSILON {
        let cos_threshold = angle_threshold.next_after_pos_infinity().cos();
        filter_points_by_vectors(poly, |v1, v2| {
            if !convex_concave_filter(v1, v2) {
                return false;
            }
            let n1 = (v1.0 * v1.0 + v1.1 * v1.1).sqrt();
            let n2 = (v2.0 * v2.0 + v2.1 * v2.1).sqrt();
            let v1n = (v1.0 / n1, v1.1 / n1);
            let v2n = (v2.0 / n2, v2.1 / n2);
            let vec_dot = v1n.0 * v2n.0 + v1n.1 * v2n.1;
            vec_dot < cos_threshold
        })
    } else {
        filter_points_by_vectors(poly, convex_concave_filter)
    }
}

/// A tiny local extension trait for `f64::next_after` toward `+INFINITY`
/// (`std::nextafter(angle_threshold, +INFINITY)`, `Polygon.cpp:300`): Rust's standard library has
/// no `next_after` of its own, so this reproduces the one direction this call site needs through
/// the IEEE-754 bit-pattern successor (next representable value upward), which is exactly what
/// `nextafter(x, +inf)` computes for a finite, non-negative `x` (this call site's own domain,
/// `0 <= angle_threshold < PI`, asserted just above).
trait NextAfterPosInfinity {
    fn next_after_pos_infinity(self) -> f64;
}
impl NextAfterPosInfinity for f64 {
    fn next_after_pos_infinity(self) -> f64 {
        if self.is_nan() || self == f64::INFINITY {
            return self;
        }
        let bits = self.to_bits();
        // PORT SRC: external: this port's own IEEE-754 bit-pattern successor, not a value
        // transcribed from a vendored literal -- `std::nextafter` itself carries no such
        // literal at its call site (`Polygon.cpp:300`); see this trait's own doc.
        let next_bits = if self >= 0.0 { bits + 1 } else { bits - 1 };
        f64::from_bits(next_bits)
    }
}

/// `Lines Polygon::lines() const { return to_lines(*this); }` (`Polygon.hpp:55`), through
/// `inline Lines to_lines(const Polygon &poly)` (`:203-213`).
pub fn to_lines(poly: &Polygon) -> Lines {
    let pts = &poly.points;
    let mut lines = Lines::with_capacity(pts.len());
    if pts.len() > 2 {
        for i in 0..pts.len() - 1 {
            lines.push(Line::new(pts[i], pts[i + 1]));
        }
        lines.push(Line::new(pts[pts.len() - 1], pts[0]));
    }
    lines
}

/// `inline Lines to_lines(const Polygons &polys)` (`:215-226`).
pub fn to_lines_polygons(polys: &Polygons) -> Lines {
    let mut lines = Lines::with_capacity(count_points(polys));
    for poly in polys {
        let pts = &poly.points;
        if pts.len() > 2 {
            for i in 0..pts.len() - 1 {
                lines.push(Line::new(pts[i], pts[i + 1]));
            }
            lines.push(Line::new(pts[pts.len() - 1], pts[0]));
        }
    }
    lines
}

/// `extern BoundingBox get_extents(const Polygon &poly) { return poly.bounding_box(); }`
/// (`Polygon.cpp:435-438`): the base's own `MultiPoint::bounding_box()`, not hidden by any
/// Polygon-specific override.
pub fn get_extents(poly: &Polygon) -> BoundingBox {
    poly.bounding_box()
}

/// `extern BoundingBox get_extents(const Polygons &polygons)` (`:440-449`).
pub fn get_extents_polygons(polygons: &Polygons) -> BoundingBox {
    let mut bb = BoundingBox::new();
    if !polygons.is_empty() {
        bb = get_extents(&polygons[0]);
        for poly in &polygons[1..] {
            bb.merge_bb(&get_extents(poly));
        }
    }
    bb
}

/// `extern BoundingBox get_extents_rotated(const Polygon &poly, double angle)` (`:451-454`).
pub fn get_extents_rotated(poly: &Polygon, angle: f64) -> BoundingBox {
    multi_point::get_extents_rotated_points(&poly.points, angle)
}

/// `extern BoundingBox get_extents_rotated(const Polygons &polygons, double angle)`
/// (`:456-465`).
pub fn get_extents_rotated_polygons(polygons: &Polygons, angle: f64) -> BoundingBox {
    let mut bb = BoundingBox::new();
    if !polygons.is_empty() {
        bb = multi_point::get_extents_rotated_points(&polygons[0].points, angle);
        for poly in &polygons[1..] {
            bb.merge_bb(&multi_point::get_extents_rotated_points(
                &poly.points,
                angle,
            ));
        }
    }
    bb
}

/// `extern std::vector<BoundingBox> get_extents_vector(const Polygons &polygons)`
/// (`:467-474`).
pub fn get_extents_vector(polygons: &Polygons) -> Vec<BoundingBox> {
    polygons.iter().map(get_extents).collect()
}

/// `bool polygon_is_convex(const Points &poly)` (`Polygon.cpp:477-493`): strict (collinear fails
/// it), over exact `i64` cross products -- the vendored body's own `.cast<int64_t>()`.
///
/// `wrapping_*` throughout, not plain `-`/`*`: the vendored `int64_t` arithmetic itself
/// overflows (technically UB in the C++ standard, but a deterministic two's-complement wrap in
/// the actual compiled oracle, which uses no overflow sanitizer -- see
/// `## Decisions recorded on 2026-11-25`-style practical-behaviour reasoning) for an input whose
/// coordinates are large enough that two edge-vector components multiply past `i64::MAX`; a
/// fuzz run surfaced exactly this (`det = v1.x() * v2.y() - ...` panicking on overflow in a debug
/// build, for coordinates at the extreme of the `i32` range the fuzz target draws from -- far
/// outside any physically meaningful scaled coordinate, but still a real divergence from the
/// oracle's own wraparound on the same input). Wrapping arithmetic reproduces the oracle's actual
/// bit pattern instead of panicking (debug) or silently matching by luck (release), so the two
/// stay bit-for-bit equal even in this UB territory.
pub fn polygon_is_convex(poly: &Points) -> bool {
    // PORT SRC: src/libslic3r/Polygon.cpp:479
    if poly.len() < 3 {
        return false;
    }
    let mut p0 = poly[poly.len() - 2];
    let mut p1 = poly[poly.len() - 1];
    for &p2 in poly {
        let v1 = p1 - p0;
        let v2 = p2 - p1;
        let det = v1
            .x()
            .wrapping_mul(v2.y())
            .wrapping_sub(v1.y().wrapping_mul(v2.x()));
        if det < 0 {
            return false;
        }
        p0 = p1;
        p1 = p2;
    }
    true
}

/// `inline bool polygon_is_convex(const Polygon &poly)` (`Polygon.hpp:116`).
pub fn polygon_is_convex_polygon(poly: &Polygon) -> bool {
    polygon_is_convex(&poly.points)
}

/// `inline bool has_duplicate_points(Polygon &&poly)` (`:119`), through
/// `slic3r_point::point::has_duplicate_points`.
pub fn has_duplicate_points_owned(poly: Polygon) -> bool {
    slic3r_point::point::has_duplicate_points(poly.points)
}

/// `inline bool has_duplicate_points(const Polygon &poly)` (`:120`).
pub fn has_duplicate_points(poly: &Polygon) -> bool {
    slic3r_point::point::has_duplicate_points_ref(&poly.points)
}

/// `bool has_duplicate_points(const Polygons &polys)` (`Polygon.cpp:495-511`): the `#if 1` branch
/// is the only one compiled upstream (the `#else` per-contour check is preprocessed out, and so
/// is not ported, per the don't-translate list's treatment of disabled code).
pub fn has_duplicate_points_polygons(polys: &Polygons) -> bool {
    let mut allpts = Points::with_capacity(count_points(polys));
    for poly in polys {
        allpts.extend_from_slice(&poly.points);
    }
    slic3r_point::point::has_duplicate_points(allpts)
}

/// `bool remove_same_neighbor(Polygon &polygon)` (`Polygon.cpp:513-531`): collapses every
/// adjacent duplicate (`std::unique`), then the wrap-around duplicate between the new last point
/// and the first.
pub fn remove_same_neighbor(polygon: &mut Polygon) -> bool {
    if polygon.points.is_empty() {
        return false;
    }
    let mut last = 1usize;
    for i in 1..polygon.points.len() {
        if polygon.points[last - 1] != polygon.points[i] {
            polygon.points[last] = polygon.points[i];
            last += 1;
        }
    }
    if polygon.points[last - 1] == polygon.points[0] {
        last -= 1;
    }
    if last == polygon.points.len() {
        return false;
    }
    polygon.points.truncate(last);
    true
}

/// `bool remove_same_neighbor(Polygons &polygons)` (`:533-543`).
pub fn remove_same_neighbor_polygons(polygons: &mut Polygons) -> bool {
    if polygons.is_empty() {
        return false;
    }
    let mut exist = false;
    for polygon in polygons.iter_mut() {
        exist |= remove_same_neighbor(polygon);
    }
    // PORT SRC: src/libslic3r/Polygon.cpp:541
    polygons.retain(|p| p.points.len() > 2);
    exist
}

/// `static inline bool is_stick(const Point &p1, const Point &p2, const Point &p3)`
/// (`Polygon.cpp:545-565`): file-scope `static`, kept module-private per the idiom map's
/// "a `static` free function" row.
fn is_stick(p1: Point, p2: Point, p3: Point) -> bool {
    let v1 = p2 - p1;
    let v2 = p3 - p2;
    // `wrapping_*`: see [`polygon_is_convex`]'s doc for why this `int64_t` arithmetic is kept
    // bit-for-bit equal to the oracle's own (possibly overflowing) result rather than panicking.
    let dir = v1
        .x()
        .wrapping_mul(v2.x())
        .wrapping_add(v1.y().wrapping_mul(v2.y()));
    if dir > 0 {
        return false;
    }
    let l2_1 = (v1.x() as f64) * (v1.x() as f64) + (v1.y() as f64) * (v1.y() as f64);
    let l2_2 = (v2.x() as f64) * (v2.x() as f64) + (v2.y() as f64) * (v2.y() as f64);
    if dir == 0 {
        // `0f64`: the vendored `return l2_1 == 0 || l2_2 == 0;` (`Polygon.cpp:558`) spells its
        // own literals as the bare integer `0`; see [`Polygon::length`]'s doc for why this
        // stays exempt.
        return l2_1 == 0f64 || l2_2 == 0f64;
    }
    let cross = (v1.x() as f64) * (v2.y() as f64) - (v2.x() as f64) * (v1.y() as f64);
    let dist2 = cross * cross / l2_1.max(l2_2);
    dist2 < slic3r_libslic3r::libslic3r::EPSILON * slic3r_libslic3r::libslic3r::EPSILON
}

/// `bool remove_sticks(Polygon &poly)` (`Polygon.cpp:567-591`).
pub fn remove_sticks(poly: &mut Polygon) -> bool {
    let mut modified = false;
    let mut j = 1usize;
    let mut i = 1usize;
    while i + 1 < poly.points.len() {
        if !is_stick(poly.points[j - 1], poly.points[i], poly.points[i + 1]) {
            if j < i {
                poly.points[j] = poly.points[i];
            }
            j += 1;
        }
        i += 1;
    }
    j += 1;
    if j < poly.points.len() {
        poly.points[j - 1] = poly.points[poly.points.len() - 1];
        poly.points.truncate(j);
        modified = true;
    }
    // PORT SRC: src/libslic3r/Polygon.cpp:584
    while poly.points.len() >= 3
        && is_stick(
            poly.points[poly.points.len() - 2],
            poly.points[poly.points.len() - 1],
            poly.points[0],
        )
    {
        poly.points.pop();
        modified = true;
    }
    // PORT SRC: src/libslic3r/Polygon.cpp:588
    while poly.points.len() >= 3
        && is_stick(
            poly.points[poly.points.len() - 1],
            poly.points[0],
            poly.points[1],
        )
    {
        poly.points.remove(0);
    }
    modified
}

/// `bool remove_sticks(Polygons &polys)` (`Polygon.cpp:593-608`).
pub fn remove_sticks_polygons(polys: &mut Polygons) -> bool {
    let mut modified = false;
    let mut j = 0usize;
    for i in 0..polys.len() {
        modified |= remove_sticks(&mut polys[i]);
        // PORT SRC: src/libslic3r/Polygon.cpp:599
        if polys[i].points.len() >= 3 {
            if j < i {
                polys.swap(i, j);
            }
            j += 1;
        }
    }
    if j < polys.len() {
        polys.truncate(j);
    }
    modified
}

/// `bool remove_degenerate(Polygons &polys)` (`Polygon.cpp:610-625`).
pub fn remove_degenerate(polys: &mut Polygons) -> bool {
    let mut modified = false;
    let mut j = 0usize;
    for i in 0..polys.len() {
        // PORT SRC: src/libslic3r/Polygon.cpp:615
        if polys[i].points.len() >= 3 {
            if j < i {
                polys.swap(i, j);
            }
            j += 1;
        } else {
            modified = true;
        }
    }
    if j < polys.len() {
        polys.truncate(j);
    }
    modified
}

/// `bool remove_small(Polygons &polys, double min_area)` (`Polygon.cpp:627-642`).
pub fn remove_small(polys: &mut Polygons, min_area: f64) -> bool {
    let mut modified = false;
    let mut j = 0usize;
    for i in 0..polys.len() {
        if polys[i].area().abs() >= min_area {
            if j < i {
                polys.swap(i, j);
            }
            j += 1;
        } else {
            modified = true;
        }
    }
    if j < polys.len() {
        polys.truncate(j);
    }
    modified
}

/// `void remove_collinear(Polygon &poly)` (`Polygon.cpp:644-677`).
pub fn remove_collinear(poly: &mut Polygon) {
    if poly.points.len() > 2 {
        let mut pp = Points::with_capacity(poly.points.len() + 2);
        pp.push(poly.points[poly.points.len() - 1]);
        pp.extend_from_slice(&poly.points);
        pp.push(poly.points[0]);
        poly.points.clear();

        let mut i = 0usize;
        let mut k;
        while i < pp.len() - 2 {
            k = i + 1;
            let p1 = pp[i];
            loop {
                if k >= pp.len() - 1 {
                    break;
                }
                let p2 = pp[k];
                let p3 = pp[k + 1];
                let l = Line::new(p1, p3);
                if l.distance_to(p2) < slic3r_libslic3r::libslic3r::scaled_epsilon() {
                    k += 1;
                } else {
                    if i > 0 {
                        poly.points.push(p1);
                    }
                    i = k;
                    break;
                }
            }
            if k > pp.len() - 2 {
                break;
            }
        }
        poly.points.push(pp[i]);
    }
}

/// `void remove_collinear(Polygons &polys)` (`:679-683`).
pub fn remove_collinear_polygons(polys: &mut Polygons) {
    for poly in polys.iter_mut() {
        remove_collinear(poly);
    }
}

// TODO(port): src/libslic3r/Polygon.cpp:685-706 -- `Polygons polygons_simplify(const Polygons
// &source_polygons, double tolerance)`, needs `union_` (`ClipperUtils.hpp`, a crate this task
// does not depend on -- see the module doc).

/// `bool polygons_match(const Polygon &l, const Polygon &r)` (`Polygon.cpp:710-726`): same
/// topology (cyclic point sequence), contours may start at different vertices.
pub fn polygons_match(l: &Polygon, r: &Polygon) -> bool {
    if l.points.len() != r.points.len() {
        return false;
    }
    let start = match l.points.iter().position(|&p| p == r.points[0]) {
        Some(i) => i,
        None => return false,
    };
    let n = l.points.len();
    for (k, &rp) in r.points.iter().enumerate() {
        if l.points[(start + k) % n] != rp {
            return false;
        }
    }
    true
}

// TODO(port): src/libslic3r/Polygon.cpp:229-240,728-735 -- `Polygon::overlaps`/the free
// `overlaps(const Polygons&, const Polygons&)`, needs `intersection_pl`/`to_polylines`
// (`ClipperUtils.hpp`, a crate this task does not depend on -- see the module doc).

/// `static int point_in_polygon(const Point &pt, const Points &path)` (`Polygon.cpp:738-766`,
/// file-scope `static`): `1` inside, `0` outside, `-1` on the boundary -- Clipper1's own
/// `PointInPolygon` algorithm, kept module-private per the idiom map's "a `static` free
/// function" row (no in-scope caller outside [`contains`] below, proven instead by proof path 3
/// -- see the module doc). `wrapping_*` in `d`'s two computations: see [`polygon_is_convex`]'s
/// doc for why this `int64_t` cross-product arithmetic is kept bit-for-bit equal to the oracle's
/// own (possibly overflowing) result rather than panicking.
fn point_in_polygon(pt: Point, path: &Points) -> i32 {
    let mut result = 0i32;
    let cnt = path.len();
    // PORT SRC: src/libslic3r/Polygon.cpp:742
    if cnt < 3 {
        return 0;
    }
    let mut ip = path[0];
    for i in 1..=cnt {
        let ip_next = if i == cnt { path[0] } else { path[i] };
        if ip_next.y() == pt.y()
            && (ip_next.x() == pt.x()
                || (ip.y() == pt.y() && (ip_next.x() > pt.x()) == (ip.x() < pt.x())))
        {
            return -1;
        }
        if (ip.y() < pt.y()) != (ip_next.y() < pt.y()) {
            if ip.x() >= pt.x() {
                if ip_next.x() > pt.x() {
                    result = 1 - result;
                } else {
                    let d = (ip.x() - pt.x())
                        .wrapping_mul(ip_next.y() - pt.y())
                        .wrapping_sub((ip_next.x() - pt.x()).wrapping_mul(ip.y() - pt.y()));
                    if d == 0 {
                        return -1;
                    }
                    if (d > 0) == (ip_next.y() > ip.y()) {
                        result = 1 - result;
                    }
                }
            } else if ip_next.x() > pt.x() {
                let d = (ip.x() - pt.x())
                    .wrapping_mul(ip_next.y() - pt.y())
                    .wrapping_sub((ip_next.x() - pt.x()).wrapping_mul(ip.y() - pt.y()));
                if d == 0 {
                    return -1;
                }
                if (d > 0) == (ip_next.y() > ip.y()) {
                    result = 1 - result;
                }
            }
        }
        ip = ip_next;
    }
    result
}

/// `bool contains(const Polygon &polygon, const Point &p, bool border_result)`
/// (`Polygon.cpp:768-775`).
pub fn contains(polygon: &Polygon, p: Point, border_result: bool) -> bool {
    let inside = point_in_polygon(p, &polygon.points);
    if inside == -1 {
        border_result
    } else {
        (inside % 2) == 1
    }
}

/// `bool contains(const Polygons &polygons, const Point &p, bool border_result)`
/// (`:777-787`).
pub fn contains_polygons(polygons: &Polygons, p: Point, border_result: bool) -> bool {
    let mut poly_count_inside = 0;
    for poly in polygons {
        let is_inside_this_poly = point_in_polygon(p, &poly.points);
        if is_inside_this_poly == -1 {
            return border_result;
        }
        poly_count_inside += is_inside_this_poly;
    }
    (poly_count_inside % 2) == 1
}

/// `Polygon make_circle(double radius, double error)` (`Polygon.cpp:789-794`).
pub fn make_circle(radius: f64, error: f64) -> Polygon {
    // PORT SRC: src/libslic3r/Polygon.cpp:791
    let angle = 2. * (1. - error / radius).acos();
    // PORT SRC: src/libslic3r/Polygon.cpp:792
    let num_segments = (2. * std::f64::consts::PI / angle).ceil() as usize;
    make_circle_num_segments(radius, num_segments)
}

/// `Polygon make_circle_num_segments(double radius, size_t num_segments)` (`:796-806`).
pub fn make_circle_num_segments(radius: f64, num_segments: usize) -> Polygon {
    let mut out = Polygon::new();
    out.points.reserve(num_segments);
    // PORT SRC: src/libslic3r/Polygon.cpp:800
    let angle_inc = 2.0 * std::f64::consts::PI / num_segments as f64;
    for i in 0..num_segments {
        let angle = angle_inc * i as f64;
        out.points.push(Point::new(
            (angle.cos() * radius) as i64,
            (angle.sin() * radius) as i64,
        ));
    }
    out
}

/// `inline double total_length(const Polygons &polylines)` (`Polygon.hpp:127-132`): despite the
/// vendored parameter's own name, this takes `Polygons` (closed rings), summing
/// `Polygon::length()` (the hiding override above, not `MultiPoint::length()`).
pub fn total_length(polylines: &Polygons) -> f64 {
    // `0f64`: the vendored `double total = 0;` (`Polygon.hpp:128`) spells its own literal as
    // the bare integer `0`; see [`Polygon::length`]'s doc for why this stays exempt.
    let mut total = 0f64;
    for p in polylines {
        total += p.length();
    }
    total
}

/// `inline double area(const Polygon &poly) { return poly.area(); }` (`:134`).
pub fn area(poly: &Polygon) -> f64 {
    poly.area()
}

/// `inline double area(const Polygons &polys)` (`:136-142`).
pub fn area_polygons(polys: &Polygons) -> f64 {
    // PORT SRC: src/libslic3r/Polygon.hpp:138
    let mut s = 0.;
    for p in polys {
        s += p.area();
    }
    s
}

/// `inline void polygons_append(Polygons &dst, const Polygons &src)` (`:155`).
pub fn polygons_append(dst: &mut Polygons, src: &Polygons) {
    dst.extend_from_slice(src);
}

/// `inline void polygons_append(Polygons &dst, Polygons &&src)` (`:157-165`).
pub fn polygons_append_moved(dst: &mut Polygons, mut src: Polygons) {
    if dst.is_empty() {
        *dst = src;
    } else {
        dst.append(&mut src);
    }
}

/// `inline void polygons_rotate(Polygons &polys, double angle)` (`:169-175`).
pub fn polygons_rotate(polys: &mut Polygons, angle: f64) {
    let cos_angle = angle.cos();
    let sin_angle = angle.sin();
    for p in polys.iter_mut() {
        p.rotate_cos_sin(cos_angle, sin_angle);
    }
}

/// `inline void polygons_reverse(Polygons &polys)` (`:177-181`).
pub fn polygons_reverse(polys: &mut Polygons) {
    for p in polys.iter_mut() {
        MultiPoint::reverse(p);
    }
}

/// `inline Points to_points(const Polygon &poly)` (`:183-186`).
pub fn to_points(poly: &Polygon) -> Points {
    poly.points.clone()
}

/// `inline size_t count_points(const Polygons &polys)` (`:188-192`).
pub fn count_points(polys: &Polygons) -> usize {
    polys.iter().map(|p| p.points.len()).sum()
}

/// `inline Points to_points(const Polygons &polys)` (`:194-201`).
pub fn to_points_polygons(polys: &Polygons) -> Points {
    let mut points = Points::with_capacity(count_points(polys));
    for poly in polys {
        points.extend_from_slice(&poly.points);
    }
    points
}

/// `inline Polygons to_polygons(const VecOfPoints &paths)` (`:272-279`): needs no `Polyline`
/// (built straight from `Points`), so this *is* ported, unlike the module doc's deferred
/// `Polyline`-typed overloads.
pub fn to_polygons_from_paths(paths: &[Points]) -> Polygons {
    paths
        .iter()
        .map(|p| Polygon::from_points(p.clone()))
        .collect()
}

/// `inline Polygons to_polygons(VecOfPoints &&paths)` (`:281-288`): the move overload.
pub fn to_polygons_from_paths_moved(paths: Vec<Points>) -> Polygons {
    paths.into_iter().map(Polygon::from_points).collect()
}

/// `struct PolygonPoint { size_t index; Point point; };` (`:301-309`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PolygonPoint {
    pub index: usize,
    pub point: Point,
}
/// `using PolygonPoints = std::vector<PolygonPoint>;` (`:310`).
pub type PolygonPoints = Vec<PolygonPoint>;

#[cfg(test)]
mod tests {
    use super::*;

    fn square(x0: i64, y0: i64, side: i64) -> Polygon {
        Polygon::from_points(vec![
            Point::new(x0, y0),
            Point::new(x0 + side, y0),
            Point::new(x0 + side, y0 + side),
            Point::new(x0, y0 + side),
        ])
    }

    fn flatten(points: &Points) -> Vec<i64> {
        points.iter().flat_map(|p| [p.x(), p.y()]).collect()
    }

    #[test]
    fn polygon_multi_point_trait_methods_work_through_the_trait() {
        let mut p = square(0, 0, 10);
        MultiPoint::scale(&mut p, 2.0);
        assert_eq!(
            p.points,
            vec![
                Point::new(0, 0),
                Point::new(20, 0),
                Point::new(20, 20),
                Point::new(0, 20),
            ]
        );
        assert_eq!(p.find_point(Point::new(20, 20)), 2);
        assert!(p.bounding_box().defined);
    }

    #[test]
    fn polygon_length_hides_multi_point_length() {
        // A right triangle: MultiPoint::length() (open, via to_lines-as-closed-ring from the
        // trait's own `lines()`) and `Polygon::length()` (closing edge computed directly) both
        // sum the same three edges of a *closed* ring here, so they agree on this input --
        // they differ only when `lines()` is overridden to a different shape than `length()`'s
        // own direct traversal, which does not happen for `Polygon`. This just exercises both
        // call forms compile and run.
        let p = square(0, 0, 10);
        let via_inherent = p.length();
        let via_trait = MultiPoint::length(&p);
        assert_eq!(via_inherent, via_trait);
        assert_eq!(via_inherent, 40.0);
    }

    #[test]
    fn polygon_orientation_and_make_clockwise() {
        let mut p = square(0, 0, 10);
        assert!(p.is_counter_clockwise());
        assert!(p.make_clockwise());
        assert!(p.is_clockwise());
        assert!(!p.make_clockwise());
    }

    #[test]
    fn polygon_douglas_peucker_simplifies_a_near_straight_edge() {
        let mut p = Polygon::from_points(vec![
            Point::new(0, 0),
            Point::new(50, 1),
            Point::new(100, 0),
            Point::new(100, 100),
            Point::new(0, 100),
        ]);
        p.douglas_peucker(5.0);
        assert_eq!(p.points.len(), 4);
    }

    #[test]
    fn polygon_convex_and_concave_points() {
        // A square has four convex corners and no concave ones.
        let p = square(0, 0, 10);
        assert_eq!(p.convex_points(0.0).len(), 4);
        assert_eq!(p.concave_points(0.0).len(), 0);
    }

    #[test]
    fn polygon_to_polygons_from_paths_needs_no_polyline() {
        let paths = vec![vec![Point::new(0, 0), Point::new(1, 0)]];
        let polys = to_polygons_from_paths(&paths);
        assert_eq!(polys.len(), 1);
        assert_eq!(polys[0].points.len(), 2);
    }

    /// Proof path 3 (R19, `port/manifest.tsv`'s `Polygon.cpp`/`Polygon.hpp` rows, one cell):
    /// every function below is genuine `Polygon`/free-function code, reached through
    /// `crates/slic3r-ffi/cxx/proof_units/polygon.cpp`'s `#include <Polygon.cpp>` translation
    /// unit, through `slic3r-proof`'s re-export of the FFI crate's per-unit proof bridges
    /// (`docs/PORTING.md`, "Decisions recorded on 2026-11-25"; the shared-file list that
    /// decision names is itself stale -- see this task's pull request body).
    // PROOF: 3
    #[test]
    fn polygon_methods_match_the_oracle() {
        // Fully-qualified `slic3r_proof::proof_units::polygon::path3_*` calls throughout this
        // test, not a `use ... as oracle` alias: `port/port-status`'s differential-comparison
        // check (`oracle_names`) recognises an oracle entry point from a `use` path's own
        // un-aliased segments, so an aliased import's call sites (`oracle::path3_area(...)`)
        // would not be recognised as reaching the oracle at all.
        use slic3r_proof::tiers::{compare_unit, Oracle};

        let xs = [0i64, 100, 100, 0];
        let ys = [0i64, 0, 100, 100];
        let p = Polygon::from_points(
            xs.iter()
                .zip(&ys)
                .map(|(&x, &y)| Point::new(x, y))
                .collect(),
        );

        compare_unit(
            "Polygon.cpp",
            Oracle::Built,
            p.area(),
            slic3r_proof::proof_units::polygon::path3_area(&xs, &ys),
        )
        .expect("Polygon::area must match the oracle exactly");
        assert_eq!(
            p.is_counter_clockwise(),
            slic3r_proof::proof_units::polygon::path3_is_counter_clockwise(&xs, &ys)
        );

        let c = p.centroid();
        compare_unit(
            "Polygon.cpp",
            Oracle::Built,
            vec![c.x(), c.y()],
            slic3r_proof::proof_units::polygon::path3_centroid(&xs, &ys),
        )
        .expect("Polygon::centroid must match the oracle exactly");

        assert_eq!(
            p.contains(Point::new(50, 50)),
            slic3r_proof::proof_units::polygon::path3_contains(&xs, &ys, 50, 50)
        );
        assert_eq!(
            p.contains(Point::new(500, 500)),
            slic3r_proof::proof_units::polygon::path3_contains(&xs, &ys, 500, 500)
        );

        let proj = p.point_projection(Point::new(50, -20));
        compare_unit(
            "Polygon.cpp",
            Oracle::Built,
            vec![proj.x(), proj.y()],
            slic3r_proof::proof_units::polygon::path3_point_projection(&xs, &ys, 50, -20),
        )
        .expect("Polygon::point_projection must match the oracle exactly");

        assert_eq!(
            polygon_is_convex(&p.points),
            slic3r_proof::proof_units::polygon::path3_polygon_is_convex(&xs, &ys)
        );

        let circle = make_circle_num_segments(100.0, 12);
        let oracle_circle =
            slic3r_proof::proof_units::polygon::path3_make_circle_num_segments(100.0, 12);
        compare_unit(
            "Polygon.cpp",
            Oracle::Built,
            flatten(&circle.points),
            oracle_circle,
        )
        .expect("make_circle_num_segments must match the oracle exactly");

        // Review finding: the first draft's input here (`(0,0),(50,0),(100,0),(100,100),
        // (0,100)`) is collinear but never reverses direction, so `is_stick` returns `false`
        // at every call (`dir > 0` at `Polygon.cpp:550`) and `remove_sticks`'s own mutation
        // branches (the `poly.points[j] = poly.points[i]` shift, the trailing-point truncation,
        // both trailing `while` loops) never ran. `(100,0) -> (50,0)` below is a genuine spike
        // (it reverses), which `is_stick` does flag (`dir < 0`, then `dist2 < SCALED_EPSILON`
        // at `Polygon.cpp:558,564`), exercising the shift this unit's proof had not reached.
        let sticky = Polygon::from_points(vec![
            Point::new(0, 0),
            Point::new(100, 0),
            Point::new(50, 0),
            Point::new(100, 100),
            Point::new(0, 100),
        ]);
        let mut rust_sticky = sticky.clone();
        let modified = remove_sticks(&mut rust_sticky);
        let oracle_sticks = slic3r_proof::proof_units::polygon::path3_remove_sticks(
            &sticky.points.iter().map(|p| p.x()).collect::<Vec<_>>(),
            &sticky.points.iter().map(|p| p.y()).collect::<Vec<_>>(),
        );
        let mut actual_sticks = vec![if modified { 1i64 } else { 0 }];
        actual_sticks.extend(flatten(&rust_sticky.points));
        compare_unit("Polygon.cpp", Oracle::Built, actual_sticks, oracle_sticks)
            .expect("remove_sticks must match the oracle exactly");

        let actual_param = p.parameter_by_length();
        let oracle_param = slic3r_proof::proof_units::polygon::path3_parameter_by_length(&xs, &ys);
        assert_eq!(
            actual_param, oracle_param,
            "Polygon::parameter_by_length diverged"
        );

        // Review finding: an earlier draft of `densify` truncated the new point's coordinates
        // (`as i64` on the raw float product) instead of rounding through
        // `Point(double, double)`'s own constructor the way `Point new_pt = points[j-1] +
        // Point(r*diff[0], r*diff[1]);` (`Polygon.cpp:401`) does -- undetected until this
        // comparison was added, because every earlier case here happened not to land near a
        // half-integer boundary. `min_length = 23.5` against this square's 100-unit edges gives
        // `r = 0.235`, so the first inserted point's `r * diff.x()` is `23.5` -- truncation
        // gives `23`, `Point`'s own round-half-away-from-zero gives `24`; measured by reverting
        // this fix locally and rerunning this test, which then fails with exactly that 23-vs-24
        // mismatch (see this task's pull request body).
        let mut densified = p.clone();
        densified.densify(23.5, None);
        let oracle_densified = slic3r_proof::proof_units::polygon::path3_densify(&xs, &ys, 23.5);
        compare_unit(
            "Polygon.cpp",
            Oracle::Built,
            flatten(&densified.points),
            oracle_densified,
        )
        .expect("Polygon::densify must match the oracle exactly");

        // `Polygon::intersection`/`first_intersection`/`intersections` each has its own
        // traversal, independent of `MultiPoint`'s (see this module's own doc) -- a line that
        // crosses two edges of the square distinguishes "first edge found" (`intersection`)
        // from "nearest to `line.a`" (`first_intersection`) and exercises `intersections`'
        // multi-result path.
        let line_ax = -10i64;
        let line_ay = 150i64;
        let line_bx = 150i64;
        let line_ay2 = -10i64;
        let crossing = slic3r_line::line::Line::new(
            Point::new(line_ax, line_ay),
            Point::new(line_bx, line_ay2),
        );
        let actual_int = p.intersection(&crossing);
        let actual_int_vec = match actual_int {
            Some(pt) => vec![1i64, pt.x(), pt.y()],
            None => vec![0, 0, 0],
        };
        let oracle_int = slic3r_proof::proof_units::polygon::path3_intersection(
            &xs,
            &ys,
            crossing.a.x(),
            crossing.a.y(),
            crossing.b.x(),
            crossing.b.y(),
        );
        compare_unit("Polygon.cpp", Oracle::Built, actual_int_vec, oracle_int)
            .expect("Polygon::intersection must match the oracle exactly");

        let actual_first = p.first_intersection(&crossing);
        let actual_first_vec = match actual_first {
            Some(pt) => vec![1i64, pt.x(), pt.y()],
            None => vec![0, 0, 0],
        };
        let oracle_first = slic3r_proof::proof_units::polygon::path3_first_intersection(
            &xs,
            &ys,
            crossing.a.x(),
            crossing.a.y(),
            crossing.b.x(),
            crossing.b.y(),
        );
        compare_unit("Polygon.cpp", Oracle::Built, actual_first_vec, oracle_first)
            .expect("Polygon::first_intersection must match the oracle exactly");

        let actual_ints = p.intersections(&crossing);
        let oracle_ints = slic3r_proof::proof_units::polygon::path3_intersections(
            &xs,
            &ys,
            crossing.a.x(),
            crossing.a.y(),
            crossing.b.x(),
            crossing.b.y(),
        );
        compare_unit(
            "Polygon.cpp",
            Oracle::Built,
            flatten(&actual_ints),
            oracle_ints,
        )
        .expect("Polygon::intersections must match the oracle exactly");

        // `MultiPoint::_douglas_peucker`, called directly by `Polygon::douglas_peucker` (see
        // this module's own doc) -- the stack/index logic is this task's most intricate, and
        // had no differential comparison of its own beyond `polygon_douglas_peucker_simplifies_
        // a_near_straight_edge`'s hand-written expected output.
        let wiggly: Points = vec![
            Point::new(0, 0),
            Point::new(20, 3),
            Point::new(40, -2),
            Point::new(60, 1),
            Point::new(100, 0),
        ];
        let wiggly_xs: Vec<i64> = wiggly.iter().map(|pt| pt.x()).collect();
        let wiggly_ys: Vec<i64> = wiggly.iter().map(|pt| pt.y()).collect();
        let actual_dp = multi_point::douglas_peucker(&wiggly, 5.0);
        let oracle_dp =
            slic3r_proof::proof_units::polygon::path3_douglas_peucker(&wiggly_xs, &wiggly_ys, 5.0);
        compare_unit(
            "MultiPoint.cpp",
            Oracle::Built,
            flatten(&actual_dp),
            oracle_dp,
        )
        .expect("MultiPoint::_douglas_peucker must match the oracle exactly");
    }
}

// PORT STATUS
// source: src/libslic3r/Polygon.cpp
// source: src/libslic3r/Polygon.hpp
// confidence: medium
// tier: exact
// oracle: built
