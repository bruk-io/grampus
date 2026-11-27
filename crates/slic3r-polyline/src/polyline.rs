//! Port of `src/libslic3r/Polyline.hpp`/`Polyline.cpp` (task `63-foundation-polyline`, plan step
//! 20, `port/foundation.tsv` row `foundation-12`): `Polyline`, the **open**-path sibling of
//! `crates/slic3r-polygon`'s `Polygon` over `crates/slic3r-multipoint`'s
//! [`MultiPoint`](slic3r_multipoint::multi_point::MultiPoint) trait (task `62-foundation-polygon`,
//! this task's own dependency), plus `ThickPolyline`, `Polyline3` and the free-function surface
//! both headers declare.
//!
//! **`Polyline` implements [`MultiPoint`]**, giving it every default method that trait carries
//! (`scale`, `translate`, `rotate*`, `find_point*`, `bounding_box`, `length`, ...) with no code of
//! its own. Several Polyline-specific methods below share a name with one of that trait's default
//! methods -- `append`, `append_points`, `reverse`, `clear` -- and are written as **inherent**
//! methods on `Polyline` rather than inside the `impl MultiPoint for Polyline` block, exactly as
//! `crates/slic3r-polygon/src/polygon.rs`'s own module doc explains for `Polygon::length`: Rust
//! resolves an inherent method ahead of a trait method of the same name on an unqualified call,
//! which is the C++ *name-hiding* relationship `Polyline::append` (`Polyline.hpp:65-71`) has over
//! `MultiPoint::append` (neither is `virtual`). `Polyline::lines()`/`last_point()` **do** override
//! `MultiPoint`'s own pure-virtual methods (`override` at `Polyline.hpp:114,116`), which is the one
//! place this unit uses genuine virtual dispatch, inside the `impl MultiPoint for Polyline` block
//! below. `ThickPolyline` (`Polyline.hpp:262-283`) publicly inherits `Polyline` in C++ but adds no
//! virtual override of its own over `MultiPoint`'s two pure virtuals (it only hides two more
//! non-virtual `Polyline` methods, `reverse`/`clear`, the same way `Polyline` hides `MultiPoint`'s)
//! -- the idiom map's "virtual dispatch over a class hierarchy" row does not cover this
//! (non-polymorphic) relationship, so [`ThickPolyline`] below is its own struct and its own
//! `impl MultiPoint for ThickPolyline`, matching `Polyline`'s shape rather than wrapping it; this
//! duplicates roughly a dozen lines (its own `points`/`points_mut`/`last_point`/`lines`) rather
//! than inventing a composition/delegation idiom this document does not sanction. **`ThickPolyline`
//! also reaches every one of `Polyline`'s inherited non-virtual public methods that this unit
//! itself ports** -- `append`/`append_before`/`append_points`/`append_points_range`/
//! `append_polyline`/`append_polyline_moved`, `leftmost_point`, `clip_end`/`clip_start`/
//! `extend_end`/`extend_start`, `equally_spaced_points`/`equally_spaced_lines`, `simplify`,
//! `split_at_index`/`split_at_length`, `is_closed` and `Index`/`IndexMut` -- through a private
//! `via_polyline` helper that moves `self.points` into a temporary [`Polyline`], calls the
//! already-ported method on it, and writes the result back. This is faithful by construction,
//! not merely convenient: in C++, `this->reverse()` (or any other non-virtual call) made from
//! inside `Polyline::clip_end`'s own body resolves at compile time to `Polyline::reverse`, never
//! `ThickPolyline::reverse`, regardless of `this`'s actual dynamic type, because non-virtual
//! dispatch binds on the *static* type the method body is compiled against -- exactly what
//! constructing a `Polyline` and calling its own methods on it reproduces. None of these methods
//! touches `width`/`endpoints` in the vendored body either (`Polyline`'s own fields are its whole
//! state), so those two fields are deliberately left unresized by every one of them: that is the
//! vendored behaviour reached through non-virtual dispatch, not an oversight here. Found missing
//! by review with a named in-scope caller (`src/libslic3r/Fill/FillConcentric.cpp:183`,
//! `thick_polylines_out[i].clip_end(this->loop_clipping)`, crate `slic3r-fill`, not yet started)
//! and fixed in the same pull request; see that review's own verdict in the pull request body.
//! `Polyline::split_at`/`is_straight` stay deferred for `ThickPolyline` too, for the identical
//! cross-crate gap named above for `Polyline` itself -- no second TODO repeats it.
//! `simplify_by_fitting_arc`/`reset_to_linear_move` are the same arc-bookkeeping-only omission as
//! `Polyline`'s own. `Polyline3`
//! (`Polyline.hpp:297-357`) is the unrelated 3D sibling, over
//! `crates/slic3r-multipoint`'s [`MultiPoint3`](slic3r_multipoint::multi_point::MultiPoint3) --
//! the same non-polymorphic-inherited-field shape `MultiPoint`/`MultiPoint3` themselves have, per
//! that crate's own module doc.
//!
//! **Deferred for the same reason `multi_point.rs`/`BoundingBox.rs`/`Point.rs` each defer their
//! own cross-crate methods -- needs a type from a crate this task does not own, or a dependency
//! edge the cut graph forbids, and this task's own rule forbids inventing one:**
//!
//! - **The `fitting_result: std::vector<PathFittingData>` field (`Polyline.hpp:135`,
//!   `Polyline3`'s own copy at `:350`) is not a field of either struct below at all.**
//!   `PathFittingData`/`EMovePathType`/`ArcSegment` live in `ArcFitter.hpp`/`ArcFitter.cpp`, whose
//!   manifest rows are `template-heavy: no` -- a Phase A/B fan-out unit (`slic3r-arcfitter`), not
//!   drafted yet, and not a type this task may invent (`task.md`'s own rule: "An included
//!   in-scope file whose manifest row is `template-heavy: no` is a fan-out module... Where a
//!   translation needs an item from one, leave a `// TODO(port):` marker rather than a guess").
//!   Every vendored method that touches `fitting_result` guards its arc bookkeeping with
//!   `if (fitting_result.empty()) return;`/`if (!fitting_result.empty()) { ... }`
//!   (`Polyline.cpp:84-85,418,432,463,496`, and `Polyline3`'s identical mirrors), and that
//!   bookkeeping never feeds back into the points it computes -- **for a polyline with no arc
//!   data, the vendored point-level computation and this port's are the same code.** So every
//!   method below that mutates points ports that computation in full and leaves a
//!   `// TODO(port):` at the omitted bookkeeping's own site, naming the vendored lines; a method
//!   that is *entirely* arc bookkeeping with no points-only remainder --
//!   `simplify_by_fitting_arc`, `reset_to_linear_move`, the two private
//!   `split_fitting_result_*_index` helpers, and `append_fitting_result_after_append_*` -- is
//!   omitted outright rather than landed as a `todo!()` body (`task.md`: "never land a `mod` line
//!   over a `todo!()`"). **Fidelity is not claimed for an arc-fitted polyline** (one whose
//!   `fitting_result` is non-empty at the pin); this port's proofs and fuzz target exercise only
//!   arc-free polylines, which is the domain the omission leaves fully faithful.
//! - **`Polyline::split_at`/`Polyline3::split_at(Point&, ...)` and both `foot_pt` overloads**
//!   (`Polyline.cpp:232-274,602-622,852-899`) need `Point::projection_onto(const Line&)`/
//!   `Point3::projection_onto(const Line3&)`, left unported in `crates/slic3r-point/src/point.rs`
//!   (that module's own doc: "needs a type from a crate this task does not own" -- written before
//!   `slic3r-line` existed for `Point.rs` to depend on, and this task may not fill `point.rs`'s own
//!   gap, "Write only your own modules"). Omitted, named here, matching
//!   `crates/slic3r-multipoint/src/multi_point.rs`'s own precedent for the identical kind of gap.
//! - **`Polyline::is_straight`** (`Polyline.cpp:357-367`) needs `Line::parallel_to(double)`, left
//!   unported in `crates/slic3r-line/src/line.rs` for the same reason one level further down:
//!   that method itself needs `Slic3r::Geometry::directions_parallel` (`slic3r-geometry`, a
//!   fan-out crate not in `slic3r-line`'s dependency list, by that module's own doc). Omitted,
//!   named here.
//!
//! `slic3r-polyline`'s own `Cargo.toml` (committed by `34-foundation-spike`, from `port/cut.tsv`'s
//! `dep` rows) carries `slic3r-arcfitter` and `slic3r-circle` as path dependencies -- the cut's
//! own rule is textual (`Polyline.cpp` `#include`s both `ArcFitter.hpp` and `Circle.hpp`), not a
//! judgement that either is actually used: `Circle.hpp` has no in-scope caller in this unit at
//! all (`grep -n Circle vendor/OrcaSlicer/src/libslic3r/Polyline.{cpp,hpp}` finds only the
//! `#include` line), and every genuine `ArcFitter`/`PathFittingData` use is the deferred
//! `fitting_result` bookkeeping above. Both stay unused dependencies of this crate, the same way
//! `crates/slic3r-polygon/src/polygon.rs` leaves `slic3r-exception` unused until its own deferred
//! `split_at_vertex` is filled.
//!
//! **Ported in full, with no deferral:** `polylines_merge` (`Polyline.hpp:240-252`) is a C++
//! template over any `PointsType` with `.begin()`/`.end()`/assignment and a free `append(dst,
//! std::move(src))` -- its one in-scope caller (`Algorithm/RegionExpansion.cpp:149`) instantiates
//! it over `ClipperZUtils::ZPath`, a type outside this crate's dependency list, but the template
//! itself needs nothing `Polyline`-specific: `Slic3r::append(std::vector<T>&, std::vector<T>&&)`
//! is already `slic3r_libslic3r::libslic3r::append`, a generic free function
//! `55-foundation-libslic3r-execution` already ported. [`polylines_merge`] below is genuinely
//! generic over `Vec<T>`, so it needs no crate this task does not already depend on and is ported
//! for real, not deferred.
//!
//! **Proof paths.** `Polyline.cpp`'s row is `proof path: 1` (the externally linked, `.cpp`-defined
//! methods, through the built oracle via `crates/slic3r-ffi/cxx/proof_units/polyline.{h,cpp}` --
//! `## Decisions recorded on 2026-11-25`, decision 1). `Polyline.hpp`'s row is `proof path: 2`;
//! `posSlice.dump` (the row's own pipeline stage) carries no field this unit's header-only surface
//! produces (its fields are closed-ring `PrintObject`/`Layer` geometry -- `lslices`/`slices`/
//! `raw_slices`/`fills`, read and confirmed empty of open-path data for the committed fixture), so
//! this proof reaches the oracle through the same per-unit bridge path 1 uses, which
//! `## Decisions recorded on 2026-11-26`, decision 2 ("What a `proof path: 2` cell commits a task
//! to") settles as conforming and not a fresh finding: "a `proof path: 2` row whose unit writes
//! nothing any dump records is proven through its per-unit bridge, keeps its `// PROOF: 2`
//! marker, and is conforming. `ObjectID.hpp` is the merged precedent."
//!
//! **A disclosed divergence from `task.md`'s own text, not a weakened check**: the task body's
//! "Built beside the other foundation tasks" section describes a path-1/path-3 proof as appending
//! one shim to a shared `crates/slic3r-ffi/cxx/proof_shims.cpp` and one bridge declaration to a
//! shared `crates/slic3r-ffi/src/proof.rs`, with `lib.rs` already carrying a `mod proof;` line.
//! None of that exists on `main`: `crates/slic3r-ffi/src/lib.rs` carries `pub mod proof_units;`,
//! not `mod proof;`, there is no `cxx/proof_shims.cpp` or `src/proof.rs` anywhere in the tree, and
//! `crates/slic3r-ffi/src/proof_units.rs`'s own header comment states why -- a shared append point
//! was a merge conflict across hundreds of port branches built side by side, so
//! `## Decisions recorded on 2026-11-25` (decision 1) replaced it with **per-unit** files, globbed
//! by `build.rs`, that a task adds without touching any file a sibling task also edits. Every
//! already-merged foundation sibling (`point.rs`, `line.rs`, `bounding_box.rs`, `multi_point.rs`,
//! `polygon.rs`) follows the per-unit convention, not the task-body text. This task follows the
//! same precedent: three new files
//! (`crates/slic3r-ffi/cxx/proof_units/polyline.{h,cpp}`,
//! `crates/slic3r-ffi/src/proof_units/polyline.rs`), no shared file in `slic3r-ffi` touched.

use slic3r_boundingbox::bounding_box::BoundingBox;
use slic3r_line::line::{Line, Line3, Lines, Lines3, ThickLine, ThickLines};
use slic3r_multipoint::multi_point::{self, MultiPoint, MultiPoint3};
use slic3r_point::point::{Point, Point3, Points, Points3, Vec2d};

// -------------------------------------------------------------------------------------------
// Polyline (Polyline.hpp:20-150, Polyline.cpp)
// -------------------------------------------------------------------------------------------

/// `class Polyline : public MultiPoint { public: ... };` (`Polyline.hpp:25-147`). The inherited
/// `points` field is this struct's only field -- see the module doc for why `fitting_result`
/// (`:135`) is not a field here at all.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Polyline {
    pub points: Points,
}

/// `typedef std::vector<Polyline> Polylines;` (`:22`).
pub type Polylines = Vec<Polyline>;

/// `inline bool operator==(const Polyline &lhs, const Polyline &rhs) { return lhs.points ==
/// rhs.points; }` (`:149`) / `operator!=` (`:150`): `#[derive(PartialEq)]` above, over the one
/// field, is exactly this comparison.
impl Eq for Polyline {}

impl std::ops::Index<usize> for Polyline {
    type Output = Point;
    /// `const Point& operator[](Points::size_type idx) const` (`:112`).
    fn index(&self, idx: usize) -> &Point {
        &self.points[idx]
    }
}
impl std::ops::IndexMut<usize> for Polyline {
    /// `Point& operator[](Points::size_type idx)` (`:111`).
    fn index_mut(&mut self, idx: usize) -> &mut Point {
        &mut self.points[idx]
    }
}

impl Polyline {
    /// `Polyline() {};` (`:27`).
    pub fn new() -> Self {
        Polyline {
            points: Points::new(),
        }
    }

    /// `explicit Polyline(const Points &points)` (`:39-41`); also stands in for the move overload
    /// (`:42-44`), identical for an owned `Points` in Rust.
    pub fn from_points(points: Points) -> Self {
        Polyline { points }
    }

    /// `explicit Polyline(const Point &p1, const Point &p2)` (`:33-38`).
    pub fn from_two_points(p1: Point, p2: Point) -> Self {
        Polyline {
            points: vec![p1, p2],
        }
    }

    /// `static Polyline new_scale(const std::vector<Vec2d> &points)` (`:55-63`).
    pub fn new_scale(points: &[Vec2d]) -> Self {
        let mut pl = Polyline::new();
        pl.points.reserve(points.len());
        for pt in points {
            pl.points.push(Point::new_scale(pt.x(), pt.y()));
        }
        pl
    }

    /// `void append(const Point &point)` (`:65-71`): hides `MultiPoint::append`. De-duplicates
    /// against the current last point, the same "don't need to append same point" rule every
    /// `Polyline` append overload below carries.
    /// // TODO(port): src/libslic3r/Polyline.hpp:70 -- `append_fitting_result_after_append_points()`
    /// omitted; see the module doc.
    pub fn append(&mut self, point: Point) {
        if !self.points.is_empty() && *self.points.last().unwrap() == point {
            return;
        }
        self.points.push(point);
    }

    /// `void append_before(const Point& point)` (`:73-86`).
    /// // TODO(port): src/libslic3r/Polyline.hpp:77-78 -- the `size() == 1` branch's
    /// `fitting_result.clear()` omitted; see the module doc.
    pub fn append_before(&mut self, point: Point) {
        if !self.points.is_empty() && self.points[0] == point {
            return;
        }
        if self.points.len() == 1 {
            self.points.push(point);
            self.points.reverse();
        } else {
            self.reverse();
            self.append(point);
            self.reverse();
        }
    }

    /// `void append(const Points &src)` (`:88-94`): hides `MultiPoint::append_points`.
    pub fn append_points(&mut self, src: &[Point]) {
        if !self.points.is_empty() && !src.is_empty() && *self.points.last().unwrap() == src[0] {
            self.append_points_range(&src[1..]);
        } else {
            self.append_points_range(src);
        }
    }

    /// `void append(const Points::const_iterator &begin, const Points::const_iterator &end)`
    /// (`:95-102`).
    /// // TODO(port): src/libslic3r/Polyline.hpp:101 -- `append_fitting_result_after_append_points()`
    /// omitted; see the module doc.
    pub fn append_points_range(&mut self, src: &[Point]) {
        if !self.points.is_empty() && !src.is_empty() && *self.points.last().unwrap() == src[0] {
            self.points.extend_from_slice(&src[1..]);
        } else {
            self.points.extend_from_slice(src);
        }
    }

    /// `void append(Points &&src)` (`:103-107`): needs no override here -- the vendored body
    /// forwards straight to `MultiPoint::append(std::move(src))` with no dedup check of its own
    /// (unlike [`append_points`] above), which is exactly
    /// [`multi_point::MultiPoint::append_moved`](MultiPoint::append_moved)'s own trait-default
    /// behaviour; call that directly. Only the omitted
    /// `append_fitting_result_after_append_points()` afterward (`:106`) differs, per the module
    /// doc.

    /// `void append(const Polyline& src)` (`Polyline.cpp:369-390`).
    /// // TODO(port): src/libslic3r/Polyline.cpp:375,380-384,388 -- `fitting_result` copy/merge
    /// bookkeeping (`append_fitting_result_after_append_polyline`) omitted; see the module doc.
    pub fn append_polyline(&mut self, src: &Polyline) {
        if !src.is_valid() {
            return;
        }
        if self.points.is_empty() {
            self.points = src.points.clone();
        } else {
            self.append(src.points[0]);
            self.points.extend_from_slice(&src.points[1..]);
        }
    }

    /// `void append(Polyline&& src)` (`Polyline.cpp:392-415`): identical points-level behaviour to
    /// [`append_polyline`] for an owned `src` -- the vendored move overload's only difference is
    /// avoiding a copy and clearing the moved-from `src` afterward, neither observable in this
    /// port's own output.
    pub fn append_polyline_moved(&mut self, src: Polyline) {
        self.append_polyline(&src);
    }

    /// `const Point& leftmost_point() const` (`Polyline.cpp:21-29`).
    pub fn leftmost_point(&self) -> Point {
        let mut p = self.points[0];
        for &pt in &self.points[1..] {
            if pt.x() < p.x() {
                p = pt;
            }
        }
        p
    }

    /// `void clear() { MultiPoint::clear(); this->fitting_result.clear(); }` (`:118`): hides
    /// `MultiPoint::clear`.
    pub fn clear(&mut self) {
        self.points.clear();
    }

    /// `void reverse()` (`Polyline.cpp:43-58`): hides `MultiPoint::reverse`.
    /// // TODO(port): src/libslic3r/Polyline.cpp:47-57 -- `fitting_result` reversal omitted; see
    /// the module doc.
    pub fn reverse(&mut self) {
        self.points.reverse();
    }

    /// `void clip_end(double distance)` (`Polyline.cpp:61-101`): removes `distance` (in scaled
    /// coordinate units) from the end of the polyline, inserting an interpolated point if the
    /// distance does not land exactly on an existing vertex.
    /// // TODO(port): src/libslic3r/Polyline.cpp:83-100 -- clipping `fitting_result` to match
    /// (including re-deriving the new last point from a clipped arc, `:92-94`) omitted; see the
    /// module doc. `remove_after_index`/`last_point_inserted` (`:64,77`) only feed that omitted
    /// bookkeeping and the latter is write-only even in the vendored body (set, never read).
    pub fn clip_end(&mut self, mut distance: f64) {
        // `0f64`: the vendored `while (distance > 0)` (`Polyline.cpp:65`) spells its own literal
        // as the bare integer `0`, so `0f64`'s normalised text (`docs/PORTING.md` decision 8)
        // matches that line's own text -- the suffix is chosen for the textual match decision 8
        // requires, not to reach the exempt `0`/`1`/`2` set un-marked (the set membership is a
        // side effect of the value, not the reason for this spelling).
        while distance > 0f64 {
            let last = self.last_point();
            let last_f = Vec2d::new(last.x() as f64, last.y() as f64);
            self.points.pop();
            if self.points.is_empty() {
                return;
            }
            let cur = self.last_point();
            let vx = cur.x() as f64 - last_f.x();
            let vy = cur.y() as f64 - last_f.y();
            let lsqr = vx * vx + vy * vy;
            if lsqr > distance * distance {
                let scale = distance / lsqr.sqrt();
                self.points.push(Point::new(
                    (last_f.x() + vx * scale) as i64,
                    (last_f.y() + vy * scale) as i64,
                ));
                break;
            }
            distance -= lsqr.sqrt();
        }
    }

    /// `void clip_start(double distance)` (`:104-110`).
    pub fn clip_start(&mut self, distance: f64) {
        self.reverse();
        self.clip_end(distance);
        if self.points.len() >= 2 {
            self.reverse();
        }
    }

    /// `void extend_end(double distance)` (`:112-118`): extends the last segment's own direction
    /// by `distance`. Eigen's `.normalized()` leaves a zero-length vector unchanged rather than
    /// dividing by a zero norm -- the same guard `crates/slic3r-polygon/src/polygon.rs`'s
    /// `filter_convex_concave_points_by_angle_threshold` already reproduces for the identical
    /// Eigen method.
    pub fn extend_end(&mut self, distance: f64) {
        let n = self.points.len();
        let d = self.points[n - 1] - self.points[n - 2];
        let dx = d.x() as f64;
        let dy = d.y() as f64;
        let norm_sq = dx * dx + dy * dy;
        // PORT SRC: external: Eigen's own `squaredNorm() == 0` guard inside
        // `MatrixBase::normalized()`, at
        // `vendor/OrcaSlicer/deps/build/arm64/dep_Eigen-prefix/src/dep_Eigen/Eigen/src/Core/
        // Dot.h:96-102` (`RealScalar z = n.squaredNorm(); if (z > RealScalar(0)) return n /
        // numext::sqrt(z); else return n;`) -- not a value transcribed from a vendored literal at
        // `Polyline.cpp:115` (that call site carries no numeric literal of its own), but the
        // library-defined threshold this doc comment cites; see
        // `crates/slic3r-polygon/src/polygon.rs`'s own identical guard and marker. Spelled `0.0`
        // rather than `0f64` so this literal is *not* in `port/port-status`'s exempt `0`/`1`/`2`
        // set, which is what makes `check_numeric_constants` read this marker at all -- an
        // earlier `0f64` spelling here left the marker mechanically inert (`port/port-status`
        // never reads past an all-exempt line to find a covering marker), found by review.
        let (vx, vy) = if norm_sq == 0.0 {
            (dx, dy)
        } else {
            let norm = norm_sq.sqrt();
            (dx / norm, dy / norm)
        };
        let new_last =
            self.points[n - 1] + Point::new((vx * distance) as i64, (vy * distance) as i64);
        self.append(new_last);
    }

    /// `void extend_start(double distance)` (`:120-125`).
    pub fn extend_start(&mut self, distance: f64) {
        self.reverse();
        self.extend_end(distance);
        self.reverse();
    }

    /// `Points equally_spaced_points(double distance) const` (`Polyline.cpp:127-153`): points
    /// picked along the polyline's own path, evenly spaced. The vendored `for` loop's own
    /// `--it;` just before the loop's own `++it` cancel out, so a long segment is revisited with
    /// the adjusted `len` rather than advancing to the next one; this loop reproduces that by not
    /// advancing `i` on that path instead of relying on the same cancellation.
    pub fn equally_spaced_points(&self, distance: f64) -> Points {
        let mut out = Points::new();
        out.push(self.first_point());
        // `0f64`: the vendored `double len = 0;` (`Polyline.cpp:133`) spells its own literal as
        // the bare integer `0`, so `0f64`'s normalised text matches that line's own text, the
        // same textual-match reasoning `clip_end` above states in full.
        let mut len = 0f64;
        let mut i = 1usize;
        while i < self.points.len() {
            let p1 = self.points[i - 1];
            let p1x = p1.x() as f64;
            let p1y = p1.y() as f64;
            let cur = self.points[i];
            let vx = cur.x() as f64 - p1x;
            let vy = cur.y() as f64 - p1y;
            let segment_length = (vx * vx + vy * vy).sqrt();
            len += segment_length;
            if len < distance {
                i += 1;
                continue;
            }
            if len == distance {
                out.push(cur);
                // `0f64`: the vendored `len = 0;` (`Polyline.cpp:144`) is the same bare-integer
                // literal as above.
                len = 0f64;
                i += 1;
                continue;
            }
            let take = segment_length - (len - distance);
            let scale = take / segment_length;
            out.push(Point::new(
                (p1x + vx * scale) as i64,
                (p1y + vy * scale) as i64,
            ));
            len = -take;
        }
        out
    }

    /// `void simplify(double tolerance)` (`:155-159`): Douglas-Peucker simplification, through
    /// `MultiPoint::_douglas_peucker` ([`multi_point::douglas_peucker`]).
    /// // TODO(port): src/libslic3r/Polyline.cpp:158 -- `fitting_result.clear()` omitted (moot:
    /// there is no `fitting_result` field here; see the module doc).
    pub fn simplify(&mut self, tolerance: f64) {
        self.points = multi_point::douglas_peucker(&self.points, tolerance);
    }

    /// `Polylines equally_spaced_lines(double distance) const` (`:167-206`): splits the polyline
    /// into consecutive pieces, each (except possibly the last) exactly `distance` long. Measures
    /// from the growing `line`'s own last point every iteration, unlike
    /// [`equally_spaced_points`]'s fixed original-vertex measurement -- ported with the same
    /// shape, including the same "revisit this index" loop structure.
    pub fn equally_spaced_lines(&self, distance: f64) -> Polylines {
        let mut lines = Polylines::new();
        let mut line = Polyline::new();
        line.append(self.first_point());
        // `0f64`: the vendored `double len = 0;` (`Polyline.cpp:172`) spells its own literal as
        // the bare integer `0`; see `equally_spaced_points`'s identical comment above.
        let mut len = 0f64;
        let mut i = 1usize;
        while i < self.points.len() {
            let p1 = *line.points.last().unwrap();
            let p1x = p1.x() as f64;
            let p1y = p1.y() as f64;
            let cur = self.points[i];
            let vx = cur.x() as f64 - p1x;
            let vy = cur.y() as f64 - p1y;
            let segment_length = (vx * vx + vy * vy).sqrt();
            len += segment_length;
            if len < distance {
                i += 1;
                continue;
            }
            if len == distance {
                line.append(cur);
                lines.push(line.clone());
                line.clear();
                line.append(cur);
                // `0f64`: the vendored `len = 0;` (`Polyline.cpp:187`) is the same bare-integer
                // literal as above.
                len = 0f64;
                i += 1;
                continue;
            }
            let take = distance;
            let scale = take / segment_length;
            line.append(Point::new(
                (p1x + vx * scale) as i64,
                (p1y + vy * scale) as i64,
            ));
            lines.push(line.clone());
            line.clear();
            line.append(lines.last().unwrap().last_point());
            len = -take;
        }
        if line.size() == 1 {
            line.append(self.last_point());
            if line.first_point() != line.last_point() {
                lines.push(line);
            }
        }
        lines
    }

    /// `bool split_at_index(const size_t index, Polyline* p1, Polyline* p2) const`
    /// (`Polyline.cpp:277-307`).
    /// // TODO(port): src/libslic3r/Polyline.cpp:295-297,302-304 --
    /// `split_fitting_result_before_index`/`_after_index` omitted (both private helpers manage
    /// only `fitting_result`, including re-deriving a clipped arc endpoint); see the module doc.
    pub fn split_at_index(&self, index: usize, p1: &mut Polyline, p2: &mut Polyline) -> bool {
        // PORT NOTE: src/libslic3r/Polyline.cpp:279 -- `index > this->size() - 1` is C++ `size_t`
        // wraparound: for an empty polyline `size() - 1` wraps to `SIZE_MAX` and the comparison is
        // always false, so the vendored body falls through to `first_point()` on an empty vector
        // (undefined behaviour, no in-scope caller reaching it). `wrapping_sub` reproduces the
        // same comparison; the fall-through still panics on an empty `self` here, matching the
        // vendored UB rather than guarding against it with no vendored counterpart to guard
        // against it with.
        if index > self.points.len().wrapping_sub(1) {
            return false;
        }
        let n = self.points.len();
        if index == 0 {
            p1.clear();
            p1.append(self.first_point());
            *p2 = self.clone();
        } else if index == n - 1 {
            p2.clear();
            p2.append(self.last_point());
            *p1 = self.clone();
        } else {
            p1.clear();
            p1.points.reserve(index + 1);
            p1.points.extend_from_slice(&self.points[..=index]);
            p2.clear();
            p2.points.reserve(n - index);
            p2.points.extend_from_slice(&self.points[index..]);
        }
        true
    }

    /// `bool split_at_length(const double length, Polyline* p1, Polyline* p2) const`
    /// (`Polyline.cpp:309-355`).
    pub fn split_at_length(&self, length: f64, p1: &mut Polyline, p2: &mut Polyline) -> bool {
        if self.points.is_empty() {
            return false;
        }
        // `0f64`: the vendored `if (length < 0 || ...)` (`Polyline.cpp:312`) spells its own
        // literal as the bare integer `0`; `0f64` normalises to that same exempt `"0"` text.
        if length < 0f64 || length > self.length() {
            return false;
        }
        let scaled_eps = slic3r_libslic3r::libslic3r::scaled_epsilon();
        if length < scaled_eps {
            p1.clear();
            p1.append(self.first_point());
            *p2 = self.clone();
        } else if slic3r_libslic3r::libslic3r::is_approx(length, self.length(), scaled_eps) {
            p2.clear();
            p2.append(self.last_point());
            *p1 = self.clone();
        } else {
            let mut line_idx: usize = 0;
            // `0f64`: the vendored `double acc_length = 0;` (`Polyline.cpp:327`) is the same
            // bare-integer literal.
            let mut acc_length = 0f64;
            let mut p = self.first_point();
            for l in self.lines() {
                p = l.b;
                let current_length = l.length();
                if acc_length + current_length >= length {
                    p = slic3r_point::point::lerp_point(
                        l.a,
                        l.b,
                        (length - acc_length) / current_length,
                    );
                    break;
                }
                acc_length += current_length;
                line_idx += 1;
            }
            let index = self.find_point(p);
            if index != -1 {
                self.split_at_index(index as usize, p1, p2);
            } else {
                let mut temp = Polyline::new();
                self.split_at_index(line_idx, p1, &mut temp);
                p1.append(p);
                self.split_at_index(line_idx + 1, &mut temp, p2);
                p2.append_before(p);
            }
        }
        true
    }

    /// `bool is_closed() const { return this->points.front() == this->points.back(); }` (`:132`).
    pub fn is_closed(&self) -> bool {
        self.points[0] == *self.points.last().unwrap()
    }
}

impl MultiPoint for Polyline {
    fn points(&self) -> &Points {
        &self.points
    }
    fn points_mut(&mut self) -> &mut Points {
        &mut self.points
    }
    /// `const Point& last_point() const override { return this->points.back(); }` (`:114`).
    fn last_point(&self) -> Point {
        *self.points.last().unwrap()
    }
    /// `Lines lines() const override;` (`:116`, defined `Polyline.cpp:31-41`): the open chain,
    /// through [`to_lines`] -- identical body to the free `to_lines(const Polyline&)`
    /// (`Polyline.hpp:175-184`), so both share [`points_to_lines`].
    fn lines(&self) -> Lines {
        to_lines(self)
    }
}

/// `inline Lines to_lines(const Polyline &poly)` (`Polyline.hpp:175-184`); see
/// [`MultiPoint::lines`] for `Polyline`'s own identical-bodied override.
pub fn to_lines(poly: &Polyline) -> Lines {
    points_to_lines(&poly.points)
}

/// The open-chain line sequence both [`to_lines`] and [`ThickPolyline::lines`] build: every
/// consecutive pair, with **no** closing edge back to the first point (unlike
/// `crates/slic3r-polygon`'s own `to_lines`, which is this unit's closed-ring sibling).
fn points_to_lines(points: &Points) -> Lines {
    let mut lines = Lines::new();
    if points.len() >= 2 {
        lines.reserve(points.len() - 1);
        for i in 0..points.len() - 1 {
            lines.push(Line::new(points[i], points[i + 1]));
        }
    }
    lines
}

/// `inline Lines to_lines(const Polylines &polys)` (`Polyline.hpp:186-200`).
// PORT NOTE: src/libslic3r/Polyline.hpp:196 -- the vendored body's own second loop walks
// `poly.points.begin()..poly.points.end()-1`, which is undefined behaviour only for an empty
// (0-point) member (`end()-1` before `begin()`); for a 1-point member `begin() == end()-1`, so
// the loop body never runs and the two already agree there without any guard. This function
// instead reuses `to_lines`'s own `len >= 2` guard for the empty case specifically (this
// module's one place that guards vendored UB rather than reproducing it, unlike
// `split_at_index`'s wraparound and `to_thick_polylines`'s `size() - 1`, both of which keep the
// vendored unguarded shape) -- a safe, empty-for-degenerate-input rendering rather than
// reproducing the vendored UB, since no in-scope caller passes an empty member and the two give
// the same result for every other input.
pub fn to_lines_polylines(polys: &Polylines) -> Lines {
    let mut n_lines = 0usize;
    for poly in polys {
        n_lines += poly.points.len().saturating_sub(1);
    }
    let mut lines = Lines::new();
    lines.reserve(n_lines);
    for poly in polys {
        lines.extend(to_lines(poly));
    }
    lines
}

/// `inline Polylines to_polylines(const std::vector<Points> &paths)` (`:202-209`).
pub fn to_polylines(paths: &[Points]) -> Polylines {
    let mut out = Polylines::new();
    out.reserve(paths.len());
    for path in paths {
        out.push(Polyline::from_points(path.clone()));
    }
    out
}

/// `inline Polylines to_polylines(std::vector<Points> &&paths)` (`:211-218`).
pub fn to_polylines_moved(paths: Vec<Points>) -> Polylines {
    let mut out = Polylines::new();
    out.reserve(paths.len());
    for path in paths {
        out.push(Polyline::from_points(path));
    }
    out
}

/// `inline void polylines_append(Polylines &dst, const Polylines &src)` (`:220-223`).
pub fn polylines_append(dst: &mut Polylines, src: &Polylines) {
    dst.extend_from_slice(src);
}

/// `inline void polylines_append(Polylines &dst, Polylines &&src)` (`:225-233`).
pub fn polylines_append_moved(dst: &mut Polylines, mut src: Polylines) {
    if dst.is_empty() {
        *dst = src;
    } else {
        dst.append(&mut src);
    }
}

/// `template<typename PointsType> inline void polylines_merge(PointsType &dst, bool dst_first,
/// PointsType &&src, bool src_first)` (`:240-252`); see the module doc for why this is ported in
/// full as a genuine Rust generic rather than deferred, through
/// [`slic3r_libslic3r::libslic3r::append`](slic3r_libslic3r::libslic3r::append) for the vendored
/// body's own free `append(dst, std::move(src))` call (`:251`).
pub fn polylines_merge<T>(dst: &mut Vec<T>, dst_first: bool, mut src: Vec<T>, src_first: bool) {
    if dst_first {
        if src_first {
            dst.reverse();
        } else {
            std::mem::swap(dst, &mut src);
        }
    } else if !src_first {
        src.reverse();
    }
    slic3r_libslic3r::libslic3r::append(dst, src);
}

/// `extern BoundingBox get_extents(const Polyline &polyline);` (`:161`, defined
/// `Polyline.cpp:528-531`).
pub fn get_extents(polyline: &Polyline) -> BoundingBox {
    polyline.bounding_box()
}

/// `extern BoundingBox get_extents(const Polylines &polylines);` (`:162`, defined
/// `Polyline.cpp:533-542`).
pub fn get_extents_polylines(polylines: &Polylines) -> BoundingBox {
    let mut bb = BoundingBox::new();
    if !polylines.is_empty() {
        bb = polylines[0].bounding_box();
        for poly in &polylines[1..] {
            bb.merge_points(&poly.points);
        }
    }
    bb
}

/// `bool remove_same_neighbor(Polyline &polyline);` (`:165`, defined `Polyline.cpp:545-557`):
/// `std::unique` over `==` is exactly `Vec::dedup`'s own default comparator.
pub fn remove_same_neighbor(polyline: &mut Polyline) -> bool {
    if polyline.points.is_empty() {
        return false;
    }
    let before = polyline.points.len();
    polyline.points.dedup();
    polyline.points.len() != before
}

/// `bool remove_same_neighbor(Polylines &polylines);` (`:166`, defined `Polyline.cpp:559-568`).
pub fn remove_same_neighbor_polylines(polylines: &mut Polylines) -> bool {
    if polylines.is_empty() {
        return false;
    }
    let mut exist = false;
    for poly in polylines.iter_mut() {
        exist |= remove_same_neighbor(poly);
    }
    polylines.retain(|p| p.points.len() > 1);
    exist
}

/// `inline double total_length(const Polylines &polylines)` (`:168-173`).
pub fn total_length(polylines: &Polylines) -> f64 {
    // `0f64`: the vendored `double total = 0;` (`Polyline.hpp:169`) spells its own literal as
    // the bare integer `0`; `0f64` normalises to that same exempt `"0"` text.
    let mut total = 0f64;
    for pl in polylines {
        total += pl.length();
    }
    total
}

/// `const Point& leftmost_point(const Polylines &polylines);` (`:254`, defined
/// `Polyline.cpp:571-583`): `throw Slic3r::InvalidArgument(...)` on an empty collection -- the
/// idiom map's error-set rule renders the same `throw`/`catch` pair as `Result`/`Err`.
pub fn leftmost_point(
    polylines: &Polylines,
) -> Result<Point, slic3r_exception::exception::Exception> {
    if polylines.is_empty() {
        return Err(slic3r_exception::exception::Exception::InvalidArgument(
            "leftmost_point() called on empty PolylineCollection".to_string(),
        ));
    }
    let mut p = polylines[0].leftmost_point();
    for poly in &polylines[1..] {
        let p2 = poly.leftmost_point();
        if p2.x() < p.x() {
            p = p2;
        }
    }
    Ok(p)
}

/// `bool remove_degenerate(Polylines &polylines);` (`:256`, defined `Polyline.cpp:585-600`): the
/// vendored body swaps only the `.points` member at each surviving slot, which this struct's one
/// field makes identical to swapping the whole `Polyline`.
pub fn remove_degenerate(polylines: &mut Polylines) -> bool {
    let mut modified = false;
    let mut j = 0usize;
    for i in 0..polylines.len() {
        if polylines[i].points.len() >= 2 {
            if j < i {
                polylines.swap(i, j);
            }
            j += 1;
        } else {
            modified = true;
        }
    }
    if j < polylines.len() {
        polylines.truncate(j);
    }
    modified
}

// TODO(port): src/libslic3r/Polyline.cpp:232-274 -- `Polyline::split_at(Point&, Polyline*,
// Polyline*)`, needs `Point::projection_onto(const Line&)`, left unported in
// `crates/slic3r-point/src/point.rs:556` (a sibling this task does not own). See the module doc.
// TODO(port): src/libslic3r/Polyline.cpp:357-367 -- `Polyline::is_straight`, needs
// `Line::parallel_to(double)`, left unported in `crates/slic3r-line/src/line.rs:422` (needs
// `slic3r-geometry`, not a dependency of that crate either). See the module doc.
// TODO(port): src/libslic3r/Polyline.cpp:161-165 -- `Polyline::simplify_by_fitting_arc`, needs
// `ArcFitter::do_arc_fitting_and_simplify` (`slic3r-arcfitter`, a fan-out crate not drafted yet).
// See the module doc.
// TODO(port): src/libslic3r/Polyline.cpp:452-457 -- `Polyline::reset_to_linear_move`, needs
// `PathFittingData` (same crate). See the module doc.
// TODO(port): src/libslic3r/Polyline.cpp:602-622 -- `foot_pt(const Points&, const Point&)`, needs
// `Point::projection_onto(const Line&)`, same gap as `split_at` above.
// TODO(port): src/libslic3r/Polyline.cpp:624-644 -- `foot_pt(const Points3&, const Point3&)`,
// needs `Point3::projection_onto(const Line3&)`, left unported in `point.rs` for the same reason.

// -------------------------------------------------------------------------------------------
// ThickPolyline (Polyline.hpp:262-283, Polyline.cpp)
// -------------------------------------------------------------------------------------------

/// `class ThickPolyline : public Polyline { public: ... };` (`Polyline.hpp:262-283`); see the
/// module doc for why this is its own struct and its own `MultiPoint` implementor rather than a
/// wrapper around [`Polyline`].
#[derive(Debug, Default, Clone)]
pub struct ThickPolyline {
    pub points: Points,
    pub width: Vec<f64>,
    pub endpoints: (bool, bool),
}

/// `typedef std::vector<ThickPolyline> ThickPolylines;` (`:23`).
pub type ThickPolylines = Vec<ThickPolyline>;

impl ThickPolyline {
    /// `ThickPolyline() : endpoints(std::make_pair(false, false)) {}` (`:264`).
    pub fn new() -> Self {
        ThickPolyline {
            points: Points::new(),
            width: Vec::new(),
            endpoints: (false, false),
        }
    }

    /// `ThickLines thicklines() const` (`Polyline.cpp:646-655`).
    pub fn thicklines(&self) -> ThickLines {
        let mut lines = ThickLines::new();
        if self.points.len() >= 2 {
            lines.reserve(self.points.len() - 1);
            for i in 0..self.points.len() - 1 {
                lines.push(ThickLine::with_widths(
                    self.points[i],
                    self.points[i + 1],
                    self.width[2 * i],
                    self.width[2 * i + 1],
                ));
            }
        }
        lines
    }

    /// `void reverse()` (`:266-270`): hides `MultiPoint::reverse` (and, in C++, `Polyline::reverse`).
    pub fn reverse(&mut self) {
        self.points.reverse();
        self.width.reverse();
        std::mem::swap(&mut self.endpoints.0, &mut self.endpoints.1);
    }

    /// `void clear()` (`:271-274`): hides `MultiPoint::clear` (and `Polyline::clear`).
    pub fn clear(&mut self) {
        self.points.clear();
        self.width.clear();
    }

    /// `void start_at_index(int index)` (`Polyline.cpp:657-668`): rotates a *closed* `ThickPolyline`
    /// (`points.front() == points.back()`) to start at `index`; a no-op on an open one, per the
    /// vendored guard.
    pub fn start_at_index(&mut self, index: i32) {
        // PORT SRC: src/libslic3r/Polyline.cpp:659
        debug_assert!(index >= 0 && (index as usize) < self.points.len());
        // PORT SRC: src/libslic3r/Polyline.cpp:660
        debug_assert!(
            self.points.first() == self.points.last() && self.width.first() == self.width.last()
        );
        let n = self.points.len();
        if index != 0
            && (index as usize) + 1 != n
            && self.points.first() == self.points.last()
            && self.width.first() == self.width.last()
        {
            self.points.pop();
            debug_assert_eq!(self.points.len() * 2, self.width.len());
            self.points.rotate_left(index as usize);
            self.width.rotate_left(2 * index as usize);
            let front = self.points[0];
            self.points.push(front);
        }
    }

    /// Moves `self.points` into a temporary [`Polyline`], runs `f` on it, and writes the result
    /// back -- see the module doc for why this reproduces non-virtual dispatch exactly rather
    /// than merely approximating it.
    fn via_polyline<R>(&mut self, f: impl FnOnce(&mut Polyline) -> R) -> R {
        let mut pl = Polyline::from_points(std::mem::take(&mut self.points));
        let r = f(&mut pl);
        self.points = pl.points;
        r
    }

    /// Inherited from `Polyline::append(const Point&)` (`Polyline.hpp:65-71`): hides
    /// `MultiPoint::append` the same way `Polyline::append` does. **The fix for a real
    /// divergence found by review**: without this inherent method, an unqualified
    /// `thick_polyline.append(point)` resolved to the non-dedup `MultiPoint::append` trait
    /// default, where C++ resolves `tp.append(point)` (`tp: ThickPolyline`) to this dedup-ing
    /// method, since `append` is not `virtual` and the lookup is static, not dynamic.
    pub fn append(&mut self, point: Point) {
        self.via_polyline(|pl| pl.append(point));
    }

    /// Inherited from `Polyline::append_before` (`Polyline.hpp:73-86`).
    pub fn append_before(&mut self, point: Point) {
        self.via_polyline(|pl| pl.append_before(point));
    }

    /// Inherited from `Polyline::append(const Points&)` (`Polyline.hpp:88-94`).
    pub fn append_points(&mut self, src: &[Point]) {
        self.via_polyline(|pl| pl.append_points(src));
    }

    /// Inherited from `Polyline::append(begin, end)` (`Polyline.hpp:95-102`).
    pub fn append_points_range(&mut self, src: &[Point]) {
        self.via_polyline(|pl| pl.append_points_range(src));
    }

    /// Inherited from `Polyline::append(const Polyline&)` (`Polyline.cpp:369-390`).
    pub fn append_polyline(&mut self, src: &Polyline) {
        self.via_polyline(|pl| pl.append_polyline(src));
    }

    /// Inherited from `Polyline::append(Polyline&&)` (`Polyline.cpp:392-415`).
    pub fn append_polyline_moved(&mut self, src: Polyline) {
        self.via_polyline(|pl| pl.append_polyline_moved(src));
    }

    /// Inherited from `Polyline::leftmost_point` (`Polyline.cpp:21-29`).
    pub fn leftmost_point(&self) -> Point {
        Polyline::from_points(self.points.clone()).leftmost_point()
    }

    /// Inherited from `Polyline::clip_end` (`Polyline.cpp:61-101`). Named in-scope caller:
    /// `src/libslic3r/Fill/FillConcentric.cpp:183` --
    /// `thick_polylines_out[i].clip_end(this->loop_clipping);` (crate `slic3r-fill`,
    /// `not-started` at the pin) -- the gap review found this method missing over.
    pub fn clip_end(&mut self, distance: f64) {
        self.via_polyline(|pl| pl.clip_end(distance));
    }

    /// Inherited from `Polyline::clip_start` (`Polyline.cpp:104-110`).
    pub fn clip_start(&mut self, distance: f64) {
        self.via_polyline(|pl| pl.clip_start(distance));
    }

    /// Inherited from `Polyline::extend_end` (`Polyline.cpp:112-118`).
    pub fn extend_end(&mut self, distance: f64) {
        self.via_polyline(|pl| pl.extend_end(distance));
    }

    /// Inherited from `Polyline::extend_start` (`Polyline.cpp:120-125`).
    pub fn extend_start(&mut self, distance: f64) {
        self.via_polyline(|pl| pl.extend_start(distance));
    }

    /// Inherited from `Polyline::equally_spaced_points` (`Polyline.cpp:127-153`).
    pub fn equally_spaced_points(&self, distance: f64) -> Points {
        Polyline::from_points(self.points.clone()).equally_spaced_points(distance)
    }

    /// Inherited from `Polyline::equally_spaced_lines` (`Polyline.cpp:167-206`).
    pub fn equally_spaced_lines(&self, distance: f64) -> Polylines {
        Polyline::from_points(self.points.clone()).equally_spaced_lines(distance)
    }

    /// Inherited from `Polyline::simplify` (`Polyline.cpp:155-159`).
    pub fn simplify(&mut self, tolerance: f64) {
        self.via_polyline(|pl| pl.simplify(tolerance));
    }

    /// Inherited from `Polyline::split_at_index` (`Polyline.cpp:277-307`).
    pub fn split_at_index(&self, index: usize, p1: &mut Polyline, p2: &mut Polyline) -> bool {
        Polyline::from_points(self.points.clone()).split_at_index(index, p1, p2)
    }

    /// Inherited from `Polyline::split_at_length` (`Polyline.cpp:309-355`).
    pub fn split_at_length(&self, length: f64, p1: &mut Polyline, p2: &mut Polyline) -> bool {
        Polyline::from_points(self.points.clone()).split_at_length(length, p1, p2)
    }

    /// Inherited from `Polyline::is_closed` (`Polyline.hpp:132`).
    pub fn is_closed(&self) -> bool {
        self.points[0] == *self.points.last().unwrap()
    }
}

impl std::ops::Index<usize> for ThickPolyline {
    type Output = Point;
    /// Inherited from `Polyline::operator[](Points::size_type) const` (`Polyline.hpp:112`).
    fn index(&self, idx: usize) -> &Point {
        &self.points[idx]
    }
}
impl std::ops::IndexMut<usize> for ThickPolyline {
    /// Inherited from `Polyline::operator[](Points::size_type)` (`Polyline.hpp:111`).
    fn index_mut(&mut self, idx: usize) -> &mut Point {
        &mut self.points[idx]
    }
}

impl MultiPoint for ThickPolyline {
    fn points(&self) -> &Points {
        &self.points
    }
    fn points_mut(&mut self) -> &mut Points {
        &mut self.points
    }
    /// `Point& last_point()`/its `const` overload, inherited from `Polyline::last_point()`
    /// (`Polyline.hpp:114`): `points.back()`.
    fn last_point(&self) -> Point {
        *self.points.last().unwrap()
    }
    /// `Lines lines() const`, inherited from `Polyline::lines()` (`Polyline.cpp:31-41`): the same
    /// open chain [`to_lines`]/[`points_to_lines`] build.
    fn lines(&self) -> Lines {
        points_to_lines(&self.points)
    }
}

/// `inline ThickPolylines to_thick_polylines(Polylines&& polylines, const coordf_t width)`
/// (`Polyline.hpp:285-295`).
/// // PORT NOTE: src/libslic3r/Polyline.hpp:291 -- `(polyline.points.size() - 1) * 2` is C++
/// `size_t` arithmetic: for an empty `polyline` (no in-scope caller passes one) `size() - 1`
/// wraps to a huge count and `std::vector::assign` would abort/OOM. Plain `usize` subtraction
/// reproduces the same unguarded shape (panicking in a debug build) rather than silently
/// returning an empty `width` for an input the vendored body has no defined behaviour for either.
pub fn to_thick_polylines(polylines: Polylines, width: f64) -> ThickPolylines {
    let mut out = ThickPolylines::new();
    out.reserve(polylines.len());
    for polyline in polylines {
        let n = polyline.points.len();
        let mut tp = ThickPolyline::new();
        tp.width = vec![width; (n - 1) * 2];
        tp.points = polyline.points;
        out.push(tp);
    }
    out
}

// -------------------------------------------------------------------------------------------
// Polyline3 (Polyline.hpp:297-357, Polyline.cpp)
// -------------------------------------------------------------------------------------------

/// `class Polyline3 : public MultiPoint3 { public: ... };` (`Polyline.hpp:297-357`); see the
/// module doc for why `fitting_result` (`:350`) is not a field here.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Polyline3 {
    pub points: Points3,
}

/// `typedef std::vector<Polyline3> Polylines3;` (`:359`).
pub type Polylines3 = Vec<Polyline3>;

impl Eq for Polyline3 {}

impl Polyline3 {
    /// `Polyline3() {}` (`:300`).
    pub fn new() -> Self {
        Polyline3 {
            points: Points3::new(),
        }
    }

    /// `explicit Polyline3(const Points3 &points)` (`:301`).
    pub fn from_points3(points: Points3) -> Self {
        Polyline3 { points }
    }

    /// `explicit Polyline3(const Polyline &poly, coord_t z = 0)` (`:302-307`).
    pub fn from_polyline(poly: &Polyline, z: i64) -> Self {
        let mut out = Polyline3::new();
        out.points.reserve(poly.points.len());
        for pt in &poly.points {
            out.points.push(Point3::new(pt.x(), pt.y(), z));
        }
        out
    }

    /// `Polyline to_polyline() const` (`Polyline.cpp:682-690`): drops the Z coordinate.
    pub fn to_polyline(&self) -> Polyline {
        let mut out = Polyline::new();
        out.points.reserve(self.points.len());
        for pt in &self.points {
            out.points.push(pt.to_point());
        }
        out
    }

    /// `void clip_end(double distance)` (`Polyline.cpp:692-733`); see [`Polyline::clip_end`]'s
    /// identical 2D shape.
    /// // TODO(port): src/libslic3r/Polyline.cpp:714-731 -- `fitting_result` clipping omitted; see
    /// the module doc.
    pub fn clip_end(&mut self, mut distance: f64) {
        // `0f64`: the vendored `while (distance > 0)` (`Polyline.cpp:696`) spells its own literal
        // as the bare integer `0`; see the 2D `Polyline::clip_end`'s identical comment above.
        while distance > 0f64 {
            let last = self.last_point();
            let last_f =
                slic3r_point::point::Vec3d::new(last.x() as f64, last.y() as f64, last.z() as f64);
            self.points.pop();
            if self.points.is_empty() {
                return;
            }
            let cur = self.last_point();
            let vx = cur.x() as f64 - last_f.x();
            let vy = cur.y() as f64 - last_f.y();
            let vz = cur.z() as f64 - last_f.z();
            let lsqr = vx * vx + vy * vy + vz * vz;
            if lsqr > distance * distance {
                let scale = distance / lsqr.sqrt();
                self.points.push(Point3::new(
                    (last_f.x() + vx * scale) as i64,
                    (last_f.y() + vy * scale) as i64,
                    (last_f.z() + vz * scale) as i64,
                ));
                break;
            }
            distance -= lsqr.sqrt();
        }
    }

    /// `void simplify(double tolerance)` (`Polyline.cpp:735-739`), through
    /// [`multi_point::douglas_peucker3`].
    pub fn simplify(&mut self, tolerance: f64) {
        self.points = multi_point::douglas_peucker3(&self.points, tolerance);
    }

    /// `void reverse()` (`Polyline.cpp:749-764`); see [`Polyline::reverse`]'s identical 2D shape.
    /// // TODO(port): src/libslic3r/Polyline.cpp:753-763 -- `fitting_result` reversal omitted; see
    /// the module doc.
    pub fn reverse(&mut self) {
        self.points.reverse();
    }

    /// `bool split_at_index(const size_t index, Polyline3 *p1, Polyline3 *p2) const`
    /// (`Polyline.cpp:766-796`); see [`Polyline::split_at_index`]'s identical 2D shape and its own
    /// doc for the `size_t` wraparound note, which applies here the same way.
    /// // TODO(port): src/libslic3r/Polyline.cpp:784-786,791-793 --
    /// `split_fitting_result_before_index`/`_after_index` omitted; see the module doc.
    pub fn split_at_index(&self, index: usize, p1: &mut Polyline3, p2: &mut Polyline3) -> bool {
        if index > self.points.len().wrapping_sub(1) {
            return false;
        }
        let n = self.points.len();
        if index == 0 {
            p1.clear();
            p1.append(self.first_point());
            *p2 = self.clone();
        } else if index == n - 1 {
            p2.clear();
            p2.append(self.last_point());
            *p1 = self.clone();
        } else {
            p1.clear();
            p1.points.reserve(index + 1);
            p1.points.extend_from_slice(&self.points[..=index]);
            p2.clear();
            p2.points.reserve(n - index);
            p2.points.extend_from_slice(&self.points[index..]);
        }
        true
    }

    /// `bool split_at_length(const double length, Polyline3 *p1, Polyline3 *p2) const`
    /// (`Polyline.cpp:901-946`); see [`Polyline::split_at_length`]'s identical 2D shape.
    pub fn split_at_length(&self, length: f64, p1: &mut Polyline3, p2: &mut Polyline3) -> bool {
        if self.points.is_empty() {
            return false;
        }
        // `0f64`: the vendored `if (length < 0 || ...)` (`Polyline.cpp:903`) spells its own
        // literal as the bare integer `0`; see the 2D `split_at_length`'s identical comment.
        if length < 0f64 || length > self.length() {
            return false;
        }
        let scaled_eps = slic3r_libslic3r::libslic3r::scaled_epsilon();
        if length < scaled_eps {
            p1.clear();
            p1.append(self.first_point());
            *p2 = self.clone();
        } else if slic3r_libslic3r::libslic3r::is_approx(length, self.length(), scaled_eps) {
            p2.clear();
            p2.append(self.last_point());
            *p1 = self.clone();
        } else {
            let mut line_idx: usize = 0;
            // `0f64`: the vendored `double acc_length = 0;` (`Polyline.cpp:918`) is the same
            // bare-integer literal the 2D `split_at_length` cites.
            let mut acc_length = 0f64;
            let mut p = self.first_point();
            for l in self.lines() {
                p = l.b;
                let current_length = l.length();
                if acc_length + current_length >= length {
                    p = slic3r_point::point::lerp_point3(
                        l.a,
                        l.b,
                        (length - acc_length) / current_length,
                    );
                    break;
                }
                acc_length += current_length;
                line_idx += 1;
            }
            let index = self.find_point3(p);
            if index != -1 {
                self.split_at_index(index as usize, p1, p2);
            } else {
                let mut temp = Polyline3::new();
                self.split_at_index(line_idx, p1, &mut temp);
                p1.append(p);
                self.split_at_index(line_idx + 1, &mut temp, p2);
                p2.append_before(p);
            }
        }
        true
    }

    /// `void append(const Point3& point)` (`Polyline.cpp:798-812`).
    /// // TODO(port): src/libslic3r/Polyline.cpp:811 -- `append_fitting_result_after_append_points()`
    /// omitted; see the module doc.
    pub fn append(&mut self, point: Point3) {
        if !self.is_empty() && self.last_point() == point {
            return;
        }
        self.points.push(point);
    }

    /// `void append(const Polyline3& src)` (`Polyline.cpp:814-834`).
    /// // TODO(port): src/libslic3r/Polyline.cpp:820,826-828,832 -- `fitting_result` copy/merge
    /// bookkeeping omitted; see the module doc.
    pub fn append_polyline3(&mut self, src: &Polyline3) {
        if !src.is_valid() {
            return;
        }
        if self.points.is_empty() {
            self.points = src.points.clone();
        } else {
            self.append(src.points[0]);
            self.points.extend_from_slice(&src.points[1..]);
        }
    }

    /// `void append_before(const Point3& point)` (`Polyline.cpp:836-850`).
    pub fn append_before(&mut self, point: Point3) {
        if !self.is_empty() && self.first_point() == point {
            return;
        }
        if self.points.len() == 1 {
            self.points.push(point);
            self.points.reverse();
        } else {
            self.reverse();
            self.append(point);
            self.reverse();
        }
    }
}

impl MultiPoint3 for Polyline3 {
    fn points(&self) -> &Points3 {
        &self.points
    }
    fn points_mut(&mut self) -> &mut Points3 {
        &mut self.points
    }
    /// `virtual Lines3 lines() const;` (`:309`, defined `Polyline.cpp:670-680`): the open chain,
    /// the 3D analogue of [`points_to_lines`].
    fn lines(&self) -> Lines3 {
        let mut lines = Lines3::new();
        if self.points.len() >= 2 {
            lines.reserve(self.points.len() - 1);
            for i in 0..self.points.len() - 1 {
                lines.push(Line3::new(self.points[i], self.points[i + 1]));
            }
        }
        lines
    }
}

// TODO(port): src/libslic3r/Polyline.cpp:741-747 -- `Polyline3::simplify_by_fitting_arc`, needs
// `ArcFitter::do_arc_fitting_and_simplify` (`slic3r-arcfitter`, a fan-out crate not drafted yet).
// See the module doc.
// TODO(port): src/libslic3r/Polyline.cpp:852-893 -- `Polyline3::split_at(Point&, ...)`, needs
// `Point::projection_onto(const Line&)`/`.to_line()` on a `Line3`, same gap as 2D `split_at`.
// TODO(port): src/libslic3r/Polyline.cpp:895-899 -- `Polyline3::split_at(Point3&, ...)`, forwards
// to the overload above; deferred with it.

#[cfg(test)]
mod tests {
    use super::*;
    use slic3r_proof::{compare_unit, Oracle};

    #[test]
    fn polyline_append_dedups_the_last_point() {
        let mut pl = Polyline::new();
        pl.append(Point::new(0, 0));
        pl.append(Point::new(0, 0));
        pl.append(Point::new(10, 0));
        assert_eq!(pl.points, vec![Point::new(0, 0), Point::new(10, 0)]);
    }

    #[test]
    fn polyline_append_before_dedups_the_first_point() {
        let mut pl = Polyline::from_points(vec![Point::new(10, 0)]);
        pl.append_before(Point::new(0, 0));
        assert_eq!(pl.points, vec![Point::new(0, 0), Point::new(10, 0)]);
        pl.append_before(Point::new(0, 0));
        assert_eq!(pl.points, vec![Point::new(0, 0), Point::new(10, 0)]);
    }

    #[test]
    fn polyline_lines_is_open_not_closed() {
        let pl = Polyline::from_points(vec![
            Point::new(0, 0),
            Point::new(10, 0),
            Point::new(10, 10),
        ]);
        let lines = pl.lines();
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0], Line::new(Point::new(0, 0), Point::new(10, 0)));
        assert_eq!(lines[1], Line::new(Point::new(10, 0), Point::new(10, 10)));
    }

    #[test]
    fn polyline_reverse_and_is_closed() {
        let mut pl =
            Polyline::from_points(vec![Point::new(0, 0), Point::new(10, 0), Point::new(0, 0)]);
        assert!(pl.is_closed());
        pl.reverse();
        assert_eq!(
            pl.points,
            vec![Point::new(0, 0), Point::new(10, 0), Point::new(0, 0)]
        );
    }

    #[test]
    fn polyline_clip_end_removes_from_the_tail() {
        let mut pl = Polyline::from_points(vec![
            Point::new(0, 0),
            Point::new(1000, 0),
            Point::new(1000, 1000),
        ]);
        pl.clip_end(500.0);
        assert_eq!(
            pl.points,
            vec![Point::new(0, 0), Point::new(1000, 0), Point::new(1000, 500),]
        );
    }

    #[test]
    fn polyline_clip_end_drops_whole_segments() {
        // Total path length is 2000 (two 1000-long segments); clipping 1500 off the end leaves
        // 500 of path measured from the start, i.e. the midpoint of the first segment.
        let mut pl = Polyline::from_points(vec![
            Point::new(0, 0),
            Point::new(1000, 0),
            Point::new(1000, 1000),
        ]);
        pl.clip_end(1500.0);
        assert_eq!(pl.points, vec![Point::new(0, 0), Point::new(500, 0)]);
    }

    #[test]
    fn polyline_split_at_index_splits_the_points() {
        let pl = Polyline::from_points(vec![
            Point::new(0, 0),
            Point::new(10, 0),
            Point::new(20, 0),
            Point::new(30, 0),
        ]);
        let mut p1 = Polyline::new();
        let mut p2 = Polyline::new();
        assert!(pl.split_at_index(1, &mut p1, &mut p2));
        assert_eq!(p1.points, vec![Point::new(0, 0), Point::new(10, 0)]);
        assert_eq!(
            p2.points,
            vec![Point::new(10, 0), Point::new(20, 0), Point::new(30, 0)]
        );
    }

    #[test]
    fn polylines_merge_generic_matches_the_vendored_shape() {
        let dst = vec![1, 2, 3];
        let src = vec![4, 5, 6];
        let mut d = dst.clone();
        polylines_merge(&mut d, true, src.clone(), true);
        let mut expected = dst.clone();
        expected.reverse();
        expected.extend(src.clone());
        assert_eq!(d, expected);
    }

    #[test]
    fn total_length_sums_every_polyline() {
        let pls = vec![
            Polyline::from_points(vec![Point::new(0, 0), Point::new(1000, 0)]),
            Polyline::from_points(vec![Point::new(0, 0), Point::new(0, 500)]),
        ];
        assert_eq!(total_length(&pls), 1500.0);
    }

    #[test]
    fn leftmost_point_errs_on_an_empty_collection() {
        let pls: Polylines = Vec::new();
        assert!(leftmost_point(&pls).is_err());
    }

    #[test]
    fn to_thick_polylines_assigns_widths_per_segment() {
        let pls = vec![Polyline::from_points(vec![
            Point::new(0, 0),
            Point::new(10, 0),
            Point::new(20, 0),
        ])];
        let out = to_thick_polylines(pls, 42.0);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].width, vec![42.0, 42.0, 42.0, 42.0]);
    }

    /// `ThickPolyline::clip_end` -- the inherited `Polyline::clip_end` method found missing by
    /// review (`src/libslic3r/Fill/FillConcentric.cpp:183` calls it on a `ThickPolyline`) --
    /// reaches `points` exactly like `Polyline::clip_end` above and leaves `width`/`endpoints`
    /// untouched, matching the vendored non-virtual call (`this->reverse()`/other calls inside
    /// `Polyline::clip_end`'s own body resolve to `Polyline`'s methods even when `this` is a
    /// `ThickPolyline`, never touching the subclass's extra fields).
    #[test]
    fn thick_polyline_clip_end_matches_polyline_and_leaves_width_alone() {
        // Chained to `polyline_clip_end_matches_the_oracle` (`// PROOF: 1`) below rather than
        // to a hand-computed expectation: build a `Polyline` from the same points and compare
        // `ThickPolyline::clip_end`'s result to *it*, so this test's own correctness rests on
        // `Polyline::clip_end` already being proven against the oracle, not on arithmetic
        // worked out by hand here.
        let points = vec![Point::new(0, 0), Point::new(1_000_000, 0)];
        let mut pl = Polyline::from_points(points.clone());
        pl.clip_end(400_000.0);
        let mut tp = ThickPolyline {
            points,
            width: vec![1.0, 2.0],
            endpoints: (true, false),
        };
        tp.clip_end(400_000.0);
        assert_eq!(tp.points, pl.points);
        assert_eq!(tp.width, vec![1.0, 2.0]);
        assert_eq!(tp.endpoints, (true, false));
    }

    /// `ThickPolyline::append` -- the real divergence review found: without this inherent
    /// method, an unqualified `tp.append(point)` resolved to the non-dedup
    /// `MultiPoint::append` trait default, where C++ resolves it to the dedup-ing
    /// `Polyline::append` (`Polyline.hpp:65-71`).
    #[test]
    fn thick_polyline_append_dedups_the_last_point() {
        let mut tp = ThickPolyline::new();
        tp.append(Point::new(0, 0));
        tp.append(Point::new(0, 0));
        tp.append(Point::new(10, 0));
        assert_eq!(tp.points, vec![Point::new(0, 0), Point::new(10, 0)]);
    }

    /// `ThickPolyline::append_before` through `via_polyline` on a single-point polyline --
    /// exercises `Polyline::append_before`'s `size() == 1` branch (`push` then `reverse`,
    /// rather than the general `reverse`/`append`/`reverse` path) by delegation, confirming
    /// `via_polyline`'s whole-vector move-and-write-back leaves that branch seeing the same
    /// length it would see called directly on a `Polyline`.
    #[test]
    fn thick_polyline_append_before_on_one_point_matches_polyline() {
        let mut pl = Polyline::from_points(vec![Point::new(10, 0)]);
        pl.append_before(Point::new(0, 0));
        let mut tp = ThickPolyline {
            points: vec![Point::new(10, 0)],
            width: Vec::new(),
            endpoints: (false, false),
        };
        tp.append_before(Point::new(0, 0));
        assert_eq!(tp.points, pl.points);
        assert_eq!(tp.points, vec![Point::new(0, 0), Point::new(10, 0)]);
    }

    /// Proof path 1 (R19): `Polyline::clip_end` (`Polyline.cpp:61-101`) is defined in the `.cpp`
    /// and externally linked, proven against the library `just oracle-build` compiles from the
    /// pin, through `crates/slic3r-ffi/cxx/proof_units/polyline.{h,cpp}`'s own shims.
    // PROOF: 1
    #[test]
    fn polyline_clip_end_matches_the_oracle() {
        let mut pl = Polyline::from_points(vec![
            Point::new(0, 0),
            Point::new(1_000_000, 0),
            Point::new(1_000_000, 1_000_000),
        ]);
        let xs: Vec<i64> = pl.points.iter().map(|p| p.x()).collect();
        let ys: Vec<i64> = pl.points.iter().map(|p| p.y()).collect();
        pl.clip_end(400_000.0);
        let actual: Vec<i64> = pl.points.iter().flat_map(|p| [p.x(), p.y()]).collect();
        let oracle = slic3r_proof::proof_units::polyline::path1_clip_end(&xs, &ys, 400_000.0);
        compare_unit("Polyline.cpp", Oracle::Built, actual, oracle)
            .expect("Polyline::clip_end must match the oracle's own clipping exactly");
    }

    /// Proof path 2 (R19) for the `Polyline.hpp` row. `posSlice.dump` (this row's own pipeline
    /// stage) carries no open-path field this header's surface produces -- confirmed by reading
    /// the committed dump (`port/dumps/20mm_cube-base-fff/posSlice.dump`: every `lslices`/
    /// `slices`/`raw_slices` field is a closed ring, and `thin_fills`/`fills` are `count=0` for
    /// the committed fixture) -- so this proof reaches the oracle through the same per-unit
    /// bridge proof path 1 uses, which `docs/PORTING.md`'s "Decisions recorded on 2026-11-26"
    /// (decision 2) settles as conforming: "a `proof path: 2` row whose unit writes nothing any
    /// dump records is proven through its per-unit bridge ... `ObjectID.hpp` is the merged
    /// precedent." Drives `Polyline.hpp`'s own declared-and-defined-in-the-header surface --
    /// `total_length`, `is_closed`, `new_scale` -- not `Polyline.cpp`'s (that is `// PROOF: 1`
    /// above), the distinction `crates/slic3r-format/src/format.rs`'s own review finding turns on.
    // PROOF: 2
    #[test]
    fn polyline_header_surface_matches_the_oracle() {
        let pl = Polyline::from_points(vec![
            Point::new(0, 0),
            Point::new(1_000_000, 0),
            Point::new(1_000_000, 1_000_000),
            Point::new(0, 0),
        ]);
        let xs: Vec<i64> = pl.points.iter().map(|p| p.x()).collect();
        let ys: Vec<i64> = pl.points.iter().map(|p| p.y()).collect();

        let mut polylines = Polylines::new();
        polylines.push(pl.clone());
        let actual_len = total_length(&polylines);
        let oracle_len = slic3r_proof::proof_units::polyline::path2_total_length(&xs, &ys);
        compare_unit("Polyline.hpp", Oracle::Built, actual_len, oracle_len)
            .expect("the free total_length(Polylines) must match the oracle exactly");

        let actual_closed = pl.is_closed();
        let oracle_closed = slic3r_proof::proof_units::polyline::path2_is_closed(&xs, &ys);
        assert_eq!(
            actual_closed, oracle_closed,
            "Polyline::is_closed must match the oracle"
        );

        let scale_in = vec![Vec2d::new(1.5, 2.5), Vec2d::new(3.0, -4.0)];
        let sxs: Vec<f64> = scale_in.iter().map(|p| p.x()).collect();
        let sys: Vec<f64> = scale_in.iter().map(|p| p.y()).collect();
        let actual_scale: Vec<i64> = Polyline::new_scale(&scale_in)
            .points
            .iter()
            .flat_map(|p| [p.x(), p.y()])
            .collect();
        let oracle_scale = slic3r_proof::proof_units::polyline::path2_new_scale(&sxs, &sys);
        compare_unit("Polyline.hpp", Oracle::Built, actual_scale, oracle_scale)
            .expect("Polyline::new_scale must match the oracle's own scaling exactly");
    }
}

// PORT STATUS
// source: src/libslic3r/Polyline.cpp
// source: src/libslic3r/Polyline.hpp
// confidence: medium
// tier: exact
// oracle: built
