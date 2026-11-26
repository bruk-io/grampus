//! Port of `src/libslic3r/Line.hpp`/`Line.cpp` (task `61-foundation-line-slicing`, plan step 20,
//! `port/foundation.tsv` row `foundation-10`): the port's line segment types -- `Line` (2D,
//! integer `coord_t`), `Line3` (3D, integer), `Linef`/`Linef3` (2D/3D, `double`) -- and the
//! generic `line_alg` distance/intersection algorithms `Line.hpp:26-153` declares as templates
//! over any line-shaped type.
//!
//! **The `line_alg` templates are monomorphized here, not translated generically.** The vendored
//! templates (`line_alg::distance_to_squared`, `distance_to_infinite_squared`, `intersection`)
//! are instantiated by their callers at a fixed `Dim`/`Scalar` pair; this port writes out each
//! instantiation an in-scope caller actually reaches, the same choice [`crates::slic3r-point`]'s
//! own module doc makes for the fully generic `Vec<N, T>` (collapsed to [`Vec2`]-shaped structs
//! per concrete `T`) and the one `cross2` instantiation `Point::ccw` needs. Three instantiations
//! are reached from this unit's own two files:
//!
//! - `Dim = 2, Scalar = i64` (`Line`'s own `distance_to_squared`/`distance_to_infinite_squared`,
//!   `Line.hpp:170-173,190,195-196`) -- [`line_distance_to_squared_i64`] and
//!   [`line_distance_to_infinite_squared_i64`].
//! - `Dim = 3, Scalar = i64` (`Line3::distance_to_squared`, `Line.hpp:242-245`, the only
//!   `line_alg` call this unit's `Line3` makes) -- [`line3_distance_to_squared_i64`].
//! - `Dim = 2, Scalar = i64` through `line_alg::intersection` (`Line::intersection`,
//!   `Line.cpp:110-113`) -- [`line_intersection_i64`]. `Floating` resolves to `f64` either way
//!   (`i64` is not `std::is_floating_point`), so this is the one `Floating` this unit's own
//!   `intersection` instantiation needs.
//!
//! A fourth and fifth instantiation -- `Dim = 2, Scalar = f64` of the same two templates -- are
//! reached not from this unit's own two files but from `AABBTreeLines.hpp`
//! (`crates/slic3r-aabbtreelines/src/aabb_tree_lines.rs`, this same task), whose
//! `detail::closest_point_to_origin` and `detail::get_intersections_with_line` instantiate
//! `line_alg::distance_to_squared`/`line_alg::intersection` against [`Linef`] (see that crate's
//! module doc for why `Linef`, not the integer `Line`, is the one `AABBTreeLines` instantiation
//! this task can hand-design). Both templates are declared in *this* unit's own file
//! (`Line.hpp:26-153`), so their `f64` monomorphizations are written here too, as `pub` functions
//! with no in-scope caller inside `line.rs` itself:
//! [`line_distance_to_squared_f64`]/[`line_intersection_f64`].
//!
//! **Deferred for the same reason `Point.rs` defers its own Transform/Matrix family -- needs a
//! type from a crate this task does not own, and this task's own rule forbids inventing one:**
//! `Line::parallel_to(double)`/`perpendicular_to(double)` (`Line.cpp:73-76,98-101`, need
//! `Slic3r::Geometry::directions_parallel`/`directions_perpendicular`, `slic3r-geometry`, a
//! fan-out crate not in this crate's dependency list), `Line::clip_with_bbox` (`Line.cpp:115-124`,
//! needs `Geometry::liang_barsky_line_clipping`, same crate), `transform(const Linef3&, const
//! Transform3d&)` (`Line.cpp:15-25`, needs the deferred `Transform3d` `Point.rs`'s own module doc
//! names), and the Boost.Polygon `geometry_concept`/`segment_traits<Line>` specialisations
//! (`Line.hpp:324-339`, the same don't-translate-able plumbing `point.rs:203`'s own deferred
//! Boost specialisations are). Each is named at its own `// TODO(port):` site below.
//!
//! **`ThickLine`/`CurledLine` inherit `Line` (`:205-223`).** Rust has no inheritance; each is its
//! own struct carrying a plain `Line` field plus its own extra data, per [`Point`]'s own doc on
//! why `Point`/`Point3` are not newtypes over their Eigen base -- here the base *is* a concrete,
//! already-ported struct, so a field (not a flattened copy of `a`/`b`) is the faithful shape: a
//! caller that used `ThickLine`'s inherited `Line` methods at the pin reaches them through
//! `.line`, exactly as it reached them through the implicit upcast upstream.

use slic3r_boundingbox::bounding_box::BoundingBox;
use slic3r_libslic3r::libslic3r::{EPSILON, PI};
use slic3r_point::point::{Point, Point3, Vec2d, Vec3d};

// -------------------------------------------------------------------------------------------
// line_alg (Line.hpp:26-153)
// -------------------------------------------------------------------------------------------

/// `inline typename Derived::Scalar cross2(...)` (`Point.hpp:121-126`), narrowed to the one
/// `Vec2d` instantiation this unit's own `line_alg::intersection`/`Line::intersection_infinite`
/// need -- see `Point.rs::cross2_f64`'s identical doc for why the fully generic Eigen-expression
/// form is not ported and why this is a second, separately written copy rather than a shared
/// `pub` function: `cross2_f64` there is private to that module (no caller outside it), so this
/// unit cannot reach it and re-derives the same one-line formula instead of widening that
/// module's own visibility for a function with no `pub` contract of its own.
fn cross2_f64(v1: Vec2d, v2: Vec2d) -> f64 {
    v1.x() * v2.y() - v1.y() * v2.x()
}

/// `(a - b)` promoted to `f64`, matching the vendored `(p1 - p2).cast<double>()` shape every
/// `line_alg` function and most of `Line`'s own methods use: the subtraction happens in
/// `coord_t` (`int64_t`) *before* the cast to `double`, so it can overflow for adversarial
/// coordinates near `i64::MIN`/`MAX` -- undefined behaviour by the C++ standard, which in
/// practice wraps on the two's-complement build this port is held against (no overflow trap, no
/// UBSan on this build), the same class of divergence `Point.rs`'s own `nearest_point_index_ptrs`
/// already documents and fixes the same way. `wrapping_sub` reproduces the oracle's own
/// two's-complement wraparound bit-for-bit instead of panicking on the overflow a plain `-`
/// would. **Found by `slic3r-line_line`'s own fuzz target** (`l1a: {x: -12255233, y:
/// 7234188376440569855}, l1b: {x: 7234017283807667300, y: 7234017283807667300}`, which panicked
/// `Line::midpoint`'s plain `+` before [`midpoint_coord`] below was added).
// FIDELITY(port): src/libslic3r/Line.hpp -- every `(p1 - p2).cast<double>()` in
// this unit subtracts two `coord_t` values before the cast; this port reproduces the oracle's own
// (UB-but-in-practice-wrapping) two's complement subtraction via `wrapping_sub` rather than
// panicking on overflow.
fn delta_f64(a: i64, b: i64) -> f64 {
    a.wrapping_sub(b) as f64
}

/// `(a + b) / 2`, matching the vendored `(this->a + this->b) / 2` shape [`Line::midpoint`]/
/// [`Line3::midpoint`] use: the addition happens in `coord_t` before the integer division by
/// `2`, so it is reproduced with `wrapping_add` for the same reason [`delta_f64`] is.
// FIDELITY(port): src/libslic3r/Line.hpp:167,237 -- `(a + b) / 2` adds two
// `coord_t` values before dividing; this port reproduces the oracle's own two's complement
// wraparound via `wrapping_add` rather than panicking on overflow (see [`delta_f64`]'s doc).
fn midpoint_coord(a: i64, b: i64) -> i64 {
    a.wrapping_add(b) / 2
}

/// `line_alg::distance_to_squared<L>(const L&, const Vec<Dim,Scalar>&, Vec<Dim,Scalar>*)`
/// (`Line.hpp:45-72`), `Dim = 2, Scalar = i64` (`Line`'s own instantiation). Returns
/// `(squared_distance, nearest_point)` -- the idiom map's "a non-const reference out-parameter"
/// -> "a return value" rule, folded into the primary return rather than a tuple of two
/// independently named values, since the vendored body always computes both together.
///
/// The `l2 == 0f64` and `t <= 0f64`/`t >= 1f64` arms return `get_a(line)`/`get_b(line)` verbatim
/// (exact integers, no rounding); the interior arm's `(get_a(line).cast<double>() + t *
/// v).cast<Scalar<L>>()` (`:70`) is the one site that rounds, and it truncates toward zero
/// (Eigen's `.cast<T>()` to an integral `T` is a C-style `static_cast`), not away from zero or to
/// nearest -- `docs/PORTING.md`'s "Rounding modes" rule is what pins this reading.
fn line_distance_to_squared_i64(a: Point, b: Point, point: Point) -> (f64, Point) {
    let v = Vec2d::new(delta_f64(b.x(), a.x()), delta_f64(b.y(), a.y()));
    let va = Vec2d::new(delta_f64(point.x(), a.x()), delta_f64(point.y(), a.y()));
    let l2 = v.x() * v.x() + v.y() * v.y();
    if l2 == 0f64 {
        return (va.x() * va.x() + va.y() * va.y(), a);
    }
    let t = (va.x() * v.x() + va.y() * v.y()) / l2;
    if t <= 0f64 {
        return (va.x() * va.x() + va.y() * va.y(), a);
    }
    if t >= 1f64 {
        let vb = Vec2d::new(delta_f64(point.x(), b.x()), delta_f64(point.y(), b.y()));
        return (vb.x() * vb.x() + vb.y() * vb.y(), b);
    }
    let nearest_x = a.x() as f64 + t * v.x();
    let nearest_y = a.y() as f64 + t * v.y();
    let dx = t * v.x() - va.x();
    let dy = t * v.y() - va.y();
    (
        dx * dx + dy * dy,
        Point::new(nearest_x as i64, nearest_y as i64),
    )
}

/// `line_alg::distance_to_infinite_squared<L>(const L&, const Vec<Dim,Scalar>&,
/// Vec<Dim,Scalar>*)` (`Line.hpp:91-107`), `Dim = 2, Scalar = i64`; see
/// [`line_distance_to_squared_i64`]'s doc for the same out-parameter and rounding notes -- this
/// template never clamps `t` to `[0, 1]`, so it has no `t <= 0f64`/`t >= 1f64` early-out arm at all.
fn line_distance_to_infinite_squared_i64(a: Point, b: Point, point: Point) -> (f64, Point) {
    let v = Vec2d::new(delta_f64(b.x(), a.x()), delta_f64(b.y(), a.y()));
    let va = Vec2d::new(delta_f64(point.x(), a.x()), delta_f64(point.y(), a.y()));
    let l2 = v.x() * v.x() + v.y() * v.y();
    if l2 == 0f64 {
        return (va.x() * va.x() + va.y() * va.y(), a);
    }
    let t = (va.x() * v.x() + va.y() * v.y()) / l2;
    let closest_x = a.x() as f64 + t * v.x();
    let closest_y = a.y() as f64 + t * v.y();
    // Review finding: this used to return `(point - closest).squaredNorm()` -- the formula
    // `Linef3::distance_to_infinite_squared` (`Line.hpp:298`) uses -- instead of the vendored
    // `(t * v - va).squaredNorm()` this template actually returns. The two are algebraically
    // equal and not equal in `f64` (no `-ffp-contract=off` FMA collapses the difference), and
    // this row's equality tier is `exact`.
    let dx = t * v.x() - va.x();
    let dy = t * v.y() - va.y();
    (
        dx * dx + dy * dy,
        Point::new(closest_x as i64, closest_y as i64),
    )
}

/// `line_alg::distance_to_squared<L>` (`Line.hpp:45-72`), `Dim = 3, Scalar = i64` --
/// `Line3::distance_to_squared`'s own instantiation (`Line.hpp:242-245`). Only the squared
/// distance is read by that caller (it discards the `Vec<3, coord_t>*` out-parameter, passing a
/// local the vendored no-out-parameter overload at `:76-80` allocates and drops), so this returns
/// `f64` alone rather than the unused nearest point -- the algorithm itself is unchanged, the
/// return value is merely narrowed to what the one in-scope caller reads.
fn line3_distance_to_squared_i64(a: Point3, b: Point3, point: Point3) -> f64 {
    let vx = delta_f64(b.x(), a.x());
    let vy = delta_f64(b.y(), a.y());
    let vz = delta_f64(b.z(), a.z());
    let vax = delta_f64(point.x(), a.x());
    let vay = delta_f64(point.y(), a.y());
    let vaz = delta_f64(point.z(), a.z());
    let l2 = vx * vx + vy * vy + vz * vz;
    if l2 == 0f64 {
        return vax * vax + vay * vay + vaz * vaz;
    }
    let t = (vax * vx + vay * vy + vaz * vz) / l2;
    if t <= 0f64 {
        return vax * vax + vay * vay + vaz * vaz;
    }
    if t >= 1f64 {
        let dx = delta_f64(point.x(), b.x());
        let dy = delta_f64(point.y(), b.y());
        let dz = delta_f64(point.z(), b.z());
        return dx * dx + dy * dy + dz * dz;
    }
    let dx = t * vx - vax;
    let dy = t * vy - vay;
    let dz = t * vz - vaz;
    dx * dx + dy * dy + dz * dz
}

/// `line_alg::distance_to_squared<L>` (`Line.hpp:45-72`), `Dim = 2, Scalar = f64` --
/// `AABBTreeLines.hpp`'s own `Linef` instantiation (see this module's own doc). `Scalar<L> =
/// f64` is already floating-point, so every cast the integer version performs is a no-op here;
/// the branching is otherwise identical.
pub fn line_distance_to_squared_f64(a: Vec2d, b: Vec2d, point: Vec2d) -> (f64, Vec2d) {
    let v = Vec2d::new(b.x() - a.x(), b.y() - a.y());
    let va = Vec2d::new(point.x() - a.x(), point.y() - a.y());
    let l2 = v.x() * v.x() + v.y() * v.y();
    if l2 == 0f64 {
        return (va.x() * va.x() + va.y() * va.y(), a);
    }
    let t = (va.x() * v.x() + va.y() * v.y()) / l2;
    if t <= 0f64 {
        return (va.x() * va.x() + va.y() * va.y(), a);
    }
    if t >= 1f64 {
        let vb = Vec2d::new(point.x() - b.x(), point.y() - b.y());
        return (vb.x() * vb.x() + vb.y() * vb.y(), b);
    }
    let nearest = Vec2d::new(a.x() + t * v.x(), a.y() + t * v.y());
    let dx = t * v.x() - va.x();
    let dy = t * v.y() - va.y();
    (dx * dx + dy * dy, nearest)
}

/// `line_alg::intersection<L>(const L&, const L&, Vec<Dim,Scalar>*)` (`Line.hpp:126-151`), `Dim =
/// 2, Scalar = i64`. `Floating` resolves to `f64` (`i64` is not `is_floating_point`), so every
/// intermediate is `f64` and the result is cast back to `i64` only at the very end --
/// `docs/PORTING.md`'s "Rounding modes" rule: Eigen's `.cast<Scalar<L>>()` truncates toward zero.
/// `#if 0`'d dead branch at `:134-139` is not ported (`docs/PORTING.md`'s don't-translate list).
fn line_intersection_i64(l1a: Point, l1b: Point, l2a: Point, l2b: Point) -> Option<Point> {
    let v1 = Vec2d::new(delta_f64(l1b.x(), l1a.x()), delta_f64(l1b.y(), l1a.y()));
    let v2 = Vec2d::new(delta_f64(l2b.x(), l2a.x()), delta_f64(l2b.y(), l2a.y()));
    let denom = cross2_f64(v1, v2);
    if denom.abs() < EPSILON {
        return None;
    }
    let v12 = Vec2d::new(delta_f64(l1a.x(), l2a.x()), delta_f64(l1a.y(), l2a.y()));
    let nume_a = cross2_f64(v2, v12);
    let nume_b = cross2_f64(v1, v12);
    let t1 = nume_a / denom;
    let t2 = nume_b / denom;
    if (0f64..=1f64).contains(&t1) && (0f64..=1f64).contains(&t2) {
        let x = l1a.x() as f64 + t1 * v1.x();
        let y = l1a.y() as f64 + t1 * v1.y();
        Some(Point::new(x as i64, y as i64))
    } else {
        None
    }
}

/// `line_alg::intersection<L>` (`Line.hpp:126-151`), `Dim = 2, Scalar = f64` --
/// `AABBTreeLines.hpp`'s own `Linef` instantiation (see this module's own doc); `Floating =
/// Scalar<L> = f64` directly, so no intermediate cast is needed at all.
pub fn line_intersection_f64(l1a: Vec2d, l1b: Vec2d, l2a: Vec2d, l2b: Vec2d) -> Option<Vec2d> {
    let v1 = Vec2d::new(l1b.x() - l1a.x(), l1b.y() - l1a.y());
    let v2 = Vec2d::new(l2b.x() - l2a.x(), l2b.y() - l2a.y());
    let denom = cross2_f64(v1, v2);
    if denom.abs() < EPSILON {
        return None;
    }
    let v12 = Vec2d::new(l1a.x() - l2a.x(), l1a.y() - l2a.y());
    let nume_a = cross2_f64(v2, v12);
    let nume_b = cross2_f64(v1, v12);
    let t1 = nume_a / denom;
    let t2 = nume_b / denom;
    if (0f64..=1f64).contains(&t1) && (0f64..=1f64).contains(&t2) {
        Some(Vec2d::new(l1a.x() + t1 * v1.x(), l1a.y() + t1 * v1.y()))
    } else {
        None
    }
}

// -------------------------------------------------------------------------------------------
// Line (Line.hpp:155-203, Line.cpp)
// -------------------------------------------------------------------------------------------

/// `class Line` (`:155-203`). `a`/`b` are public fields at the pin (plain `Point a; Point b;`,
/// read and written directly at call sites, unlike `Point`'s own Eigen-accessor-method members)
/// -- ported as public fields for the same reason `BoundingBox::min`/`max` are.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Line {
    pub a: Point,
    pub b: Point,
}

/// `typedef std::vector<Line> Lines;` (`:20`).
pub type Lines = Vec<Line>;

// TODO(port): src/libslic3r/Line.hpp:201-202 -- `static const constexpr int Dim = 2;`/`using
// Scalar = Point::Scalar;`, the `line_alg::Traits<Line>` default's own `Dim`/`Scalar` members.
// This unit monomorphizes each `line_alg` template by hand per the module doc rather than
// porting the generic `Traits` machinery itself, so there is no `line_alg::Traits<Line>` impl
// for these to be associated items of.

impl Line {
    /// `Line(const Point& _a, const Point& _b)` (`:159`).
    pub fn new(a: Point, b: Point) -> Self {
        Line { a, b }
    }

    /// `explicit operator Lines() const` (`:160`).
    pub fn to_lines(&self) -> Lines {
        vec![*self]
    }

    /// `void scale(double factor)` (`:161`).
    pub fn scale(&mut self, factor: f64) {
        self.a.mul_assign(factor);
        self.b.mul_assign(factor);
    }

    /// `void translate(const Point &v)` (`:162`).
    pub fn translate(&mut self, v: Point) {
        self.a.add_assign(v);
        self.b.add_assign(v);
    }

    /// `void translate(double x, double y)` (`:163`): constructs `Point(x, y)` through the
    /// rounding double constructor, exactly as the vendored overload's own call does.
    pub fn translate_xy(&mut self, x: f64, y: f64) {
        self.translate(Point::from_f64_rounded(x, y));
    }

    /// `void rotate(double angle, const Point &center)` (`:164`).
    pub fn rotate(&mut self, angle: f64, center: Point) {
        self.a.rotate_about(angle, center);
        self.b.rotate_about(angle, center);
    }

    /// `void reverse()` (`:165`).
    pub fn reverse(&mut self) {
        std::mem::swap(&mut self.a, &mut self.b);
    }

    /// `double length() const { return (b - a).cast<double>().norm(); }` (`:166`).
    pub fn length(&self) -> f64 {
        let dx = delta_f64(self.b.x(), self.a.x());
        let dy = delta_f64(self.b.y(), self.a.y());
        (dx * dx + dy * dy).sqrt()
    }

    /// `Point midpoint() const { return (this->a + this->b) / 2; }` (`:167`). `Point` has no
    /// `operator/` at the pin -- the vendored expression relies on `Eigen::Matrix`'s own
    /// elementwise `operator/(Scalar)` through `Point`'s base-class conversion, integer division
    /// truncating toward zero exactly as Rust's `/` on `i64` does.
    pub fn midpoint(&self) -> Point {
        Point::new(
            midpoint_coord(self.a.x(), self.b.x()),
            midpoint_coord(self.a.y(), self.b.y()),
        )
    }

    /// `bool intersection_infinite(const Line &other, Point* point) const` (`Line.cpp:27-46`).
    /// Returns `None` where the vendored body returns `false` -- both the near-parallel
    /// (`denom` near zero) and the overflow-guard arms (`result.x()`/`y()` outside `coord_t`'s
    /// range, `:38-39`) -- and `Some(point)` otherwise.
    pub fn intersection_infinite(&self, other: &Line) -> Option<Point> {
        let a1 = Vec2d::new(self.a.x() as f64, self.a.y() as f64);
        let v12 = Vec2d::new(
            delta_f64(other.a.x(), self.a.x()),
            delta_f64(other.a.y(), self.a.y()),
        );
        let v1 = Vec2d::new(
            delta_f64(self.b.x(), self.a.x()),
            delta_f64(self.b.y(), self.a.y()),
        );
        let v2 = Vec2d::new(
            delta_f64(other.b.x(), other.a.x()),
            delta_f64(other.b.y(), other.a.y()),
        );
        let denom = cross2_f64(v1, v2);
        if denom.abs() < EPSILON {
            return None;
        }
        let t1 = cross2_f64(v12, v2) / denom;
        let result_x = a1.x() + t1 * v1.x();
        let result_y = a1.y() + t1 * v1.y();
        let max = i64::MAX as f64;
        let min = i64::MIN as f64;
        if result_x > max || result_x < min || result_y > max || result_y < min {
            return None;
        }
        Some(Point::new(result_x as i64, result_y as i64))
    }

    /// `double distance_to_squared(const Point &point) const` (`:170`).
    pub fn distance_to_squared(&self, point: Point) -> f64 {
        line_distance_to_squared_i64(self.a, self.b, point).0
    }

    /// `double distance_to_squared(const Point &point, Point *closest_point) const` (`:171`),
    /// with the `Point*` out-parameter folded into the return value (idiom map).
    pub fn distance_to_squared_with_closest(&self, point: Point) -> (f64, Point) {
        line_distance_to_squared_i64(self.a, self.b, point)
    }

    /// `double distance_to(const Point &point) const` (`:172`).
    pub fn distance_to(&self, point: Point) -> f64 {
        self.distance_to_squared(point).sqrt()
    }

    /// `double distance_to_infinite_squared(const Point &point, Point *closest_point) const`
    /// (`:173`), out-parameter folded into the return value.
    pub fn distance_to_infinite_squared(&self, point: Point) -> (f64, Point) {
        line_distance_to_infinite_squared_i64(self.a, self.b, point)
    }

    /// `double perp_distance_to(const Point &point) const` (`Line.cpp:48-56`).
    pub fn perp_distance_to(&self, point: Point) -> f64 {
        let v = Vec2d::new(
            delta_f64(self.b.x(), self.a.x()),
            delta_f64(self.b.y(), self.a.y()),
        );
        let va = Vec2d::new(
            delta_f64(point.x(), self.a.x()),
            delta_f64(point.y(), self.a.y()),
        );
        if self.a == self.b {
            return (va.x() * va.x() + va.y() * va.y()).sqrt();
        }
        cross2_f64(v, va).abs() / (v.x() * v.x() + v.y() * v.y()).sqrt()
    }

    // TODO(port): src/libslic3r/Line.cpp:73-76 -- `parallel_to(double angle)`,
    // needs `Slic3r::Geometry::directions_parallel` (`slic3r-geometry`, not a dependency of this
    // crate; see the module doc).
    // TODO(port): src/libslic3r/Line.cpp:98-101 --
    // `perpendicular_to(double angle)`, needs `Slic3r::Geometry::directions_perpendicular`, same
    // crate.

    /// `bool parallel_to(const Line& line) const` (`Line.cpp:78-83`).
    pub fn parallel_to_line(&self, line: &Line) -> bool {
        let v1 = Vec2d::new(
            delta_f64(self.b.x(), self.a.x()),
            delta_f64(self.b.y(), self.a.y()),
        );
        let v2 = Vec2d::new(
            delta_f64(line.b.x(), line.a.x()),
            delta_f64(line.b.y(), line.a.y()),
        );
        let cross = cross2_f64(v1, v2);
        cross * cross
            < EPSILON
                * EPSILON
                * (v1.x() * v1.x() + v1.y() * v1.y())
                * (v2.x() * v2.x() + v2.y() * v2.y())
    }

    /// `bool overlap(const Line &line, double &overlap_length) const` (`Line.cpp:84-97`), the
    /// `double&` out-parameter folded into `Some(overlap_length)`/`None` (idiom map).
    pub fn overlap(&self, line: &Line) -> Option<f64> {
        if !self.parallel_to_line(line) {
            return None;
        }
        let line_ = Line::new(self.a, line.a);
        // Review finding: `Line.cpp:88`'s `scaled(EPSILON)` is the *integral* overload
        // (`Point.hpp:669-678`, `Tout` defaults to `coord_t`), which truncates toward zero --
        // not the `scale_()` macro (`libslic3r.h:106`, a plain `f64` division with no
        // truncation) this used to call. `slic3r_point::point::scaled_coord` is that same
        // truncating overload, already ported for exactly this call shape (`docs/PORTING.md`'s
        // "Scaling conversions" rule).
        if line_.length() > slic3r_point::point::scaled_coord(EPSILON) as f64
            && !self.parallel_to_line(&line_)
        {
            return None;
        }
        let a_min = self.a.x().min(self.b.x());
        let a_max = self.a.x().max(self.b.x());
        let b_min = line.a.x().min(line.b.x());
        let b_max = line.a.x().max(line.b.x());
        if a_min > b_max || a_max < b_min {
            return None;
        }
        // FIDELITY(port): src/libslic3r/Line.cpp:94 -- `std::min(a_max, b_max) -
        // std::max(a_min, b_min)` subtracts two `coord_t` values with the result kept as
        // `coord_t`; reproduces the oracle's own wraparound with `wrapping_sub` rather than
        // panicking on overflow (see [`delta_f64`]'s doc).
        let mut overlap_length = 0i64.max(a_max.min(b_max).wrapping_sub(a_min.max(b_min))) as f64;
        overlap_length /= (a_max as f64 - a_min as f64) / self.length();
        Some(overlap_length)
    }

    /// `bool perpendicular_to(const Line& line) const` (`Line.cpp:103-108`).
    pub fn perpendicular_to_line(&self, line: &Line) -> bool {
        let v1 = Vec2d::new(
            delta_f64(self.b.x(), self.a.x()),
            delta_f64(self.b.y(), self.a.y()),
        );
        let v2 = Vec2d::new(
            delta_f64(line.b.x(), line.a.x()),
            delta_f64(line.b.y(), line.a.y()),
        );
        let dot = v1.x() * v2.x() + v1.y() * v2.y();
        dot * dot
            < EPSILON
                * EPSILON
                * (v1.x() * v1.x() + v1.y() * v1.y())
                * (v2.x() * v2.x() + v2.y() * v2.y())
    }

    /// `double atan2_() const { return atan2(this->b(1) - this->a(1), this->b(0) - this->a(0)); }`
    /// (`:179`). `(1)`/`(0)` are Eigen's component-index operators, `y`/`x` respectively.
    pub fn atan2_(&self) -> f64 {
        delta_f64(self.b.y(), self.a.y()).atan2(delta_f64(self.b.x(), self.a.x()))
    }

    /// `double orientation() const` (`Line.cpp:58-63`).
    pub fn orientation(&self) -> f64 {
        let angle = self.atan2_();
        if angle < 0f64 {
            2f64 * PI + angle
        } else {
            angle
        }
    }

    /// `double direction() const` (`Line.cpp:65-71`).
    pub fn direction(&self) -> f64 {
        let atan2 = self.atan2_();
        if (atan2 - PI).abs() < EPSILON {
            0f64
        } else if atan2 < 0f64 {
            atan2 + PI
        } else {
            atan2
        }
    }

    /// `Vector vector() const { return this->b - this->a; }` (`:182`). Integer `wrapping_sub`,
    /// not `Point`'s own `Sub` -- see [`delta_f64`]'s doc for why the oracle's own
    /// `Vec2crd::operator-` wraps in two's complement on adversarial coordinates rather than
    /// trapping, and [`Line::normal`]'s identical note; `self.b - self.a` through `Point::sub`
    /// would panic on exactly the inputs this reproduces instead.
    // FIDELITY(port): src/libslic3r/Line.hpp:182 -- see [`delta_f64`]'s doc; this site
    // subtracts two `coord_t` values with the result kept as `coord_t` (no `f64` cast), so it
    // reproduces the oracle's own wraparound with `wrapping_sub` rather than `Point`'s own
    // (non-wrapping) `Sub`.
    pub fn vector(&self) -> Point {
        Point::new(
            self.b.x().wrapping_sub(self.a.x()),
            self.b.y().wrapping_sub(self.a.y()),
        )
    }

    /// `Vector normal() const` (`:183`). Integer `wrapping_sub`/`wrapping_neg`, not plain `-` --
    /// see [`delta_f64`]'s doc; this is the one site in this unit that both subtracts *and*
    /// negates two `coord_t` values with no intervening `f64` cast, so it needs the integer
    /// (not the float) wrapping primitives.
    // FIDELITY(port): src/libslic3r/Line.hpp:183 -- see [`delta_f64`]'s doc;
    // this site both subtracts and negates in `coord_t` with no `f64` cast, so it reproduces the
    // oracle's own wraparound with `wrapping_sub`/`wrapping_neg` instead of `delta_f64`.
    pub fn normal(&self) -> Point {
        Point::new(
            self.b.y().wrapping_sub(self.a.y()),
            self.b.x().wrapping_sub(self.a.x()).wrapping_neg(),
        )
    }

    /// `bool intersection(const Line& line, Point* intersection) const` (`Line.cpp:110-113`).
    pub fn intersection(&self, line: &Line) -> Option<Point> {
        line_intersection_i64(self.a, self.b, line.a, line.b)
    }

    // TODO(port): src/libslic3r/Line.cpp:115-124 -- `clip_with_bbox`, needs
    // `Geometry::liang_barsky_line_clipping` (`slic3r-geometry`, not a dependency; see the module
    // doc).

    /// `void extend(double offset)` (`Line.cpp:126-131`).
    pub fn extend(&mut self, offset: f64) {
        let vx = delta_f64(self.b.x(), self.a.x());
        let vy = delta_f64(self.b.y(), self.a.y());
        let len = (vx * vx + vy * vy).sqrt();
        let (ux, uy) = if len == 0f64 {
            (0f64, 0f64)
        } else {
            (vx / len, vy / len)
        };
        let offset_x = (offset * ux) as i64;
        let offset_y = (offset * uy) as i64;
        let offset_vector = Point::new(offset_x, offset_y);
        self.a.sub_assign(offset_vector);
        self.b.add_assign(offset_vector);
    }

    /// `static inline double distance_to_squared(const Point &point, const Point &a, const
    /// Point &b)` (`:190`).
    pub fn distance_to_squared_static(point: Point, a: Point, b: Point) -> f64 {
        line_distance_to_squared_i64(a, b, point).0
    }

    /// `static double distance_to(const Point &point, const Point &a, const Point &b)` (`:191`).
    pub fn distance_to_static(point: Point, a: Point, b: Point) -> f64 {
        Line::distance_to_squared_static(point, a, b).sqrt()
    }

    /// `static inline double distance_to_infinite_squared(...)` (`:195`).
    pub fn distance_to_infinite_squared_static(point: Point, a: Point, b: Point) -> f64 {
        line_distance_to_infinite_squared_i64(a, b, point).0
    }

    /// `static double distance_to_infinite(...)` (`:196`).
    pub fn distance_to_infinite_static(point: Point, a: Point, b: Point) -> f64 {
        Line::distance_to_infinite_squared_static(point, a, b).sqrt()
    }
}

/// `bool operator==(const Line &rhs) const` (`:169`) is `#[derive(PartialEq)]` above (field-wise
/// equality of two `Point`s, exactly what the vendored body compares).
///
/// `BoundingBox get_extents(const Lines &lines)` (`Line.cpp:140-148`).
pub fn get_extents(lines: &Lines) -> BoundingBox {
    let mut bbox = BoundingBox::new();
    for line in lines {
        bbox.merge(line.a);
        bbox.merge(line.b);
    }
    bbox
}

// TODO(port): src/libslic3r/Line.hpp:324-339 -- the Boost.Polygon
// `geometry_concept<Line>`/`segment_traits<Line>` specialisations; no in-scope caller reaches
// Boost.Polygon through this port, the same don't-translate-able plumbing `point.rs:203`'s own
// deferred Boost specialisations are.

// -------------------------------------------------------------------------------------------
// ThickLine (Line.hpp:205-213)
// -------------------------------------------------------------------------------------------

/// `class ThickLine : public Line` (`:205-213`); see the module doc for why this carries a
/// `line` field rather than flattening `a`/`b` or duplicating them.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct ThickLine {
    pub line: Line,
    pub a_width: f64,
    pub b_width: f64,
}

/// `typedef std::vector<ThickLine> ThickLines;` (`:22`).
pub type ThickLines = Vec<ThickLine>;

impl ThickLine {
    /// `ThickLine(const Point& a, const Point& b) : Line(a, b), a_width(0), b_width(0) {}`
    /// (`:209`).
    pub fn new(a: Point, b: Point) -> Self {
        ThickLine {
            line: Line::new(a, b),
            a_width: 0f64,
            b_width: 0f64,
        }
    }
    /// `ThickLine(const Point& a, const Point& b, double wa, double wb)` (`:210`).
    pub fn with_widths(a: Point, b: Point, a_width: f64, b_width: f64) -> Self {
        ThickLine {
            line: Line::new(a, b),
            a_width,
            b_width,
        }
    }
}

// -------------------------------------------------------------------------------------------
// CurledLine (Line.hpp:215-225)
// -------------------------------------------------------------------------------------------

/// `class CurledLine : public Line` (`:215-223`); see the module doc.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct CurledLine {
    pub line: Line,
    pub curled_height: f32,
}

/// `using CurledLines = std::vector<CurledLine>;` (`:225`).
pub type CurledLines = Vec<CurledLine>;

impl CurledLine {
    /// `CurledLine(const Point& a, const Point& b) : Line(a, b), curled_height(0.0f) {}`
    /// (`:219`).
    pub fn new(a: Point, b: Point) -> Self {
        CurledLine {
            line: Line::new(a, b),
            curled_height: 0f32,
        }
    }
    /// `CurledLine(const Point& a, const Point& b, float curled_height)` (`:220`).
    pub fn with_height(a: Point, b: Point, curled_height: f32) -> Self {
        CurledLine {
            line: Line::new(a, b),
            curled_height,
        }
    }
}

// -------------------------------------------------------------------------------------------
// Line3 (Line.hpp:227-252)
// -------------------------------------------------------------------------------------------

/// `class Line3` (`:227-252`); plain fields, as [`Line`]'s own `a`/`b`.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Line3 {
    pub a: Point3,
    pub b: Point3,
}

/// `typedef std::vector<Line3> Lines3;` (`:21`).
pub type Lines3 = Vec<Line3>;

impl Line3 {
    /// `Line3(const Point3& _a, const Point3& _b)` (`:231`).
    // TODO(port): src/libslic3r/Line.hpp:233 -- `Line3(const Vec3crd&, const Vec3crd&)`, the
    // "backward compatibility with Vec3crd" overload. `Vec3crd` is `Point3`'s own Eigen base at
    // the pin (`Point3 : public Vec3crd`, `Point.hpp:283`), so passing a `Vec3crd` is passing a
    // `Point3` by its base-class reference -- a no-op duplicate of the constructor above, not a
    // second shape this port must give its own `fn`.
    pub fn new(a: Point3, b: Point3) -> Self {
        Line3 { a, b }
    }

    /// `double length() const` (`:235`).
    pub fn length(&self) -> f64 {
        let dx = delta_f64(self.a.x(), self.b.x());
        let dy = delta_f64(self.a.y(), self.b.y());
        let dz = delta_f64(self.a.z(), self.b.z());
        (dx * dx + dy * dy + dz * dz).sqrt()
    }

    /// `Point3 vector() const` (`:236`). Integer `wrapping_sub`, not `Point3`'s own `Sub` --
    /// see [`Line::vector`]'s identical note.
    // FIDELITY(port): src/libslic3r/Line.hpp:236 -- see [`delta_f64`]'s doc; reproduces the
    // oracle's own wraparound with `wrapping_sub` rather than `Point3`'s own (non-wrapping)
    // `Sub`.
    pub fn vector(&self) -> Point3 {
        Point3::new(
            self.b.x().wrapping_sub(self.a.x()),
            self.b.y().wrapping_sub(self.a.y()),
            self.b.z().wrapping_sub(self.a.z()),
        )
    }

    /// `Point3 midpoint() const` (`:237`): each component divided by the vendored body's own
    /// literal `2` (integer division, truncating toward zero), not through `Point3::operator/`
    /// (which does not exist at the pin either).
    pub fn midpoint(&self) -> Point3 {
        Point3::new(
            midpoint_coord(self.a.x(), self.b.x()),
            midpoint_coord(self.a.y(), self.b.y()),
            midpoint_coord(self.a.z(), self.b.z()),
        )
    }

    /// `Line to_line() const { return Line(this->a.to_point(), this->b.to_point()); }` (`:240`).
    pub fn to_line(&self) -> Line {
        Line::new(self.a.to_point(), self.b.to_point())
    }

    /// `static inline double distance_to_squared(const Point3& point, const Point3& a, const
    /// Point3& b)` (`:242-245`).
    pub fn distance_to_squared(point: Point3, a: Point3, b: Point3) -> f64 {
        line3_distance_to_squared_i64(a, b, point)
    }
}

// -------------------------------------------------------------------------------------------
// Linef (Line.hpp:254-270)
// -------------------------------------------------------------------------------------------

/// `class Linef` (`:254-269`).
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Linef {
    pub a: Vec2d,
    pub b: Vec2d,
}

/// `using Linesf = std::vector<Linef>;` (`:270`).
pub type Linesf = Vec<Linef>;

impl Linef {
    /// `Linef(const Vec2d& _a, const Vec2d& _b)` (`:258`).
    pub fn new(a: Vec2d, b: Vec2d) -> Self {
        Linef { a, b }
    }

    /// `Vec2d vector() const { return this->b - this->a; }` (`:260`).
    pub fn vector(&self) -> Vec2d {
        self.b - self.a
    }

    /// `Vec2d unit_vector() const` (`:261`).
    pub fn unit_vector(&self) -> Vec2d {
        if self.length() == 0f64 {
            Vec2d::default()
        } else {
            let v = self.vector();
            let n = (v.x() * v.x() + v.y() * v.y()).sqrt();
            Vec2d::new(v.x() / n, v.y() / n)
        }
    }

    /// `double length() const { return vector().norm(); }` (`:262`).
    pub fn length(&self) -> f64 {
        let v = self.vector();
        (v.x() * v.x() + v.y() * v.y()).sqrt()
    }
}

// -------------------------------------------------------------------------------------------
// Linef3 (Line.hpp:272-318, Line.cpp)
// -------------------------------------------------------------------------------------------

/// `class Linef3` (`:272-316`).
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Linef3 {
    pub a: Vec3d,
    pub b: Vec3d,
}

/// `using Linesf3 = std::vector<Linef3>;` (`:318`).
pub type Linesf3 = Vec<Linef3>;

impl Linef3 {
    /// `Linef3(const Vec3d& _a, const Vec3d& _b)` (`:276`).
    pub fn new(a: Vec3d, b: Vec3d) -> Self {
        Linef3 { a, b }
    }

    /// `Linef3(const Vec2d& _a, const Vec2d& _b, double z)` (`:277`).
    pub fn from_2d(a: Vec2d, b: Vec2d, z: f64) -> Self {
        Linef3 {
            a: Vec3d::new(a.x(), a.y(), z),
            b: Vec3d::new(b.x(), b.y(), z),
        }
    }

    /// `Vec3d intersect_plane(double z) const` (`Line.cpp:133-138`).
    pub fn intersect_plane(&self, z: f64) -> Vec3d {
        let vx = self.b.x() - self.a.x();
        let vy = self.b.y() - self.a.y();
        let vz = self.b.z() - self.a.z();
        let t = (z - self.a.z()) / vz;
        Vec3d::new(self.a.x() + vx * t, self.a.y() + vy * t, z)
    }

    /// `void scale(double factor)` (`:280`).
    pub fn scale(&mut self, factor: f64) {
        self.a = Vec3d::new(
            self.a.x() * factor,
            self.a.y() * factor,
            self.a.z() * factor,
        );
        self.b = Vec3d::new(
            self.b.x() * factor,
            self.b.y() * factor,
            self.b.z() * factor,
        );
    }

    /// `Vec3d vector() const` (`:281`).
    pub fn vector(&self) -> Vec3d {
        Vec3d::new(
            self.b.x() - self.a.x(),
            self.b.y() - self.a.y(),
            self.b.z() - self.a.z(),
        )
    }

    /// `Vec3d unit_vector() const` (`:282`).
    pub fn unit_vector(&self) -> Vec3d {
        if self.length() == 0f64 {
            Vec3d::default()
        } else {
            let v = self.vector();
            let n = (v.x() * v.x() + v.y() * v.y() + v.z() * v.z()).sqrt();
            Vec3d::new(v.x() / n, v.y() / n, v.z() / n)
        }
    }

    /// `double length() const` (`:283`).
    pub fn length(&self) -> f64 {
        let v = self.vector();
        (v.x() * v.x() + v.y() * v.y() + v.z() * v.z()).sqrt()
    }

    /// `double distance_to_infinite_squared(const Vec3d &point, Vec3d *closest_point) const`
    /// (`:285-299`), out-parameter folded into the return value.
    pub fn distance_to_infinite_squared(&self, point: Vec3d) -> (f64, Vec3d) {
        let v = self.vector();
        let va = Vec3d::new(
            point.x() - self.a.x(),
            point.y() - self.a.y(),
            point.z() - self.a.z(),
        );
        let l2 = v.x() * v.x() + v.y() * v.y() + v.z() * v.z();
        if l2 == 0f64 {
            return (va.x() * va.x() + va.y() * va.y() + va.z() * va.z(), self.a);
        }
        let t = (va.x() * v.x() + va.y() * v.y() + va.z() * v.z()) / l2;
        let closest = Vec3d::new(
            self.a.x() + t * v.x(),
            self.a.y() + t * v.y(),
            self.a.z() + t * v.z(),
        );
        let dx = point.x() - closest.x();
        let dy = point.y() - closest.y();
        let dz = point.z() - closest.z();
        (dx * dx + dy * dy + dz * dz, closest)
    }

    /// `static inline double distance_to_infinite_squared(const Vec3d &point, const Vec3d &a,
    /// const Vec3d &b)` (`:306-309`).
    pub fn distance_to_infinite_squared_static(point: Vec3d, a: Vec3d, b: Vec3d) -> f64 {
        Linef3::new(a, b).distance_to_infinite_squared(point).0
    }
}

// TODO(port): src/libslic3r/Line.cpp:15-25 -- `transform(const Linef3&, const
// Transform3d&)`, needs the deferred `Transform3d` type `Point.rs`'s own module doc names (no
// in-scope caller in this task's own units; see this task's pull request).

#[cfg(test)]
mod tests {
    use super::*;
    use slic3r_proof::tiers::{compare_unit, Oracle};

    #[test]
    fn line_extend_moves_each_endpoint_outward() {
        let mut line = Line::new(Point::new(0, 0), Point::new(100, 0));
        line.extend(10.0);
        assert_eq!(line.a, Point::new(-10, 0));
        assert_eq!(line.b, Point::new(110, 0));
    }

    #[test]
    fn line_midpoint_truncates_toward_zero() {
        let line = Line::new(Point::new(1, 1), Point::new(2, 2));
        assert_eq!(line.midpoint(), Point::new(1, 1));
    }

    #[test]
    fn linef_unit_vector_of_a_zero_length_line_is_zero() {
        let line = Linef::new(Vec2d::new(5.0, 5.0), Vec2d::new(5.0, 5.0));
        assert_eq!(line.unit_vector(), Vec2d::default());
    }

    /// Proof path 1 (R19, `port/manifest.tsv`'s `Line.cpp` row):
    /// `Line::intersection_infinite` is externally linked, genuine `Line.cpp` code, called
    /// here through `crates/slic3r-ffi/cxx/proof_units/line.cpp`'s wrapper, reached through
    /// `slic3r-proof`'s re-export of the FFI crate's per-unit proof bridges
    /// (`docs/PORTING.md`, "Decisions recorded on 2026-11-25").
    // PROOF: 1
    #[test]
    fn line_intersection_infinite_matches_the_oracle() {
        let l1 = Line::new(Point::new(0, 0), Point::new(100, 100));
        let l2 = Line::new(Point::new(0, 100), Point::new(100, 0));
        let actual = l1.intersection_infinite(&l2);
        let oracle = slic3r_proof::proof_units::line::path1_intersection_infinite(
            l1.a.x(),
            l1.a.y(),
            l1.b.x(),
            l1.b.y(),
            l2.a.x(),
            l2.a.y(),
            l2.b.x(),
            l2.b.y(),
        );
        let actual_vec = match actual {
            Some(p) => vec![1i64, p.x(), p.y()],
            None => vec![0i64, 0, 0],
        };
        compare_unit("Line.cpp", Oracle::Built, actual_vec, oracle)
            .expect("Line::intersection_infinite must match the oracle exactly");
    }

    /// Proof path 2 (R19, `port/manifest.tsv`'s `Line.hpp` row): `length()`, `midpoint()` and
    /// `normal()` are `Line.hpp`'s own header-only, inline-defined methods -- no stage dump
    /// field is any one of these calls' own output in isolation, so this proof reaches the
    /// oracle through the per-unit bridge instead, naming oracle `built`, the same `proof path:
    /// 2`-with-no-dump-coverage shape `docs/PORTING.md`'s "Decisions recorded on 2026-11-26",
    /// decision 2, accepts (`ObjectID.hpp` is the merged precedent it names).
    // PROOF: 2
    #[test]
    fn line_header_only_methods_match_the_oracle() {
        let line = Line::new(Point::new(10, 20), Point::new(130, 220));
        let actual_length = line.length();
        let oracle_length = slic3r_proof::proof_units::line::path2_line_length(
            line.a.x(),
            line.a.y(),
            line.b.x(),
            line.b.y(),
        );
        compare_unit("Line.hpp", Oracle::Built, actual_length, oracle_length)
            .expect("Line::length() must match the oracle exactly");

        let mid = line.midpoint();
        let normal = line.normal();
        let oracle = slic3r_proof::proof_units::line::path2_line_midpoint_normal(
            line.a.x(),
            line.a.y(),
            line.b.x(),
            line.b.y(),
        );
        assert_eq!(
            vec![mid.x(), mid.y(), normal.x(), normal.y()],
            oracle,
            "Line::midpoint()/normal() must match the oracle exactly"
        );

        // Review finding: `distance_to_infinite_squared` had no oracle comparison anywhere in
        // the tree, which is what let it return `(point - closest).squaredNorm()` instead of
        // the vendored `(t * v - va).squaredNorm()` (`Line.hpp:106`) stand. `point = (0, 28)`
        // is chosen because it *discriminates* between those two expressions, not merely
        // because `t` is interior: against this `line` it gives `t = 0.007352941176470588`,
        // where the vendored formula yields `161.05882352941177` in `f64` and the pre-fix one
        // `161.0588235294118`. An interior `t` alone is not enough -- the earlier probe
        // `(50, 50)` has `t = 0.19852941176470587` and both expressions return bit-identical
        // `355.8823529411765`, so it could not have failed against the pre-fix code. Measured
        // over the integer points of the `0..=119` square: 3,815 of the 13,797 with interior
        // `t` discriminate, and `(0, 28)` is the first. The `closest_point` half of this
        // assertion is unaffected by that fix (the fix touched only the returned squared
        // distance), so it proves the out-parameter, not the formula.
        let point = Point::new(0, 28);
        let (actual_sq, actual_closest) = line.distance_to_infinite_squared(point);
        let oracle_dist = slic3r_proof::proof_units::line::path2_line_distance_to_infinite_squared(
            line.a.x(),
            line.a.y(),
            line.b.x(),
            line.b.y(),
            point.x(),
            point.y(),
        );
        compare_unit("Line.hpp", Oracle::Built, actual_sq, oracle_dist[0])
            .expect("Line::distance_to_infinite_squared must match the oracle exactly");
        assert_eq!(
            vec![actual_closest.x() as f64, actual_closest.y() as f64],
            oracle_dist[1..],
            "Line::distance_to_infinite_squared's own closest point must match the oracle"
        );
    }
}

// PORT STATUS
// source: src/libslic3r/Line.cpp
// source: src/libslic3r/Line.hpp
// confidence: medium
// tier: exact
// oracle: built
