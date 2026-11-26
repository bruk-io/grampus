//! Port of `src/libslic3r/BoundingBox.cpp`/`BoundingBox.hpp` (task `58-foundation-bounding-box`,
//! plan step 20): the port's axis-aligned bounding box types. `BoundingBox.hpp` is
//! `template-heavy: yes` in `port/manifest.tsv`; `BoundingBox.cpp` is `template-heavy: no` but is
//! hand-designed alongside it in this same task because the two are one unit.
//!
//! **Four concrete structs, not one generic type parameterized over the vendored
//! `BoundingBoxBase<PointType, APointsType>`/`BoundingBox3Base<PointType>` templates.** The
//! idiom map's template row collapses a template to the concrete instantiations in-scope code
//! actually uses, and the vendored header's own `extern template` block
//! (`BoundingBox.hpp:177-209`) names exactly four: `BoundingBoxBase<Point, Points>`
//! (`BoundingBox`), `BoundingBoxBase<Vec2d>` (`BoundingBoxf`), `BoundingBox3Base<Vec3d>`
//! (`BoundingBoxf3`), and (through `BoundingBox3`'s own, non-template constructors)
//! `BoundingBox3Base<Vec3crd>`. The remaining two listed there, `BoundingBoxBase<Vec2f>` and
//! `BoundingBox3Base<Vec3f>`, have no caller outside `src/slic3r/GUI/` at the pin (measured:
//! every non-GUI, non-test reference to `BoundingBoxf`-with-`float` is through the `Vec2d`/
//! `Vec3d` instantiations above), and R10 puts the whole GUI out of scope -- so this task ports
//! the four with an in-scope caller and leaves the other two unported, the same treatment
//! `crates/slic3r-point/src/point.rs` gives its own `Vec4<T>` ("no in-scope caller... carries no
//! arithmetic impls yet").
//!
//! A true Rust generic over all four would need `slic3r_point::point`'s `Point`/`Vec2`/`Vec3` to
//! expose a shared arithmetic surface -- component-wise min/max, a scalar multiply, a cast to
//! `f64`, a vector norm -- that module does not export today (task `56-foundation-point` gave
//! `Point`/`Vec2<T>`/`Vec3<T>` only `Add`/`Sub`/`Neg` and named accessors, per its own module
//! doc's scope line). This task's own rule ("Write only your own modules") forbids editing that
//! sibling's file to add it, so each of the four structs below is written out directly against
//! the public surface `point.rs` already exports, duplicating the handful of lines the four
//! share rather than inventing a second, parallel arithmetic trait for a sibling module's types.
//!
//! **Deferred for the same reason -- needs a type from a crate this task does not own, or a
//! dependency edge the cut graph forbids, and this task's own rule forbids inventing one:**
//! `BoundingBox::polygon`/`polygon(Polygon*)`, `BoundingBox3Base::polygon` (both need `Polygon`;
//! `port/cut.tsv` cuts the edge as `slic3r-polygon` depending on `slic3r-boundingbox`, the other
//! way round, so this crate can never reach it), `BoundingBoxf3::transformed` (needs the
//! deferred `Transform3d` `point.rs` itself leaves as `// TODO(port):`), the friend declaration
//! `get_extents_rotated` (defined in `MultiPoint.cpp`, not this unit's own file), the generic
//! `scaled<T>`/`unscaled<T>`/`cast<Tout, Tin>` templates at a `T` other than each one's own
//! default (no in-scope caller instantiates one), and the four Cereal `serialize` specialisations
//! at the bottom of `BoundingBox.hpp` (project-file (de)serialisation, under the preset/
//! project-format layer R11 and the manifest's deductions put out of scope). Each is named at its
//! own `// TODO(port):` site below, against the vendored line it would port.
//!
//! `get_extents<IncludeBoundary>(const Points&)`/`get_extents(const VecOfPoints&)`/
//! `get_extents(const std::vector<Vec2d>&)` are **not** ported here even though they need this
//! crate's own `BoundingBox`/`BoundingBoxf`: they are `Point.hpp`/`Point.cpp`'s own manifest row
//! (`port/manifest.tsv`), already merged with its own `// TODO(port):` markers naming exactly
//! this dependency direction (`crates/slic3r-point/src/point.rs:1257-1261`), and `slic3r-point`
//! cannot depend on `slic3r-boundingbox` (the cut graph runs the other way:
//! `port/cut.tsv`'s `dep slic3r-boundingbox slic3r-point 2` row). This task does not touch that
//! sibling module.

use slic3r_exception::exception::Exception;
use slic3r_point::point::{align_to_grid, Point, Vec2, Vec2d, Vec3};

// -------------------------------------------------------------------------------------------
// BoundingBox (BoundingBoxBase<Point, Points>, BoundingBox.hpp:211-233, BoundingBox.cpp)
// -------------------------------------------------------------------------------------------

/// `class BoundingBox : public BoundingBoxBase<Point, Points>` (`BoundingBox.hpp:211-233`). The
/// base's two fields (`min`, `max`, `BoundingBox.hpp:20-22`) are public data members upstream,
/// read and written directly at call sites (`bbox.min`, never through an accessor) -- ported as
/// public fields for the same reason `Point`/`Vec2`/`Vec3` keep Eigen's `.x()`/`.y()`
/// accessor-method idiom only for the *coordinate* types, not for a struct whose two members
/// already are such coordinates.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BoundingBox {
    pub min: Point,
    pub max: Point,
    pub defined: bool,
}

/// `using BoundingBoxes = std::vector<BoundingBox>;` (`:235`).
pub type BoundingBoxes = Vec<BoundingBox>;

impl BoundingBox {
    /// `BoundingBoxBase() : min(PointType::Zero()), max(PointType::Zero()), defined(false) {}`
    /// (`:24`).
    pub fn new() -> Self {
        BoundingBox {
            min: Point::zero(),
            max: Point::zero(),
            defined: false,
        }
    }

    /// `BoundingBox(const Point &pmin, const Point &pmax)` (`:225`), through the base's
    /// `BoundingBoxBase(const PointType &pmin, const PointType &pmax)` (`:25-26`): `defined` is
    /// true only when the box has strictly positive width *and* height, never merely
    /// non-negative -- a degenerate (zero-width or zero-height) box starts `defined = false`.
    pub fn from_min_max(pmin: Point, pmax: Point) -> Self {
        BoundingBox {
            min: pmin,
            max: pmax,
            defined: pmin.x() < pmax.x() && pmin.y() < pmax.y(),
        }
    }

    /// `BoundingBoxBase(const PointType &p1, const PointType &p2, const PointType &p3)`
    /// (`:27-28`): starts at `p1` (undefined), then merges `p2` and `p3` in order.
    pub fn from_three_points(p1: Point, p2: Point, p3: Point) -> Self {
        let mut out = BoundingBox {
            min: p1,
            max: p1,
            defined: false,
        };
        out.merge(p2);
        out.merge(p3);
        out
    }

    /// `BoundingBox(const Points &points)` (`:226`), through the base's `BoundingBoxBase(const
    /// PointsType &points)` (`:34-36`) and its private `construct<IncludeBoundary = false>`
    /// (`:98-112`). `construct`'s `IncludeBoundary = true` arm has no caller this task's own
    /// units can reach (see the module doc's `get_extents` paragraph), so this is the one arm
    /// this task ports.
    pub fn from_points(points: &[Point]) -> Self {
        let mut it = points.iter();
        match it.next() {
            None => BoundingBox::new(),
            Some(&first) => {
                let mut min = first;
                let mut max = first;
                for &p in it {
                    min = Point::new(min.x().min(p.x()), min.y().min(p.y()));
                    max = Point::new(max.x().max(p.x()), max.y().max(p.y()));
                }
                let defined = min.x() < max.x() && min.y() < max.y();
                BoundingBox { min, max, defined }
            }
        }
    }

    /// `void reset()` (`:38`).
    pub fn reset(&mut self) {
        self.defined = false;
        self.min = Point::zero();
        self.max = Point::zero();
    }

    /// `void merge(const PointType &point)` (`BoundingBox.cpp:74-85`).
    pub fn merge(&mut self, point: Point) {
        if self.defined {
            self.min = Point::new(self.min.x().min(point.x()), self.min.y().min(point.y()));
            self.max = Point::new(self.max.x().max(point.x()), self.max.y().max(point.y()));
        } else {
            self.min = point;
            self.max = point;
            self.defined = true;
        }
    }

    /// `void merge(const PointsType &points)` (`:90-94`): `this->merge(BoundingBoxBase(points))`.
    pub fn merge_points(&mut self, points: &[Point]) {
        let bb = BoundingBox::from_points(points);
        self.merge_bb(&bb);
    }

    /// `void merge(const BoundingBoxBase &bb)` (`:98-112`).
    pub fn merge_bb(&mut self, bb: &BoundingBox) {
        debug_assert!(bb.defined || bb.min.x() >= bb.max.x() || bb.min.y() >= bb.max.y());
        if bb.defined {
            if self.defined {
                self.min = Point::new(self.min.x().min(bb.min.x()), self.min.y().min(bb.min.y()));
                self.max = Point::new(self.max.x().max(bb.max.x()), self.max.y().max(bb.max.y()));
            } else {
                self.min = bb.min;
                self.max = bb.max;
                self.defined = true;
            }
        }
    }

    /// `void scale(double factor)` (`:64-69`): `this->min *= factor; this->max *= factor;` on an
    /// `Eigen::Matrix<int64_t, 2, 1>`. Eigen's mixed-scalar `operator*=` multiplies each
    /// component by `factor` at `double` precision and converts the *product* back to `int64_t`
    /// (truncating toward zero, like a C-style cast) -- **not** the other way round (truncating
    /// `factor` to an integer first, which this module's first draft assumed and which the
    /// oracle disproved: `scale(0.5)` on `min = (-10, -20)` gives `(-5, -10)`, not `(0, 0)`).
    /// Confirmed against the oracle by this module's own `// PROOF: 2` test
    /// (`bounding_box_header_only_methods_match_the_oracle`, the `scale` assertion).
    pub fn scale(&mut self, factor: f64) {
        self.min = Point::new(
            (self.min.x() as f64 * factor) as i64,
            (self.min.y() as f64 * factor) as i64,
        );
        self.max = Point::new(
            (self.max.x() as f64 * factor) as i64,
            (self.max.y() as f64 * factor) as i64,
        );
    }

    /// `PointType size() const` (`BoundingBox.cpp:177-181`): `this->max - this->min`.
    pub fn size(&self) -> Point {
        self.max - self.min
    }

    /// `double radius() const` (`:194-198`): `0.5 * (this->max - this->min).cast<double>().norm()`.
    pub fn radius(&self) -> f64 {
        debug_assert!(self.defined);
        let d = self.max - self.min;
        let (dx, dy) = (d.x() as f64, d.y() as f64);
        // PORT SRC: src/libslic3r/BoundingBox.cpp:197
        0.5 * (dx * dx + dy * dy).sqrt()
    }

    /// `double area() const { return double(this->max(0) - this->min(0)) * (this->max(1) -
    /// this->min(1)); }` (`BoundingBox.hpp:45`): the first factor's subtraction happens in the
    /// native (`i64`) scalar type and is only then cast to `double`; the explicit `double()` on
    /// the left operand promotes the right operand to `double` too for the multiply (the usual
    /// C++ arithmetic conversions), so the second subtraction is read in its native type and
    /// then widened, never summed in `f64` from the start.
    pub fn area(&self) -> f64 {
        ((self.max.x() - self.min.x()) as f64) * ((self.max.y() - self.min.y()) as f64)
    }

    /// `void translate(coordf_t x, coordf_t y)` (`BoundingBox.hpp:46`): `PointType v(x, y);
    /// this->min += v; this->max += v;` -- for `Point`, the `Point(double, double)` constructor
    /// rounds (`Point.hpp:208`, ported as [`Point::from_f64_rounded`]), so this overload rounds
    /// its two arguments to the nearest integer coordinate before adding.
    pub fn translate(&mut self, x: f64, y: f64) {
        debug_assert!(self.defined);
        let v = Point::from_f64_rounded(x, y);
        self.min = self.min + v;
        self.max = self.max + v;
    }

    /// `void translate(const PointType &v)` (`:47`): exact integer add, no rounding and no
    /// `assert(defined)` (unlike the `(x, y)` overload above).
    pub fn translate_point(&mut self, v: Point) {
        self.min = self.min + v;
        self.max = self.max + v;
    }

    /// `void offset(coordf_t delta)` (`BoundingBox.cpp:208-214`): `PointType v(delta, delta);
    /// this->min -= v; this->max += v;` -- rounds `delta` the same way [`BoundingBox::translate`]
    /// does, through [`Point::from_f64_rounded`].
    pub fn offset(&mut self, delta: f64) {
        let v = Point::from_f64_rounded(delta, delta);
        self.min = self.min - v;
        self.max = self.max + v;
    }

    /// `BoundingBoxBase inflated(coordf_t delta) const throw()` (`BoundingBox.hpp:49`).
    pub fn inflated(&self, delta: f64) -> Self {
        let mut out = *self;
        out.offset(delta);
        out
    }

    /// `PointType center() const` (`BoundingBox.cpp:227-231`): `(this->min + this->max) / 2`,
    /// integer division (truncating toward zero, matching Rust's `i64` `/`).
    // PORT SRC: src/libslic3r/BoundingBox.cpp:230
    pub fn center(&self) -> Point {
        Point::new(
            (self.min.x() + self.max.x()) / 2,
            (self.min.y() + self.max.y()) / 2,
        )
    }

    /// `bool contains(const PointType &point) const` (`BoundingBox.hpp:51-54`).
    pub fn contains(&self, point: &Point) -> bool {
        point.x() >= self.min.x()
            && point.x() <= self.max.x()
            && point.y() >= self.min.y()
            && point.y() <= self.max.y()
    }

    /// `bool contains(const BoundingBoxBase &other) const` (`:55-57`).
    pub fn contains_bb(&self, other: &BoundingBox) -> bool {
        self.contains(&other.min) && self.contains(&other.max)
    }

    /// `bool overlap(const BoundingBoxBase &other) const` (`:58-61`).
    pub fn overlap(&self, other: &BoundingBox) -> bool {
        !(self.max.x() < other.min.x()
            || self.min.x() > other.max.x()
            || self.max.y() < other.min.y()
            || self.min.y() > other.max.y())
    }

    /// `PointType operator[](size_t idx) const` (`:62-81`): the box's four corners in the
    /// vendored order (min, (max.x, min.y), max, (min.x, max.y)); any other index is `Point()`
    /// (zero), matching the vendored `default:` arm.
    pub fn corner(&self, idx: usize) -> Point {
        match idx {
            0 => self.min,
            1 => Point::new(self.max.x(), self.min.y()),
            2 => self.max,
            // PORT SRC: src/libslic3r/BoundingBox.hpp:73
            3 => Point::new(self.min.x(), self.max.y()),
            _ => Point::zero(),
        }
    }

    // TODO(port): src/libslic3r/BoundingBox.hpp:214-215 -- `polygon(Polygon*)`/`polygon() const`
    // need `Polygon` (`slic3r-polygon`), a crate `slic3r-boundingbox` cannot depend on: the cut
    // graph runs the other way (`port/cut.tsv`'s `dep slic3r-polygon slic3r-boundingbox 1` row).

    /// `BoundingBox rotated(double angle) const` (`BoundingBox.cpp:37-45`): merges the four
    /// corners' own rotations about the origin; always `defined` on return.
    pub fn rotated(&self, angle: f64) -> BoundingBox {
        let mut out = BoundingBox::new();
        out.merge(self.min.rotated(angle));
        out.merge(self.max.rotated(angle));
        out.merge(Point::new(self.min.x(), self.max.y()).rotated(angle));
        out.merge(Point::new(self.max.x(), self.min.y()).rotated(angle));
        out
    }

    /// `BoundingBox rotated(double angle, const Point &center) const` (`:47-55`).
    pub fn rotated_about(&self, angle: f64, center: Point) -> BoundingBox {
        let mut out = BoundingBox::new();
        out.merge(self.min.rotated_about(angle, center));
        out.merge(self.max.rotated_about(angle, center));
        out.merge(Point::new(self.min.x(), self.max.y()).rotated_about(angle, center));
        out.merge(Point::new(self.max.x(), self.min.y()).rotated_about(angle, center));
        out
    }

    /// `void rotate(double angle) { (*this) = this->rotated(angle); }` (`BoundingBox.hpp:218`).
    pub fn rotate(&mut self, angle: f64) {
        *self = self.rotated(angle);
    }

    /// `void rotate(double angle, const Point &center)` (`:219`).
    pub fn rotate_about(&mut self, angle: f64, center: Point) {
        *self = self.rotated_about(angle, center);
    }

    /// `void align_to_grid(const coord_t cell_size)` (`BoundingBox.cpp:253-259`): only `min` is
    /// read or written; `max` is untouched.
    pub fn align_to_grid(&mut self, cell_size: i64) {
        if self.defined {
            self.min = Point::new(
                align_to_grid(self.min.x(), cell_size),
                align_to_grid(self.min.y(), cell_size),
            );
        }
    }

    /// `BoundingBox scaled(double factor) const` (`BoundingBox.cpp:57-62`): copies, then
    /// [`BoundingBox::scale`]s the copy.
    pub fn scaled(&self, factor: f64) -> BoundingBox {
        let mut out = *self;
        out.scale(factor);
        out
    }
}

// `bool operator==(const BoundingBoxBase &rhs)` (`BoundingBox.hpp:82`): compares `min`/`max`
// only -- **never** `defined` -- so two boxes that differ only in `defined` compare equal. This
// is the vendored operator's own, surprising shape, not an idiom choice, so it is a hand-written
// `impl` rather than `#[derive(PartialEq)]` (which would also compare `defined`); the derived
// `PartialEq`/`Eq` on the struct above exist for container use (`HashMap`/sorting of a whole
// `BoundingBox`, which this derive, not the vendored operator, backs) and are never used to
// implement this method itself.
impl BoundingBox {
    /// `bool operator==(const BoundingBoxBase &rhs)` / `bool operator!=(...)`
    /// (`BoundingBox.hpp:82-83`).
    pub fn vendored_eq(&self, rhs: &BoundingBox) -> bool {
        self.min == rhs.min && self.max == rhs.max
    }
}

impl std::fmt::Display for BoundingBox {
    /// `friend std::ostream &operator<<(std::ostream &os, const BoundingBoxBase &bbox)`
    /// (`BoundingBox.hpp:84-88`).
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{} x {}] from ({}, {})",
            self.max.x() - self.min.x(),
            self.max.y() - self.min.y(),
            self.min.x(),
            self.min.y(),
        )
    }
}

// TODO(port): src/libslic3r/BoundingBox.hpp:232 -- `friend BoundingBox get_extents_rotated(const
// Points &points, double angle);` is declared here only to grant `MultiPoint.cpp` access to
// `BoundingBox`'s private `construct`; the function itself is `MultiPoint.hpp`/`MultiPoint.cpp`'s
// own row, not this unit's, and is not ported here.

// -------------------------------------------------------------------------------------------
// BoundingBoxf (BoundingBoxBase<Vec2d>, BoundingBox.hpp:256-262)
// -------------------------------------------------------------------------------------------

/// `class BoundingBoxf : public BoundingBoxBase<Vec2d>` (`:256-262`). See [`BoundingBox`]'s doc
/// for why `min`/`max` are public fields.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct BoundingBoxf {
    pub min: Vec2d,
    pub max: Vec2d,
    pub defined: bool,
}

impl BoundingBoxf {
    /// See [`BoundingBox::new`]'s doc; `PointType::Zero()` for `Vec2d` is `Vec2::default()`
    /// (`#[derive(Default)]`, `0.0` per component).
    pub fn new() -> Self {
        BoundingBoxf {
            min: Vec2d::default(),
            max: Vec2d::default(),
            defined: false,
        }
    }

    /// See [`BoundingBox::from_min_max`]'s doc.
    pub fn from_min_max(pmin: Vec2d, pmax: Vec2d) -> Self {
        BoundingBoxf {
            min: pmin,
            max: pmax,
            defined: pmin.x() < pmax.x() && pmin.y() < pmax.y(),
        }
    }

    /// See [`BoundingBox::from_points`]'s doc; `BoundingBoxf(const std::vector<Vec2d> &points)`
    /// (`BoundingBox.hpp:261`).
    pub fn from_points(points: &[Vec2d]) -> Self {
        let mut it = points.iter();
        match it.next() {
            None => BoundingBoxf::new(),
            Some(&first) => {
                let mut min = first;
                let mut max = first;
                for &p in it {
                    min = Vec2::new(min.x().min(p.x()), min.y().min(p.y()));
                    max = Vec2::new(max.x().max(p.x()), max.y().max(p.y()));
                }
                let defined = min.x() < max.x() && min.y() < max.y();
                BoundingBoxf { min, max, defined }
            }
        }
    }

    /// See [`BoundingBox::reset`]'s doc.
    pub fn reset(&mut self) {
        self.defined = false;
        self.min = Vec2d::default();
        self.max = Vec2d::default();
    }

    /// See [`BoundingBox::merge`]'s doc.
    pub fn merge(&mut self, point: Vec2d) {
        if self.defined {
            self.min = Vec2::new(self.min.x().min(point.x()), self.min.y().min(point.y()));
            self.max = Vec2::new(self.max.x().max(point.x()), self.max.y().max(point.y()));
        } else {
            self.min = point;
            self.max = point;
            self.defined = true;
        }
    }

    /// See [`BoundingBox::merge_points`]'s doc.
    pub fn merge_points(&mut self, points: &[Vec2d]) {
        let bb = BoundingBoxf::from_points(points);
        self.merge_bb(&bb);
    }

    /// See [`BoundingBox::merge_bb`]'s doc.
    pub fn merge_bb(&mut self, bb: &BoundingBoxf) {
        debug_assert!(bb.defined || bb.min.x() >= bb.max.x() || bb.min.y() >= bb.max.y());
        if bb.defined {
            if self.defined {
                self.min = Vec2::new(self.min.x().min(bb.min.x()), self.min.y().min(bb.min.y()));
                self.max = Vec2::new(self.max.x().max(bb.max.x()), self.max.y().max(bb.max.y()));
            } else {
                self.min = bb.min;
                self.max = bb.max;
                self.defined = true;
            }
        }
    }

    /// `void scale(double factor)` (`BoundingBox.cpp:64-69`) at `PointType = Vec2d`: a plain
    /// `f64` multiply, no truncation (unlike [`BoundingBox::scale`]'s `Point` instantiation).
    pub fn scale(&mut self, factor: f64) {
        self.min = Vec2::new(self.min.x() * factor, self.min.y() * factor);
        self.max = Vec2::new(self.max.x() * factor, self.max.y() * factor);
    }

    /// See [`BoundingBox::size`]'s doc.
    pub fn size(&self) -> Vec2d {
        self.max - self.min
    }

    /// See [`BoundingBox::radius`]'s doc; same template body, shared with the `Point`
    /// instantiation (`BoundingBox.cpp:194-198`, explicitly instantiated again at `Vec2d` at
    /// `:200`).
    pub fn radius(&self) -> f64 {
        debug_assert!(self.defined);
        let d = self.max - self.min;
        // PORT SRC: src/libslic3r/BoundingBox.cpp:197
        0.5 * (d.x() * d.x() + d.y() * d.y()).sqrt()
    }

    /// See [`BoundingBox::area`]'s doc; `PointType::Scalar` is already `f64`, so there is no
    /// native-width subtlety here.
    pub fn area(&self) -> f64 {
        (self.max.x() - self.min.x()) * (self.max.y() - self.min.y())
    }

    /// See [`BoundingBox::translate`]'s doc; `Vec2d(double, double)` is a plain component
    /// assignment (no rounding), unlike `Point`'s rounding constructor.
    pub fn translate(&mut self, x: f64, y: f64) {
        debug_assert!(self.defined);
        let v = Vec2::new(x, y);
        self.min = self.min + v;
        self.max = self.max + v;
    }

    /// See [`BoundingBox::translate_point`]'s doc.
    pub fn translate_point(&mut self, v: Vec2d) {
        self.min = self.min + v;
        self.max = self.max + v;
    }

    /// See [`BoundingBox::offset`]'s doc; no rounding at this instantiation.
    pub fn offset(&mut self, delta: f64) {
        let v = Vec2::new(delta, delta);
        self.min = self.min - v;
        self.max = self.max + v;
    }

    /// See [`BoundingBox::inflated`]'s doc.
    pub fn inflated(&self, delta: f64) -> Self {
        let mut out = *self;
        out.offset(delta);
        out
    }

    /// See [`BoundingBox::center`]'s doc; plain `f64` division by `2`, not integer division.
    /// Spelled `2f64` (an integer literal with a float suffix, equal to `2.0`) rather than
    /// `2.0`, matching the vendored line's own bare `2` digit-for-digit -- `docs/PORTING.md`,
    /// "Decisions recorded on 2026-11-26", decision 8.
    pub fn center(&self) -> Vec2d {
        Vec2::new(
            (self.min.x() + self.max.x()) / 2f64,
            (self.min.y() + self.max.y()) / 2f64,
        )
    }

    /// See [`BoundingBox::contains`]'s doc.
    pub fn contains(&self, point: &Vec2d) -> bool {
        point.x() >= self.min.x()
            && point.x() <= self.max.x()
            && point.y() >= self.min.y()
            && point.y() <= self.max.y()
    }

    /// See [`BoundingBox::contains_bb`]'s doc.
    pub fn contains_bb(&self, other: &BoundingBoxf) -> bool {
        self.contains(&other.min) && self.contains(&other.max)
    }

    /// See [`BoundingBox::overlap`]'s doc.
    pub fn overlap(&self, other: &BoundingBoxf) -> bool {
        !(self.max.x() < other.min.x()
            || self.min.x() > other.max.x()
            || self.max.y() < other.min.y()
            || self.min.y() > other.max.y())
    }

    /// See [`BoundingBox::corner`]'s doc.
    pub fn corner(&self, idx: usize) -> Vec2d {
        match idx {
            0 => self.min,
            1 => Vec2::new(self.max.x(), self.min.y()),
            2 => self.max,
            // PORT SRC: src/libslic3r/BoundingBox.hpp:73
            3 => Vec2::new(self.min.x(), self.max.y()),
            _ => Vec2::default(),
        }
    }

    /// See [`BoundingBox::vendored_eq`]'s doc.
    pub fn vendored_eq(&self, rhs: &BoundingBoxf) -> bool {
        self.min == rhs.min && self.max == rhs.max
    }
}

impl std::fmt::Display for BoundingBoxf {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{} x {}] from ({}, {})",
            self.max.x() - self.min.x(),
            self.max.y() - self.min.y(),
            self.min.x(),
            self.min.y(),
        )
    }
}

// -------------------------------------------------------------------------------------------
// BoundingBox3 (BoundingBox3Base<Vec3crd>, BoundingBox.hpp:237-254)
// -------------------------------------------------------------------------------------------

/// `class BoundingBox3 : public BoundingBox3Base<Vec3crd>` (`:237-254`). `Vec3crd` is
/// `slic3r_point::point::Vec3<i64>` (`Point.hpp:53`); see [`BoundingBox`]'s doc for the public
/// fields.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BoundingBox3 {
    pub min: Vec3<i64>,
    pub max: Vec3<i64>,
    pub defined: bool,
}

impl BoundingBox3 {
    pub fn new() -> Self {
        BoundingBox3 {
            min: Vec3::default(),
            max: Vec3::default(),
            defined: false,
        }
    }

    /// `BoundingBox3(const Vec3crd &pmin, const Vec3crd &pmax)` (`:241`), through
    /// `BoundingBox3Base(const PointType &pmin, const PointType &pmax)` (`:122-124`): starts from
    /// the 2D base's `defined` rule (`min.x() < max.x() && min.y() < max.y()`), then additionally
    /// forces `defined = false` when `pmin.z() >= pmax.z()`.
    pub fn from_min_max(pmin: Vec3<i64>, pmax: Vec3<i64>) -> Self {
        let mut defined = pmin.x() < pmax.x() && pmin.y() < pmax.y();
        if pmin.z() >= pmax.z() {
            defined = false;
        }
        BoundingBox3 {
            min: pmin,
            max: pmax,
            defined,
        }
    }

    /// `BoundingBox3(const Points3& points)` (`:242-252`): unlike the 2D `BoundingBox(const
    /// Points&)` constructor, this one does not go through `construct`/`merge` at all -- it
    /// reads `points.front()` directly for both `min` and `max`, then folds every point
    /// (including the first again) through component-wise min/max, and sets `defined = true`
    /// unconditionally whenever `points` is non-empty (never checking strict ordering the way
    /// the 2D constructor or `BoundingBox3Base`'s own iterator constructor below does). An empty
    /// `points` leaves the default-constructed state (`defined = false`).
    pub fn from_points3(points: &[Vec3<i64>]) -> Self {
        let mut out = BoundingBox3::new();
        if let Some(&first) = points.first() {
            out.min = first;
            out.max = first;
            for &p in points {
                out.min = Vec3::new(
                    out.min.x().min(p.x()),
                    out.min.y().min(p.y()),
                    out.min.z().min(p.z()),
                );
                out.max = Vec3::new(
                    out.max.x().max(p.x()),
                    out.max.y().max(p.y()),
                    out.max.z().max(p.z()),
                );
            }
            out.defined = true;
        }
        out
    }

    /// `BoundingBox3Base(It from, It to)` (`BoundingBox.hpp:128-142`), the generic iterator
    /// constructor `BoundingBox3`'s own `std::vector<Vec3crd>` constructor forwards to
    /// (`:253`, `BoundingBox3(const std::vector<Vec3crd>& points)`): throws
    /// `InvalidArgument` on an empty input (`:131`), unlike every other constructor in this
    /// module, and sets `defined` from the *strict* 3D ordering (`min < max` on all three axes),
    /// unlike [`BoundingBox3::from_points3`] above.
    pub fn from_vec3_slice(points: &[Vec3<i64>]) -> Result<Self, Exception> {
        let mut it = points.iter();
        let &first = it.next().ok_or_else(|| {
            Exception::InvalidArgument(
                "Empty point set supplied to BoundingBox3Base constructor.".to_string(),
            )
        })?;
        let mut min = first;
        let mut max = first;
        for &p in it {
            min = Vec3::new(min.x().min(p.x()), min.y().min(p.y()), min.z().min(p.z()));
            max = Vec3::new(max.x().max(p.x()), max.y().max(p.y()), max.z().max(p.z()));
        }
        let defined = min.x() < max.x() && min.y() < max.y() && min.z() < max.z();
        Ok(BoundingBox3 { min, max, defined })
    }

    /// `void merge(const PointType &point)` (`BoundingBox.cpp:138-149`).
    pub fn merge(&mut self, point: Vec3<i64>) {
        if self.defined {
            self.min = Vec3::new(
                self.min.x().min(point.x()),
                self.min.y().min(point.y()),
                self.min.z().min(point.z()),
            );
            self.max = Vec3::new(
                self.max.x().max(point.x()),
                self.max.y().max(point.y()),
                self.max.z().max(point.z()),
            );
        } else {
            self.min = point;
            self.max = point;
            self.defined = true;
        }
    }

    /// `void merge(const BoundingBox3Base &bb)` (`:160-174`).
    pub fn merge_bb(&mut self, bb: &BoundingBox3) {
        debug_assert!(
            bb.defined
                || bb.min.x() >= bb.max.x()
                || bb.min.y() >= bb.max.y()
                || bb.min.z() >= bb.max.z()
        );
        if bb.defined {
            if self.defined {
                self.min = Vec3::new(
                    self.min.x().min(bb.min.x()),
                    self.min.y().min(bb.min.y()),
                    self.min.z().min(bb.min.z()),
                );
                self.max = Vec3::new(
                    self.max.x().max(bb.max.x()),
                    self.max.y().max(bb.max.y()),
                    self.max.z().max(bb.max.z()),
                );
            } else {
                self.min = bb.min;
                self.max = bb.max;
                self.defined = true;
            }
        }
    }

    /// `PointType size() const` (`:186-190`).
    pub fn size(&self) -> Vec3<i64> {
        self.max - self.min
    }

    /// `double radius() const` (`:202-206`).
    pub fn radius(&self) -> f64 {
        let d = self.max - self.min;
        let (dx, dy, dz) = (d.x() as f64, d.y() as f64, d.z() as f64);
        // PORT SRC: src/libslic3r/BoundingBox.cpp:204
        0.5 * (dx * dx + dy * dy + dz * dz).sqrt()
    }

    /// `void translate(coordf_t x, coordf_t y, coordf_t z)` (`BoundingBox.hpp:154`): asserts
    /// `defined`; `PointType v(x, y, z)` constructs a bare `Vec3crd`
    /// (`Eigen::Matrix<coord_t, 3, 1>`), **not** `Point3`, which has no dedicated rounding
    /// constructor of its own at the pin -- unlike `Point(double, double)`
    /// (`Point.hpp:208`, ported as [`slic3r_point::point::Point::from_f64_rounded`]), a bare
    /// Eigen vector's own per-component `double` -> `coord_t` conversion truncates toward zero,
    /// the same rule `docs/PORTING.md`'s "Scaling conversions" states for `.cast<Tout>()`
    /// (`Point.hpp:689`) -- there being no evidence anywhere at the pin of a bare Eigen integer
    /// vector rounding a floating constructor argument instead of truncating it.
    pub fn translate(&mut self, x: f64, y: f64, z: f64) {
        debug_assert!(self.defined);
        let v = Vec3::new(x as i64, y as i64, z as i64);
        self.min = self.min + v;
        self.max = self.max + v;
    }

    /// `void offset(coordf_t delta)` (`:219-224`); see [`BoundingBox3::translate`]'s doc for why
    /// this truncates rather than rounds.
    pub fn offset(&mut self, delta: f64) {
        let d = delta as i64;
        let v = Vec3::new(d, d, d);
        self.min = self.min - v;
        self.max = self.max + v;
    }

    /// `PointType center() const` (`:236-240`): integer division, truncating toward zero.
    // PORT SRC: src/libslic3r/BoundingBox.cpp:239
    pub fn center(&self) -> Vec3<i64> {
        Vec3::new(
            (self.min.x() + self.max.x()) / 2,
            (self.min.y() + self.max.y()) / 2,
            (self.min.z() + self.max.z()) / 2,
        )
    }

    /// `coordf_t max_size() const { PointType s = size(); return std::max(s.x(), std::max(s.y(),
    /// s.z())); }` (`:244-249`).
    pub fn max_size(&self) -> i64 {
        let s = self.size();
        s.x().max(s.y()).max(s.z())
    }

    /// `bool contains(const PointType &point) const` (`:161-163`): the 2D `contains` plus a `z`
    /// range check.
    pub fn contains(&self, point: &Vec3<i64>) -> bool {
        point.x() >= self.min.x()
            && point.x() <= self.max.x()
            && point.y() >= self.min.y()
            && point.y() <= self.max.y()
            && point.z() >= self.min.z()
            && point.z() <= self.max.z()
    }

    /// `bool contains(const BoundingBox3Base& other) const` (`:165-167`).
    pub fn contains_bb(&self, other: &BoundingBox3) -> bool {
        self.contains(&other.min) && self.contains(&other.max)
    }

    /// `bool intersects(const BoundingBox3Base& other) const` (`:170-173`): intersects *without*
    /// boundaries (strict `<`/`>`, unlike [`BoundingBox::overlap`]'s inclusive `<=`/`>=` logic).
    pub fn intersects(&self, other: &BoundingBox3) -> bool {
        self.min.x() < other.max.x()
            && self.max.x() > other.min.x()
            && self.min.y() < other.max.y()
            && self.max.y() > other.min.y()
            && self.min.z() < other.max.z()
            && self.max.z() > other.min.z()
    }

    // TODO(port): src/libslic3r/BoundingBox.hpp:148 -- `Polygon polygon(bool is_scaled) const`
    // needs `Polygon` (`slic3r-polygon`); see this module's own doc for the dependency-direction
    // reason.
}

// -------------------------------------------------------------------------------------------
// BoundingBoxf3 (BoundingBox3Base<Vec3d>, BoundingBox.hpp:264-270)
// -------------------------------------------------------------------------------------------

/// `class BoundingBoxf3 : public BoundingBox3Base<Vec3d> { using BoundingBox3Base::BoundingBox3Base; ... }`
/// (`:264-270`): inherits every `BoundingBox3Base<Vec3d>` constructor unchanged, so this struct's
/// own constructors mirror [`BoundingBox3`]'s `Vec3crd` ones at `Vec3d` instead.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct BoundingBoxf3 {
    pub min: slic3r_point::point::Vec3d,
    pub max: slic3r_point::point::Vec3d,
    pub defined: bool,
}

impl BoundingBoxf3 {
    pub fn new() -> Self {
        BoundingBoxf3 {
            min: Vec3::default(),
            max: Vec3::default(),
            defined: false,
        }
    }

    /// See [`BoundingBox3::from_min_max`]'s doc.
    pub fn from_min_max(
        pmin: slic3r_point::point::Vec3d,
        pmax: slic3r_point::point::Vec3d,
    ) -> Self {
        let mut defined = pmin.x() < pmax.x() && pmin.y() < pmax.y();
        if pmin.z() >= pmax.z() {
            defined = false;
        }
        BoundingBoxf3 {
            min: pmin,
            max: pmax,
            defined,
        }
    }

    /// See [`BoundingBox3::from_vec3_slice`]'s doc.
    pub fn from_vec3_slice(points: &[slic3r_point::point::Vec3d]) -> Result<Self, Exception> {
        let mut it = points.iter();
        let &first = it.next().ok_or_else(|| {
            Exception::InvalidArgument(
                "Empty point set supplied to BoundingBox3Base constructor.".to_string(),
            )
        })?;
        let mut min = first;
        let mut max = first;
        for &p in it {
            min = Vec3::new(min.x().min(p.x()), min.y().min(p.y()), min.z().min(p.z()));
            max = Vec3::new(max.x().max(p.x()), max.y().max(p.y()), max.z().max(p.z()));
        }
        let defined = min.x() < max.x() && min.y() < max.y() && min.z() < max.z();
        Ok(BoundingBoxf3 { min, max, defined })
    }

    /// See [`BoundingBox3::merge`]'s doc.
    pub fn merge(&mut self, point: slic3r_point::point::Vec3d) {
        if self.defined {
            self.min = Vec3::new(
                self.min.x().min(point.x()),
                self.min.y().min(point.y()),
                self.min.z().min(point.z()),
            );
            self.max = Vec3::new(
                self.max.x().max(point.x()),
                self.max.y().max(point.y()),
                self.max.z().max(point.z()),
            );
        } else {
            self.min = point;
            self.max = point;
            self.defined = true;
        }
    }

    /// `void merge(const PointsType &points) { this->merge(BoundingBox3Base(points)); }`
    /// (`BoundingBox.cpp:153-157`), explicitly instantiated only at `Vec3d`
    /// (`:158`, `template void BoundingBox3Base<Vec3d>::merge(const Pointf3s &points);`) --
    /// `BoundingBox3` (`Vec3crd`) has no in-scope instantiation of this overload, so only this
    /// `f64` instantiation is ported, per the idiom map's "monomorphised the same way ... used
    /// in in-scope code" rule. `BoundingBox3Base(const PointsType &points) :
    /// BoundingBox3Base(points.begin(), points.end()) {}` (`BoundingBox.hpp:144-146`) forwards
    /// to the throwing iterator constructor unconditionally, so this returns `Err` on an empty
    /// `points` the same way that constructor throws, per `docs/PORTING.md`'s "Error sets" rule
    /// (a `throw` site and its catch keep the same set of cases across the port) -- never a
    /// silently-empty merge.
    pub fn merge_points(&mut self, points: &[slic3r_point::point::Vec3d]) -> Result<(), Exception> {
        let bb = BoundingBoxf3::from_vec3_slice(points)?;
        self.merge_bb(&bb);
        Ok(())
    }

    /// See [`BoundingBox3::merge_bb`]'s doc.
    pub fn merge_bb(&mut self, bb: &BoundingBoxf3) {
        debug_assert!(
            bb.defined
                || bb.min.x() >= bb.max.x()
                || bb.min.y() >= bb.max.y()
                || bb.min.z() >= bb.max.z()
        );
        if bb.defined {
            if self.defined {
                self.min = Vec3::new(
                    self.min.x().min(bb.min.x()),
                    self.min.y().min(bb.min.y()),
                    self.min.z().min(bb.min.z()),
                );
                self.max = Vec3::new(
                    self.max.x().max(bb.max.x()),
                    self.max.y().max(bb.max.y()),
                    self.max.z().max(bb.max.z()),
                );
            } else {
                self.min = bb.min;
                self.max = bb.max;
                self.defined = true;
            }
        }
    }

    /// See [`BoundingBox3::size`]'s doc.
    pub fn size(&self) -> slic3r_point::point::Vec3d {
        self.max - self.min
    }

    /// See [`BoundingBox3::radius`]'s doc.
    pub fn radius(&self) -> f64 {
        let d = self.max - self.min;
        // PORT SRC: src/libslic3r/BoundingBox.cpp:204
        0.5 * (d.x() * d.x() + d.y() * d.y() + d.z() * d.z()).sqrt()
    }

    /// `void translate(const Vec3d &v)` (`BoundingBox.hpp:155`): exact add, `Vec3d` already
    /// holds `f64`, so no rounding happens at this instantiation.
    pub fn translate_vec(&mut self, v: slic3r_point::point::Vec3d) {
        self.min = self.min + v;
        self.max = self.max + v;
    }

    /// See [`BoundingBox3::offset`]'s doc; no rounding at this instantiation.
    pub fn offset(&mut self, delta: f64) {
        let v = Vec3::new(delta, delta, delta);
        self.min = self.min - v;
        self.max = self.max + v;
    }

    /// See [`BoundingBox3::center`]'s doc; plain `f64` division, spelled `2f64` for the same
    /// reason [`BoundingBoxf::center`]'s doc gives.
    pub fn center(&self) -> slic3r_point::point::Vec3d {
        Vec3::new(
            (self.min.x() + self.max.x()) / 2f64,
            (self.min.y() + self.max.y()) / 2f64,
            (self.min.z() + self.max.z()) / 2f64,
        )
    }

    /// See [`BoundingBox3::max_size`]'s doc.
    pub fn max_size(&self) -> f64 {
        let s = self.size();
        s.x().max(s.y()).max(s.z())
    }

    // TODO(port): src/libslic3r/BoundingBox.cpp:261-290 -- `BoundingBoxf3::transformed(const
    // Transform3d&)` needs the deferred `Transform3d` type
    // (`crates/slic3r-point/src/point.rs:197`'s own `// TODO(port):`); no in-scope caller this
    // task's own units reach forces it yet.
}

// -------------------------------------------------------------------------------------------
// Free functions (BoundingBox.hpp:272-310)
// -------------------------------------------------------------------------------------------

/// `template<typename PointType, typename PointsType> inline bool empty(const BoundingBoxBase
/// &bb)` (`:272-276`), at `PointType = Point`.
pub fn empty(bb: &BoundingBox) -> bool {
    !bb.defined || bb.min.x() >= bb.max.x() || bb.min.y() >= bb.max.y()
}

/// See [`empty`]'s doc; at `PointType = Vec2d`.
pub fn empty_f(bb: &BoundingBoxf) -> bool {
    !bb.defined || bb.min.x() >= bb.max.x() || bb.min.y() >= bb.max.y()
}

/// `template<typename PointType> inline bool empty(const BoundingBox3Base &bb)` (`:278-282`), at
/// `PointType = Vec3crd`.
pub fn empty3(bb: &BoundingBox3) -> bool {
    !bb.defined || bb.min.x() >= bb.max.x() || bb.min.y() >= bb.max.y() || bb.min.z() >= bb.max.z()
}

/// See [`empty3`]'s doc; at `PointType = Vec3d`.
pub fn empty3_f(bb: &BoundingBoxf3) -> bool {
    !bb.defined || bb.min.x() >= bb.max.x() || bb.min.y() >= bb.max.y() || bb.min.z() >= bb.max.z()
}

/// `inline BoundingBox scaled(const BoundingBoxf &bb) { return {scaled(bb.min), scaled(bb.max)};
/// }` (`:284`): `Slic3r::scaled(const Vec2d&)` is `scaled_eigen2` (already ported,
/// `crates/slic3r-point/src/point.rs`). The braced return **is** a call to `BoundingBox`'s own
/// two-point constructor (`BoundingBox` is not an aggregate -- its base declares constructors
/// -- so C++ list-initialization resolves `{a, b}` against `BoundingBox(const Point&, const
/// Point&)` rather than initialising fields positionally), which re-derives `defined` from the
/// *scaled* min/max's own strict ordering; it does **not** carry `bb.defined` through
/// unchanged, a divergence this module's own first draft got wrong and corrected before
/// the pull request's own review found it (its own `vendored_eq`/`corner` doc comments
/// elsewhere in this module are about the opposite case -- a field read directly, never
/// through a constructor -- which is why this one call site needs its own note).
pub fn scaled(bb: &BoundingBoxf) -> BoundingBox {
    let min = slic3r_point::point::scaled_eigen2(bb.min);
    let max = slic3r_point::point::scaled_eigen2(bb.max);
    BoundingBox::from_min_max(Point::new(min.x(), min.y()), Point::new(max.x(), max.y()))
}

/// `template<class T = double> BoundingBoxBase<Vec<2, T>> unscaled(const BoundingBox &bb)`
/// (`:292-293`), at its default `T = double`: `Slic3r::unscaled<double>(const Vec2crd&)` is
/// [`slic3r_point::point::unscaled_eigen2`]. See [`scaled`]'s doc for why this also re-derives
/// `defined` through the two-point constructor rather than carrying `bb.defined` through.
pub fn unscaled(bb: &BoundingBox) -> BoundingBoxf {
    let min = slic3r_point::point::unscaled_eigen2(Vec2::new(bb.min.x(), bb.min.y()));
    let max = slic3r_point::point::unscaled_eigen2(Vec2::new(bb.max.x(), bb.max.y()));
    BoundingBoxf::from_min_max(min, max)
}

// TODO(port): src/libslic3r/BoundingBox.hpp:286-287 -- `template<class T = coord_t>
// BoundingBoxBase<Vec<2, T>> scaled(const BoundingBoxf &bb)`, the templated overload at a `T`
// other than its default; no in-scope caller instantiates it at any `T` but the default the
// non-template `scaled` above already covers.
// TODO(port): src/libslic3r/BoundingBox.hpp:289-290 -- the `BoundingBox3Base<Vec<3, T>>
// scaled<T>(const BoundingBoxf3&)` 3D overload; no in-scope caller.
// TODO(port): src/libslic3r/BoundingBox.hpp:295-296 -- the `BoundingBox3Base<Vec<3, T>>
// unscaled<T>(const BoundingBox3&)` 3D overload; no in-scope caller.
// TODO(port): src/libslic3r/BoundingBox.hpp:298-310 -- the generic `cast<Tout, Tin>` 2D/3D
// free functions; no in-scope caller instantiates either at a concrete `Tout`/`Tin` pair.
// TODO(port): src/libslic3r/BoundingBox.hpp:315-320 -- the four Cereal `serialize`
// specialisations (`BoundingBox`/`BoundingBox3`/`BoundingBoxf`/`BoundingBoxf3`): project-file
// (de)serialisation, under the preset/project-format layer `docs/PORTING.md`'s don't-translate
// list and the manifest's deductions put out of scope (R11).

#[cfg(test)]
mod tests {
    use super::*;
    use slic3r_proof::compare_unit;
    use slic3r_proof::proof_units::bounding_box;
    use slic3r_proof::Oracle;

    /// Proof path 1 (R19), first marker for the `BoundingBox.cpp` row: drives
    /// `BoundingBox.cpp`'s own, non-template methods (externally linked in the archive `just
    /// oracle-build` produces) against this module's port.
    // PROOF: 1
    #[test]
    fn bounding_box_cpp_methods_match_the_oracle() {
        let min = Point::new(-10, -20);
        let max = Point::new(30, 40);

        // `BoundingBox::rotated(angle)`.
        for &angle in &[0.0_f64, 0.3, std::f64::consts::FRAC_PI_4, -1.7, 6.0] {
            let bb = BoundingBox::from_min_max(min, max);
            let actual = bb.rotated(angle);
            let oracle_out = bounding_box::path1_rotated(min.x(), min.y(), max.x(), max.y(), angle);
            compare_unit(
                "BoundingBox.cpp",
                Oracle::Built,
                vec![
                    actual.min.x(),
                    actual.min.y(),
                    actual.max.x(),
                    actual.max.y(),
                ],
                oracle_out,
            )
            .expect("BoundingBox::rotated(angle) must match the oracle exactly");
        }

        // `BoundingBox::rotated(angle, center)`.
        let center = Point::new(5, 5);
        for &angle in &[0.0_f64, 0.3, 2.1, -0.9] {
            let bb = BoundingBox::from_min_max(min, max);
            let actual = bb.rotated_about(angle, center);
            let oracle_out = bounding_box::path1_rotated_about(
                min.x(),
                min.y(),
                max.x(),
                max.y(),
                center.x(),
                center.y(),
                angle,
            );
            compare_unit(
                "BoundingBox.cpp",
                Oracle::Built,
                vec![
                    actual.min.x(),
                    actual.min.y(),
                    actual.max.x(),
                    actual.max.y(),
                ],
                oracle_out,
            )
            .expect("BoundingBox::rotated(angle, center) must match the oracle exactly");
        }

        // `BoundingBox::scaled(factor)`.
        for &factor in &[1.0_f64, 2.0, 0.5, 3.75, -1.0] {
            let bb = BoundingBox::from_min_max(min, max);
            let actual = bb.scaled(factor);
            let oracle_out =
                bounding_box::path1_scaled(min.x(), min.y(), max.x(), max.y(), bb.defined, factor);
            compare_unit(
                "BoundingBox.cpp",
                Oracle::Built,
                vec![
                    actual.min.x(),
                    actual.min.y(),
                    actual.max.x(),
                    actual.max.y(),
                    actual.defined as i64,
                ],
                oracle_out,
            )
            .expect("BoundingBox::scaled(factor) must match the oracle exactly");
        }

        // `BoundingBox::align_to_grid(cell_size)`.
        for &(mx, my, cell) in &[
            (7_i64, -13_i64, 5_i64),
            (-7, 13, 5),
            (0, 0, 10),
            (23, -4, 7),
        ] {
            let mut actual =
                BoundingBox::from_min_max(Point::new(mx, my), Point::new(mx + 1, my + 1));
            actual.align_to_grid(cell);
            let oracle_out = bounding_box::path1_align_to_grid(mx, my, actual.defined, cell);
            compare_unit(
                "BoundingBox.cpp",
                Oracle::Built,
                vec![actual.min.x(), actual.min.y()],
                oracle_out,
            )
            .expect("BoundingBox::align_to_grid(cell_size) must match the oracle exactly");
        }
    }

    /// Proof path 2 (R19), second marker for the `BoundingBox.hpp` row: drives
    /// `BoundingBox.hpp`'s own header-only `BoundingBoxBase<Point, Points>` surface, instantiated
    /// directly in `cxx/proof_units/bounding_box.cpp`, against this module's port.
    // PROOF: 2
    #[test]
    fn bounding_box_header_only_methods_match_the_oracle() {
        let min = Point::new(-10, -20);
        let max = Point::new(30, 40);
        let bb = BoundingBox::from_min_max(min, max);

        // merge(point)
        for &(px, py) in &[(100_i64, 100_i64), (-100, -100), (5, 5)] {
            let mut actual = bb;
            actual.merge(Point::new(px, py));
            let oracle_out = bounding_box::path2_merge_point(
                min.x(),
                min.y(),
                max.x(),
                max.y(),
                bb.defined,
                px,
                py,
            );
            compare_unit(
                "BoundingBox.hpp",
                Oracle::Built,
                vec![
                    actual.min.x(),
                    actual.min.y(),
                    actual.max.x(),
                    actual.max.y(),
                    actual.defined as i64,
                ],
                oracle_out,
            )
            .expect("BoundingBoxBase::merge(point) must match the oracle exactly");
        }

        // merge(points) / BoundingBox(points) construction.
        let xs = [3_i64, -7, 42, 0];
        let ys = [9_i64, 11, -5, 0];
        let points: Vec<Point> = xs
            .iter()
            .zip(ys.iter())
            .map(|(&x, &y)| Point::new(x, y))
            .collect();
        let actual = BoundingBox::from_points(&points);
        let oracle_out = bounding_box::path2_merge_points(&xs, &ys);
        compare_unit(
            "BoundingBox.hpp",
            Oracle::Built,
            vec![
                actual.min.x(),
                actual.min.y(),
                actual.max.x(),
                actual.max.y(),
                actual.defined as i64,
            ],
            oracle_out,
        )
        .expect("BoundingBox(const Points&) must match the oracle exactly");

        // merge(bb)
        let other = BoundingBox::from_min_max(Point::new(-50, -50), Point::new(-1, -1));
        let mut actual = bb;
        actual.merge_bb(&other);
        let oracle_out = bounding_box::path2_merge_bb(
            min.x(),
            min.y(),
            max.x(),
            max.y(),
            bb.defined,
            other.min.x(),
            other.min.y(),
            other.max.x(),
            other.max.y(),
            other.defined,
        );
        compare_unit(
            "BoundingBox.hpp",
            Oracle::Built,
            vec![
                actual.min.x(),
                actual.min.y(),
                actual.max.x(),
                actual.max.y(),
                actual.defined as i64,
            ],
            oracle_out,
        )
        .expect("BoundingBoxBase::merge(bb) must match the oracle exactly");

        // scale(factor), driven directly (not through `scaled`) to pin down the `f64 -> i64`
        // truncation-before-multiply semantics this module's own doc discusses.
        for &factor in &[1.0_f64, 2.0, 0.5, 3.75, -1.25, 100.9] {
            let mut actual = bb;
            actual.scale(factor);
            let oracle_out = bounding_box::path2_scale(min.x(), min.y(), max.x(), max.y(), factor);
            compare_unit(
                "BoundingBox.hpp",
                Oracle::Built,
                vec![
                    actual.min.x(),
                    actual.min.y(),
                    actual.max.x(),
                    actual.max.y(),
                ],
                oracle_out,
            )
            .expect("BoundingBoxBase::scale(factor) must match the oracle exactly");
        }

        // size(), radius(), area(), center()
        let oracle_size = bounding_box::path2_size(min.x(), min.y(), max.x(), max.y());
        let actual_size = bb.size();
        assert_eq!(
            (actual_size.x(), actual_size.y()),
            (oracle_size[0], oracle_size[1])
        );
        let oracle_radius = bounding_box::path2_radius(min.x(), min.y(), max.x(), max.y());
        assert_eq!(bb.radius(), oracle_radius);
        let oracle_area = bounding_box::path2_area(min.x(), min.y(), max.x(), max.y());
        assert_eq!(bb.area(), oracle_area);
        let oracle_center = bounding_box::path2_center(min.x(), min.y(), max.x(), max.y());
        let actual_center = bb.center();
        assert_eq!(
            (actual_center.x(), actual_center.y()),
            (oracle_center[0], oracle_center[1])
        );

        // translate(x, y) [rounding] and translate(Point) [exact].
        for &(tx, ty) in &[(1.4_f64, -2.6), (0.5, 0.5), (-0.5, -0.5), (100.9, -100.1)] {
            let mut actual = bb;
            actual.translate(tx, ty);
            let oracle_out =
                bounding_box::path2_translate_xy(min.x(), min.y(), max.x(), max.y(), tx, ty);
            compare_unit(
                "BoundingBox.hpp",
                Oracle::Built,
                vec![
                    actual.min.x(),
                    actual.min.y(),
                    actual.max.x(),
                    actual.max.y(),
                ],
                oracle_out,
            )
            .expect("BoundingBoxBase::translate(x, y) must match the oracle exactly");
        }
        let mut actual = bb;
        actual.translate_point(Point::new(7, -3));
        let oracle_out =
            bounding_box::path2_translate_point(min.x(), min.y(), max.x(), max.y(), 7, -3);
        compare_unit(
            "BoundingBox.hpp",
            Oracle::Built,
            vec![
                actual.min.x(),
                actual.min.y(),
                actual.max.x(),
                actual.max.y(),
            ],
            oracle_out,
        )
        .expect("BoundingBoxBase::translate(const PointType&) must match the oracle exactly");

        // offset(delta)
        for &delta in &[1.4_f64, -2.6, 0.5, -0.5] {
            let mut actual = bb;
            actual.offset(delta);
            let oracle_out = bounding_box::path2_offset(min.x(), min.y(), max.x(), max.y(), delta);
            compare_unit(
                "BoundingBox.hpp",
                Oracle::Built,
                vec![
                    actual.min.x(),
                    actual.min.y(),
                    actual.max.x(),
                    actual.max.y(),
                ],
                oracle_out,
            )
            .expect("BoundingBoxBase::offset(delta) must match the oracle exactly");
        }

        // contains(point), contains(bb), overlap(bb)
        for &(px, py) in &[(0_i64, 0), (-10, -20), (30, 40), (31, 40), (-11, -20)] {
            let actual = bb.contains(&Point::new(px, py));
            let oracle_out =
                bounding_box::path2_contains_point(min.x(), min.y(), max.x(), max.y(), px, py);
            assert_eq!(actual, oracle_out, "contains(point) at ({px}, {py})");
        }
        let inside = BoundingBox::from_min_max(Point::new(-5, -5), Point::new(5, 5));
        let outside = BoundingBox::from_min_max(Point::new(100, 100), Point::new(200, 200));
        let overlapping = BoundingBox::from_min_max(Point::new(20, 20), Point::new(50, 50));
        for other in [inside, outside, overlapping] {
            let actual_contains = bb.contains_bb(&other);
            let oracle_contains = bounding_box::path2_contains_bbox(
                min.x(),
                min.y(),
                max.x(),
                max.y(),
                other.min.x(),
                other.min.y(),
                other.max.x(),
                other.max.y(),
            );
            assert_eq!(
                actual_contains, oracle_contains,
                "contains(bb) against {other:?}"
            );

            let actual_overlap = bb.overlap(&other);
            let oracle_overlap = bounding_box::path2_overlap(
                min.x(),
                min.y(),
                max.x(),
                max.y(),
                other.min.x(),
                other.min.y(),
                other.max.x(),
                other.max.y(),
            );
            assert_eq!(
                actual_overlap, oracle_overlap,
                "overlap(bb) against {other:?}"
            );
        }

        // operator[] (corner)
        for idx in 0..5_i32 {
            let actual = bb.corner(idx as usize);
            let oracle_out = bounding_box::path2_index(min.x(), min.y(), max.x(), max.y(), idx);
            assert_eq!(
                (actual.x(), actual.y()),
                (oracle_out[0], oracle_out[1]),
                "corner({idx})"
            );
        }

        // operator== (ignores `defined`)
        let same_min_max_different_defined = BoundingBox {
            min: bb.min,
            max: bb.max,
            defined: !bb.defined,
        };
        let actual_eq = bb.vendored_eq(&same_min_max_different_defined);
        let oracle_eq = bounding_box::path2_eq(
            min.x(),
            min.y(),
            max.x(),
            max.y(),
            min.x(),
            min.y(),
            max.x(),
            max.y(),
        );
        assert_eq!(actual_eq, oracle_eq);
        assert!(actual_eq, "operator== must ignore `defined`");
    }

    #[test]
    fn empty_matches_vendored_definition() {
        assert!(empty(&BoundingBox::new()));
        let bb = BoundingBox::from_min_max(Point::new(0, 0), Point::new(10, 10));
        assert!(!empty(&bb));
        let degenerate = BoundingBox::from_min_max(Point::new(0, 0), Point::new(0, 10));
        assert!(empty(&degenerate));
    }

    #[test]
    fn bounding_box3_from_vec3_slice_rejects_empty_input() {
        let err = BoundingBox3::from_vec3_slice(&[]).unwrap_err();
        assert!(matches!(err, Exception::InvalidArgument(_)));
    }

    #[test]
    fn bounding_boxf_sanity() {
        // No dedicated oracle proof for the `Vec2d` instantiation (see the module doc); this is
        // a plain self-consistency check against the formulas `BoundingBox`'s own proof already
        // pins down at `Point`.
        let mut bb = BoundingBoxf::from_min_max(Vec2::new(-1.0, -2.0), Vec2::new(3.0, 4.0));
        bb.merge(Vec2::new(10.0, -10.0));
        assert_eq!(bb.min, Vec2::new(-1.0, -10.0));
        assert_eq!(bb.max, Vec2::new(10.0, 4.0));
        let center = bb.center();
        assert_eq!(center, Vec2::new(4.5, -3.0));
    }
}

// PORT STATUS
// source: src/libslic3r/BoundingBox.cpp
// source: src/libslic3r/BoundingBox.hpp
// confidence: medium
// tier: exact
// oracle: built
