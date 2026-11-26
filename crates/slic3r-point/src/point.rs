//! Port of `src/libslic3r/Point.hpp`/`Point.cpp` (task `56-foundation-point`, plan step 20): the
//! port's fixed-point coordinate type and the Eigen-backed vector aliases every later unit is
//! written against. `Point.hpp` is `template-heavy: yes` in `port/manifest.tsv`; `Point.cpp` is
//! `template-heavy: no` but is hand-designed alongside it in this same task because the two are
//! one unit (task.md: "`Point.hpp` and `Point.cpp` are the port's fixed-point coordinate type").
//!
//! **Scope this module hand-designs, and the line it draws.** `docs/PORTING.md`'s type map
//! names exactly `Point.hpp:52-69` -- the thirteen `Vec2`/`Vec3`/`Vec4` aliases over `coord_t`,
//! `int32_t`, `int64_t`, `float` and `double` -- as the Eigen vector aliases this task
//! hand-designs; [`Vec2`]/[`Vec3`]/[`Vec4`] below and their thirteen named aliases are that.
//! The fully generic `Mat<N, M, T>`/`Vec<N, T>` templates (`:44-48`) and the `Transform`/
//! `Matrix2f..Matrix4d`/`identity`/`has_reflection`/`get_base` family (`:83-96`, `:106,170-194`)
//! sit *outside* that cited line range and have no in-scope caller inside this task's own two
//! units (`cross2`/`ccw` below need one `f64` cross product, not an arbitrary `N`-dimensional
//! matrix) -- they are left as `// TODO(port):` markers rather than guessed at, named in this
//! task's pull request, for whichever fan-out unit first needs a concrete instantiation.
//!
//! **Deferred for the same reason -- needs a type from a crate this task does not own, and this
//! task's own rule forbids inventing one:** `Point::projection_onto` (both overloads, `Line`/
//! `MultiPoint`), `Point::ccw(const Line&)`, `Point3::projection_onto`/`Point3::ccw(const Line3&)`
//! (`Line3`/`MultiPoint3`), `get_extents` (all three overloads, `BoundingBox`/`BoundingBoxf`),
//! `to_polyline` (`Polyline`/`Polyline3`), `transform` (both overloads, `Transform3f`/
//! `Transform3d`, itself deferred per the paragraph above), and the four `to_string` overloads
//! (`LocalesUtils::float_to_string_decimal_point`, a fan-out module `slic3r-localesutils` has not
//! written yet). Each is named at its own `// TODO(port):` site below, against the vendored line
//! it would port.
//!
//! **`PointHash` (`:505-509`) is not ported as a standalone hasher.** It exists upstream only to
//! key `std::unordered_multimap<Vec2crd, ValueType, PointHash>` inside
//! [`ClosestPointInRadiusLookup`]; this port keys its own lookup structure by `(i64, i64)`
//! tuples (`HashMap`'s blanket `Hash` for tuples of `Hash` types), so there is no separate named
//! hasher type to carry over -- see [`ClosestPointInRadiusLookup`]'s own doc for why `HashMap`,
//! not the port's one insertion-ordered spelling, is this site's faithful Rust shape.

use std::collections::HashMap;
use std::ops::{Add, Neg, Sub};

use slic3r_libslic3r::libslic3r::{fast_round_up, scale_, EPSILON};

// -------------------------------------------------------------------------------------------
// The Eigen vector aliases (Point.hpp:52-69)
// -------------------------------------------------------------------------------------------

/// `template<int N, class T> using Vec = Eigen::Matrix<T, N, 1, Eigen::DontAlign, N, 1>;`
/// collapsed to a plain 2-element struct for `N = 2` -- see the module doc for why the fully
/// generic `N`-dimensional form is not ported. Eigen's `.x()`/`.y()` accessor-method idiom
/// (never bare field access at a call site) is kept as methods here, per `docs/PORTING.md`'s
/// idiom map ("a method on a class" -> "a method on the corresponding Rust type").
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Vec2<T> {
    x: T,
    y: T,
}

impl<T: Copy> Vec2<T> {
    pub fn new(x: T, y: T) -> Self {
        Vec2 { x, y }
    }
    pub fn x(&self) -> T {
        self.x
    }
    pub fn y(&self) -> T {
        self.y
    }
}

impl<T: Add<Output = T> + Copy> Add for Vec2<T> {
    type Output = Vec2<T>;
    fn add(self, rhs: Self) -> Self {
        Vec2::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl<T: Sub<Output = T> + Copy> Sub for Vec2<T> {
    type Output = Vec2<T>;
    fn sub(self, rhs: Self) -> Self {
        Vec2::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl<T: Neg<Output = T> + Copy> Neg for Vec2<T> {
    type Output = Vec2<T>;
    fn neg(self) -> Self {
        Vec2::new(-self.x, -self.y)
    }
}

/// The `N = 3` instantiation of the same collapsed `Vec<N, T>` template; see [`Vec2`]'s doc.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Vec3<T> {
    x: T,
    y: T,
    z: T,
}

impl<T: Copy> Vec3<T> {
    pub fn new(x: T, y: T, z: T) -> Self {
        Vec3 { x, y, z }
    }
    pub fn x(&self) -> T {
        self.x
    }
    pub fn y(&self) -> T {
        self.y
    }
    pub fn z(&self) -> T {
        self.z
    }
}

impl<T: Add<Output = T> + Copy> Add for Vec3<T> {
    type Output = Vec3<T>;
    fn add(self, rhs: Self) -> Self {
        Vec3::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}

impl<T: Sub<Output = T> + Copy> Sub for Vec3<T> {
    type Output = Vec3<T>;
    fn sub(self, rhs: Self) -> Self {
        Vec3::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

/// The `N = 4` instantiation; see [`Vec2`]'s doc. No in-scope caller in this task's own units
/// constructs one beyond the bare alias, so it carries no arithmetic impls yet -- a future
/// caller that needs one adds it against this same struct, named in its own pull request.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Vec4<T> {
    x: T,
    y: T,
    z: T,
    w: T,
}

impl<T: Copy> Vec4<T> {
    pub fn new(x: T, y: T, z: T, w: T) -> Self {
        Vec4 { x, y, z, w }
    }
    pub fn x(&self) -> T {
        self.x
    }
    pub fn y(&self) -> T {
        self.y
    }
    pub fn z(&self) -> T {
        self.z
    }
    pub fn w(&self) -> T {
        self.w
    }
}

/// `using Vec2crd = Eigen::Matrix<coord_t, 2, 1, Eigen::DontAlign>;` (`:52`).
// PORT SRC: src/libslic3r/Point.hpp:52
pub type Vec2crd = Vec2<i64>;
/// `using Vec3crd = Eigen::Matrix<coord_t, 3, 1, Eigen::DontAlign>;` (`:53`).
// PORT SRC: src/libslic3r/Point.hpp:53
pub type Vec3crd = Vec3<i64>;
/// `using Vec2i32 = Eigen::Matrix<int32_t, 2, 1, Eigen::DontAlign>;` (`:57`).
// PORT SRC: src/libslic3r/Point.hpp:57
pub type Vec2i32 = Vec2<i32>;
/// `using Vec2i64 = Eigen::Matrix<int64_t, 2, 1, Eigen::DontAlign>;` (`:58`).
// PORT SRC: src/libslic3r/Point.hpp:58
pub type Vec2i64 = Vec2<i64>;
/// `using Vec3i32 = Eigen::Matrix<int32_t, 3, 1, Eigen::DontAlign>;` (`:59`).
// PORT SRC: src/libslic3r/Point.hpp:59
pub type Vec3i32 = Vec3<i32>;
/// `using Vec3i64 = Eigen::Matrix<int64_t, 3, 1, Eigen::DontAlign>;` (`:60`).
// PORT SRC: src/libslic3r/Point.hpp:60
pub type Vec3i64 = Vec3<i64>;
/// `using Vec4i32 = Eigen::Matrix<int32_t, 4, 1, Eigen::DontAlign>;` (`:61`).
// PORT SRC: src/libslic3r/Point.hpp:61
pub type Vec4i32 = Vec4<i32>;
/// `using Vec2f = Eigen::Matrix<float, 2, 1, Eigen::DontAlign>;` (`:64`).
// PORT SRC: src/libslic3r/Point.hpp:64
pub type Vec2f = Vec2<f32>;
/// `using Vec3f = Eigen::Matrix<float, 3, 1, Eigen::DontAlign>;` (`:65`).
// PORT SRC: src/libslic3r/Point.hpp:65
pub type Vec3f = Vec3<f32>;
/// `using Vec4f = Eigen::Matrix<float, 4, 1, Eigen::DontAlign>;` (`:66`).
// PORT SRC: src/libslic3r/Point.hpp:66
pub type Vec4f = Vec4<f32>;
/// `using Vec2d = Eigen::Matrix<double, 2, 1, Eigen::DontAlign>;` (`:67`).
// PORT SRC: src/libslic3r/Point.hpp:67
pub type Vec2d = Vec2<f64>;
/// `using Vec3d = Eigen::Matrix<double, 3, 1, Eigen::DontAlign>;` (`:68`).
// PORT SRC: src/libslic3r/Point.hpp:68
pub type Vec3d = Vec3<f64>;
/// `using Vec4d = Eigen::Matrix<double, 4, 1, Eigen::DontAlign>;` (`:69`).
// PORT SRC: src/libslic3r/Point.hpp:69
pub type Vec4d = Vec4<f64>;

// TODO(port): src/libslic3r/Point.hpp:44-48 -- the fully generic `Mat<N, M, T>`/`Vec<N, T>`
// templates; no in-scope caller of this task's own units instantiates one beyond the thirteen
// named aliases above (see the module doc).
// TODO(port): src/libslic3r/Point.hpp:83-96 -- Matrix2f..Matrix4d and Transform2f..Transform3d
// (Eigen::Transform); no in-scope caller in this task's own units.
// TODO(port): src/libslic3r/Point.hpp:106-108,170,179,187-194 -- identity<N,T>/identity3f/
// identity3d, has_reflection and the get_base family, all over the deferred Transform type above.
// TODO(port): src/libslic3r/Point.hpp:759-783 -- `std::hash<Vec2ds>`/`std::hash<vector<int>>`
// specialisations for `ConfigOptionPointsGroups`; no in-scope caller in this task's own units.
// TODO(port): src/libslic3r/Point.hpp:786-813 -- the Boost.Polygon `geometry_concept`/
// `point_traits`/`point_mutable_traits` specialisations for `Point`; no in-scope caller.
// TODO(port): src/libslic3r/Point.hpp:816-833 -- the Cereal `serialize`/`load`/`save`
// specialisations for the Eigen vector/matrix/transform aliases; no in-scope caller, and the
// `Matrix2f`/`Transform3d` arms need the deferred Transform/Matrix types above regardless.
// TODO(port): src/libslic3r/Point.hpp:836-848 -- the `Eigen::begin`/`Eigen::end` range-for
// helpers over the deferred generic `Mat<N, M, T>` above; no in-scope caller.

/// `inline typename Derived::Scalar cross2(const Eigen::MatrixBase<Derived> &v1, const
/// Eigen::MatrixBase<Derived2> &v2)` (`:121-126`), narrowed to the one instantiation
/// [`Point::ccw`] below actually needs (`Scalar = f64`) -- see the module doc for why the fully
/// generic form over an arbitrary Eigen expression is not ported. Not `pub`: the idiom map's "a
/// free function in an anonymous namespace, or a `static` free function" rule keeps a function
/// with no in-scope caller beyond this module private, proven by proof path 3 where needed rather
/// than by visibility; this one has an in-scope caller ([`Point::ccw`]) but still no caller
/// *outside* this module, so it stays private until one exists.
// PORT SRC: src/libslic3r/Point.hpp:125
fn cross2_f64(v1: Vec2d, v2: Vec2d) -> f64 {
    v1.x() * v2.y() - v1.y() * v2.x()
}
// PORT NOTE: src/libslic3r/Point.hpp:112-113 -- `template<int Options> int32_t cross2(...) =
// delete;` forbids calling the generic `cross2` with 32-bit-integer Eigen vectors at compile
// time, to stop a caller silently overflowing a 32-bit cross product. This port has no generic
// `cross2` at all -- only the private, monomorphic `cross2_f64` above, which can never be
// instantiated over `i32` -- so there is no overload for a Rust `= delete` counterpart to forbid;
// the type system already rules out the call the vendored deletion exists to catch.

// -------------------------------------------------------------------------------------------
// Point (Point.hpp:198-280, Point.cpp)
// -------------------------------------------------------------------------------------------

/// `class Point : public Vec2crd` (`:198-270`). Rust has no inheritance, so `Point` is its own
/// struct with the same two fields `Vec2crd` stores, rather than a newtype wrapping one: every
/// in-scope caller reaches `x`/`y` through the accessor methods below, exactly as it would reach
/// `Vec2crd::x()`/`y()` through the base class upstream.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Point {
    x: i64,
    y: i64,
}

/// `using Vector = Point;` (`:42`): the vendored codebase's own alias for "a `Point` used as a
/// displacement rather than a position" -- the same underlying type either way, so this is a
/// plain type alias, not a second struct.
pub type Vector = Point;

/// `using Points = std::vector<Point, PointsAllocator<Point>>;` (`:73`). `PointsAllocator`
/// (`tbb::scalable_allocator<Point>`, `:71-72`) is a custom allocator with no behavioural effect
/// on what a `Points` vector *contains*, only on where its backing memory comes from -- `Vec<T>`
/// with the global allocator is the faithful Rust shape per `docs/PORTING.md`'s idiom map (no
/// entry names a custom-allocator container, and none of this task's own call sites is
/// allocator-sensitive).
pub type Points = Vec<Point>;
/// `using PointPtrs = std::vector<Point*>;` (`:74`): a container of borrowed pointers, read-only
/// at every in-scope call site in this task's own units (`Point::nearest_point_index`'s own
/// overload below immediately narrows it to `PointConstPtrs`) -- ported as borrowed references,
/// per the idiom map's "a raw pointer that is not walking a contiguous Rust-owned buffer" ->
/// `port/ownership.tsv` rule read the other way: there is no struct *field* here for that file to
/// classify (`PointPtrs`/`PointConstPtrs` are function-parameter container aliases, never a
/// struct field), so this is a plain borrow, not a row this task invents a type for.
pub type PointPtrs<'a> = Vec<&'a Point>;
/// `using PointConstPtrs = std::vector<const Point*>;` (`:75`); see [`PointPtrs`]'s doc.
pub type PointConstPtrs<'a> = Vec<&'a Point>;
/// `using Pointfs = std::vector<Vec2d>;` (`:77`).
pub type Pointfs = Vec<Vec2d>;
/// `using Vec2ds = std::vector<Vec2d>;` (`:78`); the identical shape as [`Pointfs`] under its own
/// vendored name -- the vendored header gives the same `std::vector<Vec2d>` two aliases, and
/// this port gives it the same two `pub type`s rather than collapsing them, so a caller naming
/// either vendored alias finds its exact counterpart here.
pub type Vec2ds = Vec<Vec2d>;
/// `using Pointf3s = std::vector<Vec3d>;` (`:79`).
pub type Pointf3s = Vec<Vec3d>;
/// `using VecOfPoints = std::vector<Points, PointsAllocator<Points>>;` (`:81`); see [`Points`]'s
/// doc for why the custom allocator needs no counterpart.
pub type VecOfPoints = Vec<Points>;

/// `inline bool operator<(const Vec2d &lhs, const Vec2d &rhs)` (`:110`): lexicographic by `x`
/// then `y`, over plain `f64` comparisons (no `NaN` handling beyond what `<`/`==` already give
/// `f64`, matching the vendored body exactly) -- a free function rather than `impl Ord`/`PartialOrd`
/// for [`Vec2<f64>`], since `f64` has no total order and this predicate is the vendored one's
/// literal two-comparison body, not a claim that `Vec2d` is now totally ordered.
pub fn vec2d_less(lhs: Vec2d, rhs: Vec2d) -> bool {
    lhs.x() < rhs.x() || (lhs.x() == rhs.x() && lhs.y() < rhs.y())
}

impl Point {
    /// `Point() : Vec2crd(0, 0) {}` (`:203`).
    // PORT SRC: src/libslic3r/Point.hpp:203
    pub fn zero() -> Self {
        Point { x: 0, y: 0 }
    }

    /// The four integer constructors (`:204-207`) collapse to one: Rust has no overload
    /// resolution, and every in-scope caller passes two values that fit in `i64` regardless of
    /// whether the vendored call site chose the `int32_t` or `int64_t` overload -- the stored
    /// width is `coord_t` (`i64`) either way.
    pub fn new(x: i64, y: i64) -> Self {
        Point { x, y }
    }

    /// `Point(double x, double y) : Vec2crd(coord_t(std::round(x)), coord_t(std::round(y))) {}`
    /// (`:208`): round-half-away-from-zero (`std::round`), **not** [`fast_round_up`]'s
    /// round-half-up -- the two agree everywhere except exactly on a `.5` boundary with a
    /// negative input, where this constructor's own vendored choice is `std::round`, not the
    /// `floor(a + 0.5)` idiom `Point::rotate` below uses. Each is ported against its own call
    /// site's literal vendored function, never substituted for the other.
    // PORT SRC: src/libslic3r/Point.hpp:208
    pub fn from_f64_rounded(x: f64, y: f64) -> Self {
        Point {
            x: x.round() as i64,
            y: y.round() as i64,
        }
    }

    /// `static Point new_scale(coordf_t x, coordf_t y) { return Point(coord_t(scale_(x)),
    /// coord_t(scale_(y))); }` (`:214`): `scale_` (`libslic3r.h:106`, `slic3r_libslic3r::scale_`)
    /// is a plain division with no rounding of its own; the `coord_t(...)` C-style cast that
    /// wraps it truncates toward zero, exactly as Rust's `as i64` does for an in-range `f64`.
    pub fn new_scale(x: f64, y: f64) -> Self {
        Point {
            x: scale_(x) as i64,
            y: scale_(y) as i64,
        }
    }
    /// `template<typename OtherDerived> static Point new_scale(const Eigen::MatrixBase<OtherDerived>
    /// &v)` (`:215-216`): the vendored overload is a generic Eigen-expression template; this port
    /// has no generic Eigen-expression type (only the concrete [`Vec2`]/[`Vec3`]/[`Vec4`] structs,
    /// per the module doc), so it is narrowed to the one concrete shape an in-scope caller can
    /// actually hold, `Vec2d` -- the same computation as [`Point::new_scale`], read from a vector
    /// rather than two scalars.
    pub fn new_scale_vec2(v: Vec2d) -> Self {
        Point::new_scale(v.x(), v.y())
    }

    pub fn x(&self) -> i64 {
        self.x
    }
    pub fn y(&self) -> i64 {
        self.y
    }

    /// `Point& operator+=(const Point& rhs)` (`:226`).
    pub fn add_assign(&mut self, rhs: Point) {
        self.x += rhs.x;
        self.y += rhs.y;
    }

    /// `Point& operator-=(const Point& rhs)` (`:227`).
    pub fn sub_assign(&mut self, rhs: Point) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }

    /// `Point& operator*=(const double &rhs) { x = coord_t(x * rhs); y = coord_t(y * rhs); }`
    /// (`:228`): each component promoted to `f64`, multiplied, then truncated toward zero back to
    /// `coord_t` -- never rounded, matching the vendored C-style cast exactly.
    // PORT SRC: src/libslic3r/Point.hpp:228
    pub fn mul_assign(&mut self, rhs: f64) {
        self.x = ((self.x as f64) * rhs) as i64;
        self.y = ((self.y as f64) * rhs) as i64;
    }

    /// `Point operator*(const double &rhs) { return Point(x * rhs, y * rhs); }` (`:229`): the
    /// member overload, which (unlike the free `operator*` at `:277-280`) truncates through the
    /// `Point(double, double)` constructor's `std::round` -- the vendored source calls the
    /// *rounding* constructor here, not a bare C-style cast, so this overload rounds while
    /// [`Point::scaled_mul`] (the free-function overload) truncates. Named separately because
    /// Rust has no overload resolution on which `operator*` a call site picked.
    pub fn mul_rounded(&self, rhs: f64) -> Point {
        Point::from_f64_rounded(self.x as f64 * rhs, self.y as f64 * rhs)
    }

    /// `inline Point operator* (const Point& l, const double& r) { return { coord_t(l.x() * r),
    /// coord_t(l.y() * r) }; }` (`:277-280`): the free-function overload, truncating toward zero
    /// via the brace-init `coord_t(...)` cast -- see [`Point::mul_rounded`]'s doc for why this is
    /// a second, separately named function rather than the same one.
    // PORT SRC: src/libslic3r/Point.hpp:279
    pub fn scaled_mul(&self, rhs: f64) -> Point {
        Point {
            x: (self.x as f64 * rhs) as i64,
            y: (self.y as f64 * rhs) as i64,
        }
    }

    /// `bool both_comp(const Point &rhs, const std::string& op)` (`:230-236`): the vendored
    /// signature takes the comparison operator as a runtime string (`">"`/`"<"`), matched inside
    /// the body; Rust's idiom for "pick one of two fixed comparisons" is two named predicates
    /// rather than a string-typed parameter re-matched on every call, so this collapses to
    /// [`Point::both_greater`]/[`Point::both_less`] below.
    pub fn both_greater(&self, rhs: &Point) -> bool {
        self.x > rhs.x && self.y > rhs.y
    }
    /// See [`Point::both_greater`]'s doc; the `op == "<"` branch (`:234`).
    pub fn both_less(&self, rhs: &Point) -> bool {
        self.x < rhs.x && self.y < rhs.y
    }
    /// `bool any_comp(const Point &rhs, const std::string &op)` (`:237-244`); see
    /// [`Point::both_greater`]'s doc for why the string-typed operator collapses to two named
    /// predicates.
    pub fn any_greater(&self, rhs: &Point) -> bool {
        self.x > rhs.x || self.y > rhs.y
    }
    /// See [`Point::any_greater`]'s doc; the `op == "<"` branch (`:242`).
    pub fn any_less(&self, rhs: &Point) -> bool {
        self.x < rhs.x || self.y < rhs.y
    }

    /// `void rotate(double angle) { this->rotate(std::cos(angle), std::sin(angle)); }` (`:246`).
    pub fn rotate(&mut self, angle: f64) {
        self.rotate_cos_sin(angle.cos(), angle.sin());
    }

    /// `void rotate(double cos_a, double sin_a)` (`:247-252`): rotates in place about the
    /// origin, each output component rounded with `round()` (C's `::round`, round-half-away-from-
    /// zero -- **not** [`fast_round_up`]'s round-half-up; see [`Point::from_f64_rounded`]'s doc
    /// for the same distinction at a different call site).
    // PORT SRC: src/libslic3r/Point.hpp:250
    // PORT SRC: src/libslic3r/Point.hpp:251
    pub fn rotate_cos_sin(&mut self, cos_a: f64, sin_a: f64) {
        let cur_x = self.x as f64;
        let cur_y = self.y as f64;
        self.x = (cos_a * cur_x - sin_a * cur_y).round() as i64;
        self.y = (sin_a * cur_x + cos_a * cur_y).round() as i64;
    }

    /// `void rotate(double angle, const Point &center)` (`Point.cpp:57-65`): rotates about
    /// `center` rather than the origin, with [`fast_round_up`] (`libslic3r.h`'s
    /// `fast_round_up<coord_t>`, round-half-up, special-cased exactly at the one boundary value
    /// its own doc comment states) -- the vendored source's *own* choice of rounding function
    /// differs between this overload and the two above it, and both are ported against their own
    /// literal call site rather than unified.
    pub fn rotate_about(&mut self, angle: f64, center: Point) {
        let s = angle.sin();
        let c = angle.cos();
        let cur = Vec2d::new(self.x as f64, self.y as f64);
        let center_d = Vec2d::new(center.x as f64, center.y as f64);
        let d = cur - center_d;
        self.x = fast_round_up::<i64>(center.x as f64 + c * d.x() - s * d.y());
        self.y = fast_round_up::<i64>(center.y as f64 + s * d.x() + c * d.y());
    }

    /// `Point rotated(double angle) const` (`:255`).
    pub fn rotated(&self, angle: f64) -> Point {
        let mut res = *self;
        res.rotate(angle);
        res
    }
    /// `Point rotated(double cos_a, double sin_a) const` (`:256`).
    pub fn rotated_cos_sin(&self, cos_a: f64, sin_a: f64) -> Point {
        let mut res = *self;
        res.rotate_cos_sin(cos_a, sin_a);
        res
    }
    /// `Point rotated(double angle, const Point &center) const` (`:257`).
    pub fn rotated_about(&self, angle: f64, center: Point) -> Point {
        let mut res = *self;
        res.rotate_about(angle, center);
        res
    }
    /// `Point rotate_90_degree_ccw() const { return Point(-this->y(), this->x()); }` (`:258`).
    pub fn rotate_90_degree_ccw(&self) -> Point {
        Point {
            x: -self.y,
            y: self.x,
        }
    }

    /// `int nearest_point_index(const Points &points) const` (`Point.cpp:197-204`): wraps every
    /// point into a borrowed-pointer vector and dispatches to the `PointConstPtrs` overload --
    /// ported directly, since [`PointConstPtrs`] is already `Vec<&Point>` (see its own doc).
    pub fn nearest_point_index(&self, points: &Points) -> i32 {
        let refs: PointConstPtrs = points.iter().collect();
        self.nearest_point_index_ptrs(&refs)
    }

    /// `int nearest_point_index(const PointConstPtrs &points) const` (`Point.cpp:206-229`): `-1`
    /// if `points` is empty, else the index of the closest point by squared distance, with the
    /// vendored early-outs on the running-best per-axis distance kept exactly (`:212-220`) rather
    /// than simplified to a plain squared-distance scan -- a simplification that keeps the same
    /// *result* here (both compute the true nearest point either way) but is kept faithful
    /// because a later unit may copy this loop shape against a case where the early-out changes
    /// which candidate is examined, not just how fast.
    // FIDELITY(port): src/libslic3r/Point.cpp:213 -- `(*this)(0) - (*it)->x()` subtracts two
    // `coord_t` (`int64_t`) values *before* `sqr<double>` converts the result, so the oracle's
    // own subtraction can overflow signed 64-bit arithmetic for two arbitrary far-apart points
    // (undefined behaviour by the C++ standard, which in practice wraps on the two's-complement
    // build this port is held against, since it is compiled without an overflow sanitizer).
    // Found by `slic3r-point_point`'s fuzz target's `Points` arm: a plain Rust `-` panics on the
    // same overflow instead of silently wrapping, which would make this port crash on inputs the
    // oracle merely computes a nonsensical (but defined-on-this-build) distance for. `wrapping_sub`
    // reproduces the oracle's own two's-complement wraparound bit-for-bit rather than papering
    // over the divergence with a widen-before-subtract that the oracle's call site does not do.
    pub fn nearest_point_index_ptrs(&self, points: &PointConstPtrs) -> i32 {
        let mut idx: i32 = -1;
        // PORT SRC: src/libslic3r/Point.cpp:209
        let mut distance: f64 = -1_f64;
        for (i, p) in points.iter().enumerate() {
            let dx = self.x.wrapping_sub(p.x) as f64;
            let mut d = dx * dx;
            // PORT SRC: src/libslic3r/Point.cpp:215
            if distance != -1_f64 && d > distance {
                continue;
            }
            let dy = self.y.wrapping_sub(p.y) as f64;
            d += dy * dy;
            // PORT SRC: src/libslic3r/Point.cpp:220
            if distance != -1_f64 && d > distance {
                continue;
            }
            idx = i as i32;
            distance = d;
            if distance < EPSILON {
                break;
            }
        }
        idx
    }

    /// `int nearest_point_index(const PointPtrs &points) const` (`Point.cpp:231-238`); see
    /// [`PointPtrs`]'s doc for why this is the same shape as the const-pointer overload.
    pub fn nearest_point_index_mut_ptrs(&self, points: &PointPtrs) -> i32 {
        let refs: PointConstPtrs = points.iter().map(|p| &**p).collect();
        self.nearest_point_index_ptrs(&refs)
    }

    /// `bool nearest_point(const Points &points, Point* point) const` (`Point.cpp:240-246`): the
    /// non-const reference out-parameter collapses to a return value, per the idiom map.
    pub fn nearest_point(&self, points: &Points) -> Option<Point> {
        let idx = self.nearest_point_index(points);
        if idx == -1 {
            None
        } else {
            Some(points[idx as usize])
        }
    }

    /// `double ccw(const Point &p1, const Point &p2) const` (`Point.cpp:74-79`): the signed area
    /// (times two) of the triangle `(p1, p2, *this)` -- positive for a counter-clockwise turn,
    /// negative for clockwise, zero for collinear. The vendored body's own live line (`:78`) casts
    /// both operands to `double` before [`cross2_f64`]; the `int64_t` line above it (`:77`) is
    /// commented out upstream and is not ported (`docs/PORTING.md`'s don't-translate list:
    /// upstream code that is commented out).
    // PORT SRC: src/libslic3r/Point.cpp:78
    pub fn ccw(&self, p1: Point, p2: Point) -> f64 {
        let v1 = Vec2d::new((p2.x - p1.x) as f64, (p2.y - p1.y) as f64);
        let v2 = Vec2d::new((self.x - p1.x) as f64, (self.y - p1.y) as f64);
        cross2_f64(v1, v2)
    }

    // TODO(port): src/libslic3r/Point.cpp:81-84 -- `Point::ccw(const Line &line) const`, needs
    // `Line` (`slic3r-line`, a fan-out crate this task does not own).
    // TODO(port): src/libslic3r/Point.hpp:266 -- `Point::projection_onto(const MultiPoint &)`,
    // needs `MultiPoint`/`Lines` (`slic3r-multipoint`).
    // TODO(port): src/libslic3r/Point.hpp:267 -- `Point::projection_onto(const Line &)`, needs
    // `Line`.

    /// `double ccw_angle(const Point &p1, const Point &p2) const` (`Point.cpp:88-95`): the CCW
    /// angle swept from `this->p1` to `this->p2`, normalised to `[0, 2*PI)`.
    // PORT SRC: src/libslic3r/Point.cpp:94
    pub fn ccw_angle(&self, p1: Point, p2: Point) -> f64 {
        let angle = ((p1.x - self.x) as f64).atan2((p1.y - self.y) as f64)
            - ((p2.x - self.x) as f64).atan2((p2.y - self.y) as f64);
        // PORT SRC: src/libslic3r/Point.cpp:94
        if angle <= 0_f64 {
            // PORT SRC: src/libslic3r/Point.cpp:94
            angle + 2_f64 * slic3r_libslic3r::libslic3r::PI
        } else {
            angle
        }
    }

    /// `double distance_to(const Point &point) const { return (point - *this).cast<double>()
    /// .norm(); }` (`:269`).
    pub fn distance_to(&self, point: Point) -> f64 {
        let dx = (point.x - self.x) as f64;
        let dy = (point.y - self.y) as f64;
        (dx * dx + dy * dy).sqrt()
    }
}

/// `inline bool operator<(const Point &l, const Point &r)` (`:272-275`): lexicographic by `x`
/// then `y`.
impl PartialOrd for Point {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Point {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // PORT SRC: src/libslic3r/Point.hpp:274
        self.x.cmp(&other.x).then_with(|| self.y.cmp(&other.y))
    }
}

impl Add for Point {
    type Output = Point;
    fn add(self, rhs: Point) -> Point {
        Point {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}
impl Sub for Point {
    type Output = Point;
    fn sub(self, rhs: Point) -> Point {
        Point {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

/// `inline std::ostream &operator<<(std::ostream &os, const Point &pt)` (`:393-397`): writes the
/// *unscaled* (millimetre) coordinates, comma-separated -- a human-facing `Display`, not emitted
/// G-code, so the idiom map's "never `Display` for compared output" caution does not apply here.
// PORT NOTE: src/libslic3r/Point.hpp:395 -- `std::ostream <<` on a `double` renders at its
// default precision, 6 significant digits (`unscale_(1234567)` prints `1.23457`); Rust's `{}`
// renders shortest-round-trip (`1.234567`). Not compared output (see this impl's own doc), so
// `docs/PORTING.md`'s "Number formatting" rule does not bind it, but noted here since a later
// unit that logs through this `Display` inherits the divergence.
impl std::fmt::Display for Point {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{},{}",
            slic3r_libslic3r::libslic3r::unscale_(self.x as f64),
            slic3r_libslic3r::libslic3r::unscale_(self.y as f64)
        )
    }
}

/// `std::ostream& operator<<(std::ostream &stm, const Vec2d &pointf)` (`Point.cpp:248-251`):
/// writes the raw components as stored, comma-separated -- unlike [`Point`]'s own `Display`
/// above, this does **not** unscale, exactly matching the vendored body's own plain `pointf(0)`/
/// `pointf(1)` reads.
// PORT NOTE: src/libslic3r/Point.cpp:250 -- same `std::ostream` 6-significant-digit vs. Rust
// shortest-round-trip divergence as [`Point`]'s own `Display`, see that impl's note.
impl std::fmt::Display for Vec2<f64> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{},{}", self.x(), self.y())
    }
}

// -------------------------------------------------------------------------------------------
// Point3 (Point.hpp:283-377, Point.cpp)
// -------------------------------------------------------------------------------------------

/// `class Point3 : public Vec3crd` (`:283-377`); see [`Point`]'s doc for why this is its own
/// struct rather than a newtype.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Point3 {
    x: i64,
    y: i64,
    z: i64,
}

/// `using Points3 = std::vector<Point3, PointsAllocator<Point3>>;` (`:76`); see [`Points`]'s doc
/// for why the custom allocator needs no counterpart.
pub type Points3 = Vec<Point3>;

impl Point3 {
    /// The three integer/default constructors (`:287-289`) collapse to one, the same way
    /// [`Point::new`] collapses `Point`'s four.
    pub fn new(x: i64, y: i64, z: i64) -> Self {
        Point3 { x, y, z }
    }

    /// `Point3(double x, double y, double z = 0.0)` (`:290`): round-half-away-from-zero
    /// (`std::round`), the same rounding [`Point::from_f64_rounded`] uses for `Point`'s own
    /// double constructor -- see that method's doc for why this is `std::round`, not
    /// [`fast_round_up`].
    // PORT SRC: src/libslic3r/Point.hpp:290
    pub fn from_f64_rounded(x: f64, y: f64, z: f64) -> Self {
        Point3 {
            x: x.round() as i64,
            y: y.round() as i64,
            z: z.round() as i64,
        }
    }

    /// `explicit Point3(const Vec2crd& vec2crd, coord_t z = 0)` (`:291`).
    pub fn from_vec2crd(vec2crd: Vec2crd, z: i64) -> Self {
        Point3 {
            x: vec2crd.x(),
            y: vec2crd.y(),
            z,
        }
    }

    /// `explicit Point3(const Vec3crd &vec3crd)` (`:292`): a plain component-wise copy, `Vec3crd`
    /// already being the same `(i64, i64, i64)` shape this struct stores.
    pub fn from_vec3crd(vec3crd: Vec3crd) -> Self {
        Point3 {
            x: vec3crd.x(),
            y: vec3crd.y(),
            z: vec3crd.z(),
        }
    }

    /// `static Point3 new_scale(coordf_t x, coordf_t y, coordf_t z)` (`:297-299`); see
    /// [`Point::new_scale`]'s doc for `scale_`'s truncating cast.
    pub fn new_scale(x: f64, y: f64, z: f64) -> Self {
        Point3 {
            x: scale_(x) as i64,
            y: scale_(y) as i64,
            z: scale_(z) as i64,
        }
    }
    /// `static Point3 new_scale(const Vec3d &v)` (`:300-302`); the same computation as
    /// [`Point3::new_scale`], read from a `Vec3d` rather than three scalars.
    pub fn new_scale_vec3(v: Vec3d) -> Self {
        Point3::new_scale(v.x(), v.y(), v.z())
    }
    /// `static Point3 new_scale(const Vec3f &v)` (`:303-305`); same as
    /// [`Point3::new_scale_vec3`], over `f32` components.
    pub fn new_scale_vec3_f32(v: Vec3f) -> Self {
        Point3::new_scale(v.x() as f64, v.y() as f64, v.z() as f64)
    }

    pub fn x(&self) -> i64 {
        self.x
    }
    pub fn y(&self) -> i64 {
        self.y
    }
    pub fn z(&self) -> i64 {
        self.z
    }

    /// `Point3& operator+=(const Point3& rhs)` (`:315`).
    pub fn add_assign(&mut self, rhs: Point3) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
    }
    /// `Point3& operator-=(const Point3& rhs)` (`:316`).
    pub fn sub_assign(&mut self, rhs: Point3) {
        self.x -= rhs.x;
        self.y -= rhs.y;
        self.z -= rhs.z;
    }
    /// `Point3& operator*=(const double &rhs)` (`:317-322`): each component promoted to `f64`,
    /// multiplied, then truncated back to `coord_t`; see [`Point::mul_assign`]'s identical doc.
    pub fn mul_assign(&mut self, rhs: f64) {
        self.x = ((self.x as f64) * rhs) as i64;
        self.y = ((self.y as f64) * rhs) as i64;
        self.z = ((self.z as f64) * rhs) as i64;
    }
    /// `Point3 operator*(const double &rhs) const` (`:323`): the member overload, which (like
    /// [`Point::mul_rounded`]) rounds through the `Point3(double, double, double)` constructor
    /// rather than truncating.
    pub fn mul_rounded(&self, rhs: f64) -> Point3 {
        Point3::from_f64_rounded(
            self.x as f64 * rhs,
            self.y as f64 * rhs,
            self.z as f64 * rhs,
        )
    }
    /// `inline Point3 operator* (const Point3& l, const double& r)` (`:388-391`): the
    /// free-function overload, truncating via the brace-init `coord_t(...)` cast; see
    /// [`Point::scaled_mul`]'s identical doc.
    pub fn scaled_mul(&self, rhs: f64) -> Point3 {
        Point3 {
            x: (self.x as f64 * rhs) as i64,
            y: (self.y as f64 * rhs) as i64,
            z: (self.z as f64 * rhs) as i64,
        }
    }

    /// `bool both_comp(const Point3 &rhs, const std::string& op)` (`:325-331`); see
    /// [`Point::both_greater`]'s doc for why the runtime-string operator collapses to two named
    /// predicates.
    pub fn both_greater(&self, rhs: &Point3) -> bool {
        self.x > rhs.x && self.y > rhs.y
    }
    /// See [`Point3::both_greater`]'s doc; the `op == "<"` branch (`:329`).
    pub fn both_less(&self, rhs: &Point3) -> bool {
        self.x < rhs.x && self.y < rhs.y
    }
    /// `bool any_comp(const Point3 &rhs, const std::string &op)` (`:332-339`); see
    /// [`Point::any_greater`]'s doc.
    pub fn any_greater(&self, rhs: &Point3) -> bool {
        self.x > rhs.x || self.y > rhs.y
    }
    /// See [`Point3::any_greater`]'s doc; the `op == "<"` branch (`:338`).
    pub fn any_less(&self, rhs: &Point3) -> bool {
        self.x < rhs.x || self.y < rhs.y
    }
    /// `bool any_comp(const coord_t val, const std::string &op)` (`:340-347`): the scalar
    /// overload, comparing both `x` and `y` against the same bare value rather than another
    /// `Point3`'s components.
    pub fn any_greater_val(&self, val: i64) -> bool {
        self.x > val || self.y > val
    }
    /// See [`Point3::any_greater_val`]'s doc; the `op == "<"` branch (`:346`).
    pub fn any_less_val(&self, val: i64) -> bool {
        self.x < val || self.y < val
    }

    /// `Point to_point() const { return Point(this->x(), this->y()); }` (`:372-374`).
    pub fn to_point(&self) -> Point {
        Point::new(self.x, self.y)
    }

    /// `void rotate(double angle) { this->rotate(std::cos(angle), std::sin(angle)); }`
    /// (`:349`): rotates the `(x, y)` plane about the origin, `z` unchanged.
    pub fn rotate(&mut self, angle: f64) {
        self.rotate_cos_sin(angle.cos(), angle.sin());
    }
    /// `void rotate(double cos_a, double sin_a)` (`:350-355`): same per-component formula as
    /// [`Point::rotate_cos_sin`], `round()`-rounded, `z` unchanged.
    // PORT SRC: src/libslic3r/Point.hpp:353
    // PORT SRC: src/libslic3r/Point.hpp:354
    pub fn rotate_cos_sin(&mut self, cos_a: f64, sin_a: f64) {
        let cur_x = self.x as f64;
        let cur_y = self.y as f64;
        self.x = (cos_a * cur_x - sin_a * cur_y).round() as i64;
        self.y = (sin_a * cur_x + cos_a * cur_y).round() as i64;
    }
    /// `void rotate(double angle, const Point3 &center)` (`Point.cpp:293-299`): rotates the
    /// `(x, y)` plane about `center`'s `(x, y)`, through [`Point::rotate`] -- the vendored body
    /// round-trips through a `Point3`-as-diff-then-sum rather than operating on `x`/`y` directly;
    /// this port keeps that same two-step shape (diff from center in 3D, rotate the 2D part,
    /// sum back) because the vendored `z` component of `diff`/`sum` is carried through unchanged
    /// either way and a direct `(x, y)`-only rewrite would be a second, unaudited order choice.
    pub fn rotate_about(&mut self, angle: f64, center: Point3) {
        let diff = Point3::new(self.x - center.x, self.y - center.y, self.z - center.z);
        let mut temp = Point::new(diff.x, diff.y);
        temp.rotate(angle);
        let sum = Point3::new(temp.x() + center.x, temp.y() + center.y, diff.z + center.z);
        *self = sum;
    }
    /// `Point3 rotated(double angle) const` (`:358`).
    pub fn rotated(&self, angle: f64) -> Point3 {
        let mut res = *self;
        res.rotate(angle);
        res
    }
    /// `Point3 rotated(double cos_a, double sin_a) const` (`:359`).
    pub fn rotated_cos_sin(&self, cos_a: f64, sin_a: f64) -> Point3 {
        let mut res = *self;
        res.rotate_cos_sin(cos_a, sin_a);
        res
    }
    /// `Point3 rotated(double angle, const Point3 &center) const` (`:360`).
    pub fn rotated_about(&self, angle: f64, center: Point3) -> Point3 {
        let mut res = *self;
        res.rotate_about(angle, center);
        res
    }
    /// `Point3 rotate_90_degree_ccw() const { return Point3(-this->y(), this->x(), this->z()); }`
    /// (`:361`).
    pub fn rotate_90_degree_ccw(&self) -> Point3 {
        Point3::new(-self.y, self.x, self.z)
    }

    /// `int nearest_point_index(const Points &points) const { return
    /// this->to_point().nearest_point_index(points); }` (`Point.cpp:301-303`).
    pub fn nearest_point_index(&self, points: &Points) -> i32 {
        self.to_point().nearest_point_index(points)
    }

    /// `bool nearest_point(const Points &points, Point3* point) const` (`Point.cpp:305-312`);
    /// the non-const reference out-parameter collapses to a return value, per the idiom map.
    pub fn nearest_point(&self, points: &Points) -> Option<Point3> {
        self.to_point()
            .nearest_point(points)
            .map(|pt2d| Point3::new(pt2d.x(), pt2d.y(), self.z))
    }

    /// `double ccw(const Point3 &p1, const Point3 &p2) const { return
    /// this->to_point().ccw(p1.to_point(), p2.to_point()); }` (`Point.cpp:314-316`).
    pub fn ccw(&self, p1: Point3, p2: Point3) -> f64 {
        self.to_point().ccw(p1.to_point(), p2.to_point())
    }

    // TODO(port): src/libslic3r/Point.cpp:318-323 -- `Point3::ccw(const Line3 &line) const`,
    // needs `Line3` (`slic3r-line`, a fan-out crate this task does not own).

    /// `double ccw_angle(const Point3 &p1, const Point3 &p2) const` (`Point.cpp:325-327`).
    pub fn ccw_angle(&self, p1: Point3, p2: Point3) -> f64 {
        self.to_point().ccw_angle(p1.to_point(), p2.to_point())
    }

    // TODO(port): src/libslic3r/Point.cpp:329-334 -- `Point3::projection_onto(const MultiPoint3
    // &)`, needs `MultiPoint3` (`slic3r-multipoint`); upstream itself throws
    // `RuntimeError("Point3::projection_onto(MultiPoint3) not implemented yet")` at the pin
    // (`:332`) rather than computing anything, so there is no behaviour here to port either way.
    // TODO(port): src/libslic3r/Point.cpp:336-354 -- `Point3::projection_onto(const Line3 &)`,
    // needs `Line3`.

    /// `double distance_to(const Point3 &point) const { return (point - *this).cast<double>()
    /// .norm(); }` (`:376`).
    pub fn distance_to(&self, point: Point3) -> f64 {
        let dx = (point.x - self.x) as f64;
        let dy = (point.y - self.y) as f64;
        let dz = (point.z - self.z) as f64;
        (dx * dx + dy * dy + dz * dz).sqrt()
    }
}

impl Add for Point3 {
    type Output = Point3;
    fn add(self, rhs: Point3) -> Point3 {
        Point3::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}
impl Sub for Point3 {
    type Output = Point3;
    fn sub(self, rhs: Point3) -> Point3 {
        Point3::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

/// `inline void append_points(Points &dst, const Points3 &src)` (`:380-386`).
pub fn append_points(dst: &mut Points, src: &Points3) {
    dst.extend(src.iter().map(Point3::to_point));
}

// TODO(port): src/libslic3r/Point.cpp:270-271 -- `to_polyline(const Points&)`/
// `to_polyline(const Points3&)` (declared src/libslic3r/Point.hpp:99-100), the two siblings of
// `to_points`/`to_points3` in the same vendored block, need `Polyline`/`Polyline3` from a fan-out
// crate this task does not own (see the module doc).

/// `Points to_points(const Points3 &points3)` (`Point.cpp:273-278`): drops each point's `z`,
/// through [`Point3::to_point`].
pub fn to_points(points3: &Points3) -> Points {
    points3.iter().map(Point3::to_point).collect()
}
/// `Points3 to_points3(const Points& points)` (`Point.cpp:280-286`): lifts each point to 3D with
/// `z = 0`, through the implicit `Point3(const Vec2crd&, coord_t z = 0)` conversion the vendored
/// `points3.emplace_back(pt)` invokes (`Point` derives `Vec2crd`) -- [`Point3::from_vec2crd`] is
/// that same conversion, ported explicitly since Rust has no implicit base-class slicing.
pub fn to_points3(points: &Points) -> Points3 {
    points
        .iter()
        .map(|pt| Point3::new(pt.x(), pt.y(), 0))
        .collect()
}

// TODO(port): src/libslic3r/Point.cpp:19-36 -- `transform(const std::vector<Vec3f>&,
// const Transform3f&)`, needs the deferred `Transform3f` type (see the module doc).
// TODO(port): src/libslic3r/Point.cpp:38-55 -- `transform(const Pointf3s&, const Transform3d&)`,
// needs the deferred `Transform3d` type.

/// `inline Vec2d unscale(coord_t x, coord_t y)` (`:158`).
pub fn unscale2(x: i64, y: i64) -> Vec2d {
    Vec2d::new(
        slic3r_libslic3r::libslic3r::unscale::<f64>(x),
        slic3r_libslic3r::libslic3r::unscale::<f64>(y),
    )
}
/// `inline Vec2d unscale(const Vec2crd &pt)` (`:159`).
pub fn unscale2_pt(pt: Vec2crd) -> Vec2d {
    unscale2(pt.x(), pt.y())
}
/// `inline Vec3d unscale(coord_t x, coord_t y, coord_t z)` (`:161`).
pub fn unscale3(x: i64, y: i64, z: i64) -> Vec3d {
    Vec3d::new(
        slic3r_libslic3r::libslic3r::unscale::<f64>(x),
        slic3r_libslic3r::libslic3r::unscale::<f64>(y),
        slic3r_libslic3r::libslic3r::unscale::<f64>(z),
    )
}
/// `inline Vec3d unscale(const Vec3crd &pt)` (`:162`).
pub fn unscale3_pt(pt: Vec3crd) -> Vec3d {
    unscale3(pt.x(), pt.y(), pt.z())
}
/// `inline Vec2d unscale(const Vec2d &pt)` (`:160`): `unscale<double>(Q)` with `Q = double`,
/// so `T(v) * T(SCALING_FACTOR)` is a plain `f64` multiplication of an already-`f64` input --
/// `slic3r_libslic3r::unscale` is narrowed to a `CoordT` (`i64`) input (that crate's own doc
/// explains why), so this overload is written out directly rather than reused.
pub fn unscale2_f64(pt: Vec2d) -> Vec2d {
    Vec2d::new(
        pt.x() * slic3r_libslic3r::libslic3r::SCALING_FACTOR,
        pt.y() * slic3r_libslic3r::libslic3r::SCALING_FACTOR,
    )
}
/// `inline Vec3d unscale(const Vec3d &pt)` (`:163`); see [`unscale2_f64`]'s doc.
pub fn unscale3_f64(pt: Vec3d) -> Vec3d {
    Vec3d::new(
        pt.x() * slic3r_libslic3r::libslic3r::SCALING_FACTOR,
        pt.y() * slic3r_libslic3r::libslic3r::SCALING_FACTOR,
        pt.z() * slic3r_libslic3r::libslic3r::SCALING_FACTOR,
    )
}

// TODO(port): src/libslic3r/Point.hpp:165-168 -- the four `to_string` overloads, need
// `LocalesUtils::float_to_string_decimal_point` (`slic3r-localesutils`, a fan-out crate whose
// modules Phase A/B has not written yet).

/// `inline bool has_reflection(...)`, `get_base`/`get_x_base`/`get_y_base`/`get_z_base` and their
/// `Transform3d::LinearPart` overloads (`:179-194`) are the deferred-Transform family named in
/// the module doc; `to_2d`/`to_3d`/`perp`/`angle` (`:128-156`), likewise outside the hand-designed
/// `:52-69` range and with no in-scope caller in this task's own units, are deferred the same way.
// TODO(port): src/libslic3r/Point.hpp:129-134 -- `perp`, generic over an arbitrary Eigen 2D
// vector expression; no in-scope caller.
// TODO(port): src/libslic3r/Point.hpp:137-144 -- `angle`, same reason.
// TODO(port): src/libslic3r/Point.hpp:147-150 -- `to_2d`, same reason.
// TODO(port): src/libslic3r/Point.hpp:153-156 -- `to_3d`, same reason.

/// `template<class Tout, class Tin, class = FloatingOnly<Tin>> inline constexpr
/// FloatingOnly<Tout> scaled(const Tin &v) noexcept { return Tout(v / Tin(SCALING_FACTOR)); }`
/// (`:660-667`), the floating-output overload: divides at the input's own precision (`Tin`) --
/// `v` is never widened to `f64` first -- then converts the quotient to `Tout`. Rust has no
/// overload resolution on the output type's own "is floating" trait the way the C++ SFINAE here
/// does (`docs/PORTING.md`'s "Scaling conversions" rule this binds), so the floating-output,
/// integral-output and Eigen overloads below are three separately named functions -- see the
/// module-level done-when line this satisfies for why they are kept visibly distinct rather than
/// collapsed with a mode flag.
///
/// `SCALING_FACTOR` is always `slic3r_libslic3r::libslic3r::SCALING_FACTOR` (`f64`); the `Tin(SCALING_FACTOR)`
/// cast in the vendored body is captured here by dividing in `f64` only when `Tin = f64`, and in
/// `f32` (casting the constant down first) when `Tin = f32` -- never the reverse.
pub fn scaled_f64_to_f64(v: f64) -> f64 {
    v / slic3r_libslic3r::libslic3r::SCALING_FACTOR
}
/// See [`scaled_f64_to_f64`]'s doc; `Tin = f32`, `Tout = f64`.
pub fn scaled_f32_to_f64(v: f32) -> f64 {
    (v / (slic3r_libslic3r::libslic3r::SCALING_FACTOR as f32)) as f64
}
/// See [`scaled_f64_to_f64`]'s doc; `Tin = f64`, `Tout = f32`.
pub fn scaled_f64_to_f32(v: f64) -> f32 {
    (v / slic3r_libslic3r::libslic3r::SCALING_FACTOR) as f32
}
/// See [`scaled_f64_to_f64`]'s doc; `Tin = f32`, `Tout = f32`.
pub fn scaled_f32_to_f32(v: f32) -> f32 {
    v / (slic3r_libslic3r::libslic3r::SCALING_FACTOR as f32)
}

/// `template<class Tout = coord_t, class Tin, class = FloatingOnly<Tin>> inline constexpr
/// ScaledCoordOnly<Tout> scaled(const Tin &v) noexcept { return Tout(v / Tin(SCALING_FACTOR)); }`
/// (`:673-678`), the integral-output overload (`Tout` defaults to, and every in-scope caller
/// instantiates, `coord_t`): divides at the input's own precision, exactly as
/// [`scaled_f64_to_f64`] does, then converts the quotient to `i64` by **truncation toward
/// zero** -- `Tout(quotient)`'s C-style cast on a floating value truncates, matching Rust's
/// `as i64`. **No rounding is added**: `Point.hpp:676`'s `std::round(...)` is commented out
/// upstream under the `TODO: is the rounding necessary?` comment at `:670-672`, and
/// `docs/PORTING.md`'s don't-translate list is explicit that this commented-out line is not
/// revived. Divides only by [`slic3r_libslic3r::libslic3r::SCALING_FACTOR`], never by its reciprocal, and
/// declares no second spelling of `0.000001`.
// PORT SRC: src/libslic3r/Point.hpp:677
pub fn scaled_coord(v: f64) -> i64 {
    (v / slic3r_libslic3r::libslic3r::SCALING_FACTOR) as i64
}
/// See [`scaled_coord`]'s doc; `Tin = f32`.
// PORT SRC: src/libslic3r/Point.hpp:677
pub fn scaled_coord_f32(v: f32) -> i64 {
    (v / (slic3r_libslic3r::libslic3r::SCALING_FACTOR as f32)) as i64
}

/// `template<class Tout = coord_t, class Tin, int N, class = FloatingOnly<Tin>, int...EigenArgs>
/// inline Eigen::Matrix<ArithmeticOnly<Tout>, N, EigenArgs...> scaled(const Eigen::Matrix<Tin, N,
/// EigenArgs...> &v) { return (v / SCALING_FACTOR).template cast<Tout>(); }` (`:681-690`), the
/// Eigen overload for `N = 2`: divides each element at the input's own precision, **then**
/// truncates each element to `i64` -- element-wise application of the same rule
/// [`scaled_coord`]/[`scaled_coord_f32`] state, per element, never through a running accumulator.
pub fn scaled_eigen2(v: Vec2d) -> Vec2<i64> {
    Vec2::new(scaled_coord(v.x()), scaled_coord(v.y()))
}
/// See [`scaled_eigen2`]'s doc; `Tin = f32`.
pub fn scaled_eigen2_f32(v: Vec2<f32>) -> Vec2<i64> {
    Vec2::new(scaled_coord_f32(v.x()), scaled_coord_f32(v.y()))
}
/// See [`scaled_eigen2`]'s doc; `N = 3`.
pub fn scaled_eigen3(v: Vec3d) -> Vec3<i64> {
    Vec3::new(
        scaled_coord(v.x()),
        scaled_coord(v.y()),
        scaled_coord(v.z()),
    )
}
/// See [`scaled_eigen3`]'s doc; `Tin = f32`.
pub fn scaled_eigen3_f32(v: Vec3<f32>) -> Vec3<i64> {
    Vec3::new(
        scaled_coord_f32(v.x()),
        scaled_coord_f32(v.y()),
        scaled_coord_f32(v.z()),
    )
}

/// `template<class Tout = double, class Tin, class = ArithmeticOnly<Tin>, class =
/// FloatingOnly<Tout>> inline constexpr Tout unscaled(const Tin &v) noexcept { return Tout(v) *
/// Tout(SCALING_FACTOR); }` (`:697-700`): casts `v` to `Tout` **first** (never through `f64` as
/// an intermediate when `Tout = f32`), then multiplies by `SCALING_FACTOR` cast to that same
/// `Tout` -- a pure multiplication that never truncates, unlike every `scaled*` function above.
/// `Tin` here is `coord_t` (`i64`), the one arithmetic input width every in-scope caller of this
/// task's own units would use.
// PORT SRC: src/libslic3r/Point.hpp:699
pub fn unscaled_f64(v: i64) -> f64 {
    (v as f64) * slic3r_libslic3r::libslic3r::SCALING_FACTOR
}
/// See [`unscaled_f64`]'s doc; `Tout = f32`.
// PORT SRC: src/libslic3r/Point.hpp:699
pub fn unscaled_f32(v: i64) -> f32 {
    (v as f32) * (slic3r_libslic3r::libslic3r::SCALING_FACTOR as f32)
}

/// `template<class Tout = double, class Tin, int N, class = ArithmeticOnly<Tin>, class =
/// FloatingOnly<Tout>, int...EigenArgs> inline constexpr Eigen::Matrix<Tout, N, EigenArgs...>
/// unscaled(const Eigen::Matrix<Tin, N, EigenArgs...> &v) noexcept { return v.template
/// cast<Tout>() * Tout(SCALING_FACTOR); }` (`:710-714`): element-wise [`unscaled_f64`], never
/// truncating.
pub fn unscaled_eigen2(v: Vec2crd) -> Vec2d {
    Vec2d::new(unscaled_f64(v.x()), unscaled_f64(v.y()))
}
/// See [`unscaled_eigen2`]'s doc; `N = 3`.
pub fn unscaled_eigen3(v: Vec3crd) -> Vec3d {
    Vec3d::new(
        unscaled_f64(v.x()),
        unscaled_f64(v.y()),
        unscaled_f64(v.z()),
    )
}

/// `inline coord_t align_to_grid(const coord_t coord, const coord_t spacing)` (`:718-727`):
/// rounds `coord` down to the nearest multiple of `spacing` at or below it, for either sign of
/// `coord` -- C++ integer division truncates toward zero for both signs, so a plain `coord /
/// spacing * spacing` rounds *up* (toward zero) for a negative `coord`; the `spacing - 1`
/// adjustment on the negative branch is what makes it round down instead, exactly mirroring the
/// vendored branch.
// PORT SRC: src/libslic3r/Point.hpp:722-724
pub fn align_to_grid(coord: i64, spacing: i64) -> i64 {
    if coord < 0 {
        (coord - spacing + 1) / spacing * spacing
    } else {
        coord / spacing * spacing
    }
}
/// `inline Point align_to_grid(Point coord, Point spacing)` (`:728-729`).
pub fn align_to_grid_point(coord: Point, spacing: Point) -> Point {
    Point::new(
        align_to_grid(coord.x(), spacing.x()),
        align_to_grid(coord.y(), spacing.y()),
    )
}
/// `inline coord_t align_to_grid(coord_t coord, coord_t spacing, coord_t base)` (`:730-731`).
pub fn align_to_grid_based(coord: i64, spacing: i64, base: i64) -> i64 {
    base + align_to_grid(coord - base, spacing)
}
/// `inline Point align_to_grid(Point coord, Point spacing, Point base)` (`:732-733`).
pub fn align_to_grid_point_based(coord: Point, spacing: Point, base: Point) -> Point {
    Point::new(
        align_to_grid_based(coord.x(), spacing.x(), base.x()),
        align_to_grid_based(coord.y(), spacing.y(), base.y()),
    )
}

/// `template<typename T> struct MinMax { T min; T max; };` (`:736`).
pub struct MinMax<T> {
    pub min: T,
    pub max: T,
}

/// `template<typename T> static bool apply(T &val, const MinMax<T> &limit)` (`:744-752`):
/// clamps `val` into `[limit.min, limit.max]` in place, returning whether it changed.
pub fn apply_minmax<T: PartialOrd + Copy>(val: &mut T, limit: &MinMax<T>) -> bool {
    if *val > limit.max {
        *val = limit.max;
        return true;
    }
    if *val < limit.min {
        *val = limit.min;
        return true;
    }
    false
}
/// `template<typename T> static bool apply(std::optional<T> &val, const MinMax<T> &limit)`
/// (`:738-741`): the `Option` overload, `false` when there is no value to clamp -- Rust has no
/// overload resolution on the two vendored `apply`s, so this is named separately rather than
/// sharing [`apply_minmax`]'s name.
pub fn apply_minmax_option<T: PartialOrd + Copy>(val: &mut Option<T>, limit: &MinMax<T>) -> bool {
    match val {
        None => false,
        Some(v) => apply_minmax(v, limit),
    }
}

/// `inline bool is_approx(const Point &p1, const Point &p2, coord_t epsilon =
/// coord_t(SCALED_EPSILON))` (`:399-403`). `SCALED_EPSILON` (`libslic3r.h:109`,
/// `slic3r_libslic3r::scaled_epsilon`) is a function, not a constant (that crate's own doc
/// explains why); its default here is that function's value truncated to `i64`, matching the
/// vendored `coord_t(SCALED_EPSILON)` cast exactly.
pub fn is_approx_point(p1: Point, p2: Point, epsilon: i64) -> bool {
    let d = p2 - p1;
    d.x().abs() < epsilon && d.y().abs() < epsilon
}
/// [`is_approx_point`]'s default `epsilon`, computed once per call rather than as a `const` --
/// see that function's own doc for why.
pub fn is_approx_point_default(p1: Point, p2: Point) -> bool {
    is_approx_point(p1, p2, slic3r_libslic3r::libslic3r::scaled_epsilon() as i64)
}

/// `inline bool is_approx(const Vec2f &p1, const Vec2f &p2, float epsilon = float(EPSILON))`
/// (`:405-409`).
pub fn is_approx_vec2f(p1: Vec2f, p2: Vec2f, epsilon: f32) -> bool {
    (p2.x() - p1.x()).abs() < epsilon && (p2.y() - p1.y()).abs() < epsilon
}
/// `inline bool is_approx(const Vec2d &p1, const Vec2d &p2, double epsilon = EPSILON)`
/// (`:411-415`).
pub fn is_approx_vec2d(p1: Vec2d, p2: Vec2d, epsilon: f64) -> bool {
    (p2.x() - p1.x()).abs() < epsilon && (p2.y() - p1.y()).abs() < epsilon
}
/// `inline bool is_approx(const Vec3f &p1, const Vec3f &p2, float epsilon = float(EPSILON))`
/// (`:417-421`).
pub fn is_approx_vec3f(p1: Vec3f, p2: Vec3f, epsilon: f32) -> bool {
    (p2.x() - p1.x()).abs() < epsilon
        && (p2.y() - p1.y()).abs() < epsilon
        && (p2.z() - p1.z()).abs() < epsilon
}
/// `inline bool is_approx(const Vec3d &p1, const Vec3d &p2, double epsilon = EPSILON)`
/// (`:423-427`).
pub fn is_approx_vec3d(p1: Vec3d, p2: Vec3d, epsilon: f64) -> bool {
    (p2.x() - p1.x()).abs() < epsilon
        && (p2.y() - p1.y()).abs() < epsilon
        && (p2.z() - p1.z()).abs() < epsilon
}

/// `inline Point lerp(const Point &a, const Point &b, double t)` (`:429-433`): linear
/// interpolation in `f64`, truncated (not rounded) back to `coord_t` by Eigen's own
/// `.cast<coord_t>()`, which is a plain element-wise C-style cast -- truncating, like every other
/// `.cast<integer>()` site in this header.
// PORT SRC: src/libslic3r/Point.hpp:432
pub fn lerp_point(a: Point, b: Point, t: f64) -> Point {
    let ax = a.x() as f64;
    let ay = a.y() as f64;
    let bx = b.x() as f64;
    let by = b.y() as f64;
    Point::new(
        // PORT SRC: src/libslic3r/Point.hpp:432
        ((1_f64 - t) * ax + t * bx) as i64,
        // PORT SRC: src/libslic3r/Point.hpp:432
        ((1_f64 - t) * ay + t * by) as i64,
    )
}
/// `inline Point3 lerp(const Point3& a, const Point3& b, double t)` (`:435-439`).
pub fn lerp_point3(a: Point3, b: Point3, t: f64) -> Point3 {
    let ax = a.x() as f64;
    let ay = a.y() as f64;
    let az = a.z() as f64;
    let bx = b.x() as f64;
    let by = b.y() as f64;
    let bz = b.z() as f64;
    Point3::new(
        // PORT SRC: src/libslic3r/Point.hpp:438
        ((1_f64 - t) * ax + t * bx) as i64,
        // PORT SRC: src/libslic3r/Point.hpp:438
        ((1_f64 - t) * ay + t * by) as i64,
        // PORT SRC: src/libslic3r/Point.hpp:438
        ((1_f64 - t) * az + t * bz) as i64,
    )
}

// TODO(port): src/libslic3r/Point.hpp:441-453 -- `get_extents<IncludeBoundary>(const Points&)`
// and `get_extents<IncludeBoundary>(const VecOfPoints&)` (`Point.cpp:165-186`), need
// `BoundingBox` (`slic3r-boundingbox`, a fan-out crate this task does not own).
// TODO(port): src/libslic3r/Point.hpp:455 -- `get_extents(const std::vector<Vec2d>&)`
// (`Point.cpp:188-194`), needs `BoundingBoxf`.

/// `bool has_duplicate_points(Points &&pts)` (`Point.cpp:138-145`): sorts a copy, then checks
/// successive elements -- the vendored signature takes ownership of its argument (`Points&&`)
/// precisely so it can sort in place rather than copy; this port's owned `Vec<Point>` parameter
/// is the same move.
pub fn has_duplicate_points(mut pts: Points) -> bool {
    pts.sort_unstable();
    for i in 1..pts.len() {
        if pts[i - 1] == pts[i] {
            return true;
        }
    }
    false
}
/// `inline bool has_duplicate_points(const Points &pts)` (`:460-464`): copies, then delegates.
pub fn has_duplicate_points_ref(pts: &Points) -> bool {
    has_duplicate_points(pts.clone())
}
/// `inline bool has_duplicate_successive_points(const Points &pts)` (`:468-474`): adjacent
/// elements only, no sort.
pub fn has_duplicate_successive_points(pts: &Points) -> bool {
    for i in 1..pts.len() {
        if pts[i - 1] == pts[i] {
            return true;
        }
    }
    false
}
/// `inline bool has_duplicate_successive_points_closed(const Points &pts)` (`:478-481`): also
/// treats the first and last point as adjacent.
pub fn has_duplicate_successive_points_closed(pts: &Points) -> bool {
    has_duplicate_successive_points(pts) || (pts.len() >= 2 && pts.first() == pts.last())
}

/// `Points collect_duplicates(Points pts)` (`Point.cpp:147-163`): every point that occurs more
/// than once in `pts`, each reported exactly once, in sorted order -- the vendored body's own
/// `!duplicits.empty() && duplicits.back() == *act` guard against reporting the same duplicate
/// twice is kept as a literal check against the last pushed element, not simplified to a `dedup`
/// call, since the two are only equivalent because the input is already sorted at this point.
pub fn collect_duplicates(mut pts: Points) -> Points {
    pts.sort_unstable();
    let mut duplicates: Points = Vec::new();
    // PORT NOTE: src/libslic3r/Point.cpp:151 -- the vendored body reads `*pts.begin()`
    // unconditionally before this loop, undefined behaviour on an empty `pts`; this port adds
    // the early return above it does not have, since Rust has no equivalent "read past the end"
    // to be faithful to and `Vec::is_empty()` is the one-line fix rather than a silent guess.
    if pts.is_empty() {
        return duplicates;
    }
    let mut prev = pts[0];
    for &act in pts.iter().skip(1) {
        if prev == act {
            if duplicates.last() != Some(&act) {
                duplicates.push(act);
            }
        }
        prev = act;
    }
    duplicates
}

/// `inline bool shorter_then(const Point& p0, const coord_t len)` (`:486-493`): true when `p0`'s
/// distance from the origin is at most `len`, computed with the same early per-axis bound check
/// and `int64_t` squared-norm the vendored body uses (`docs/PORTING.md`'s "Scaling conversions"
/// family does not cover this -- it is a plain magnitude check, not a scale conversion -- but the
/// widening to `i64` before squaring is load-bearing: `coord_t` is already `i64` at the pin, so
/// this is a same-width cast whose only purpose, upstream, is documentation that the
/// multiplication must not be done in a narrower type).
pub fn shorter_then(p0: Point, len: i64) -> bool {
    if p0.x() > len || p0.x() < -len {
        return false;
    }
    if p0.y() > len || p0.y() < -len {
        return false;
    }
    // `p0.cast<int64_t>().squaredNorm() <= Slic3r::sqr(int64_t(len))` (`:492`): plain `i64`
    // multiplication through the port's own `sqr`, not a wrapping substitute -- the two early
    // returns above already bound `|x|`/`|y|` to `len`, so this is the same arithmetic the
    // vendored line performs, held to the same overflow behaviour rather than silently widened.
    let x = p0.x();
    let y = p0.y();
    let squared_norm = slic3r_libslic3r::libslic3r::sqr(x) + slic3r_libslic3r::libslic3r::sqr(y);
    squared_norm <= slic3r_libslic3r::libslic3r::sqr(len)
}

/// `namespace int128 { int orient(...); int cross(...); }` (`Point.hpp:495-502`,
/// `Point.cpp:253-267`): the two exact integer orientation predicates, both forwarding to
/// [`slic3r_int128::int128::Int128::sign_determinant_2x2_filtered`] (`slic3r-int128`, task
/// `53-foundation-int128-queue`'s own unit) -- `+1` CCW, `0` collinear, `-1` CW.
pub mod int128 {
    use super::Vec2crd;

    /// `int orient(const Vec2crd &p1, const Vec2crd &p2, const Vec2crd &p3)`
    /// (`Point.cpp:255-260`): the exact orientation of `p3` relative to the directed line `p1 ->
    /// p2`, via the two edge vectors `p2 - p1` and `p3 - p1`.
    pub fn orient(p1: Vec2crd, p2: Vec2crd, p3: Vec2crd) -> i32 {
        let v1 = p2 - p1;
        let v2 = p3 - p1;
        slic3r_int128::int128::Int128::sign_determinant_2x2_filtered(v1.x(), v1.y(), v2.x(), v2.y())
    }

    /// `int cross(const Vec2crd &v1, const Vec2crd &v2)` (`Point.cpp:262-265`): the exact sign of
    /// the 2D cross product of `v1` and `v2` directly (no third point to form edge vectors from).
    pub fn cross(v1: Vec2crd, v2: Vec2crd) -> i32 {
        slic3r_int128::int128::Int128::sign_determinant_2x2_filtered(v1.x(), v1.y(), v2.x(), v2.y())
    }
}

/// `struct PointHash { size_t operator()(const Vec2crd &pt) const noexcept { ... } };`
/// (`:505-509`) is not ported as a standalone item -- see the module doc. [`ClosestPointInRadiusLookup`]
/// below keys its own lookup table by `(i64, i64)` tuples instead.
///
/// `template<typename ValueType, typename PointAccessor> class ClosestPointInRadiusLookup`
/// (`:515-646`): a 2D spatial hash for finding the closest stored value within a fixed search
/// radius of a query point, bucketed onto a power-of-two grid (`m_grid_log2`) sized from the
/// radius.
///
/// **Keyed by `std::collections::HashMap`, not this port's one insertion-ordered spelling --
/// but the per-key group order inside it is still an order choice, marked `// FIDELITY(port):`
/// below, not the container spelling itself.** The container choice is checked against
/// `port/reports/order-containers.md`, the per-site authority for which existing sites need the
/// insertion-ordered spelling, and it carries no row for this class (checked: `grep -n
/// 'Point.hpp' port/reports/order-containers.md port/reports/order-comparators.md` finds neither
/// this file nor any of its lines) -- `HashMap`, not `IndexMap`, is therefore this site's
/// faithful Rust shape. What *is* an order choice is the per-key `Vec<V>`'s insertion order,
/// which [`ClosestPointInRadiusLookup::find`]'s strict `d2 < dist_min` tie-break and
/// [`ClosestPointInRadiusLookup::find_all`]'s result order both observe -- `task.md`'s own rule
/// names exactly this ("an insertion-ordered map whose iteration order reaches output"), which
/// is why the marker below is `// FIDELITY(port):`, the one that lands this site in
/// `port/reports/divergences.md`, not `// PORT NOTE:`.
// FIDELITY(port): src/libslic3r/Point.hpp:594-640 -- `std::unordered_multimap<Vec2crd, ValueType,
// PointHash>` ported as `std::collections::HashMap<(i64, i64), Vec<ValueType>>` rather than this
// port's one insertion-ordered spelling; see this struct's own doc comment for why no audited
// call site requires the insertion-ordered form yet.
pub struct ClosestPointInRadiusLookup<V, A> {
    point_accessor: A,
    map: HashMap<(i64, i64), Vec<V>>,
    search_radius: i64,
    grid_resolution: i64,
    grid_log2: i64,
}

impl<V, A> ClosestPointInRadiusLookup<V, A>
where
    A: Fn(&V) -> Option<Vec2crd>,
{
    /// `ClosestPointInRadiusLookup(coord_t search_radius, PointAccessor point_accessor =
    /// PointAccessor())` (`:518-548`): derives `m_grid_resolution`/`m_grid_log2` from the search
    /// radius via the same iterated-shift bit trick the vendored constructor uses, rather than
    /// `ilog2`/`next_power_of_two` -- kept literal because the vendored sequence's early-exit
    /// shape (checking `> 32767`, then `> 127`, then `> 7`, then `> 1`) is itself a correctness
    /// precondition this port is held to exactly, not an optimisation free to rewrite.
    pub fn new(search_radius: i64, point_accessor: A) -> Self {
        // `coord_t gridres = 2 * m_search_radius + 4;` (`:522`).
        // PORT SRC: src/libslic3r/Point.hpp:522
        let gridres = 2 * search_radius + 4;
        let mut grid_resolution = gridres;
        let mut grid_log2: i64 = 0;
        // PORT SRC: src/libslic3r/Point.hpp:527
        if grid_resolution > 32767 {
            // PORT SRC: src/libslic3r/Point.hpp:528
            grid_resolution >>= 16;
            // PORT SRC: src/libslic3r/Point.hpp:529
            grid_log2 += 16;
        }
        // PORT SRC: src/libslic3r/Point.hpp:531
        if grid_resolution > 127 {
            // PORT SRC: src/libslic3r/Point.hpp:532
            grid_resolution >>= 8;
            // PORT SRC: src/libslic3r/Point.hpp:533
            grid_log2 += 8;
        }
        // PORT SRC: src/libslic3r/Point.hpp:535
        if grid_resolution > 7 {
            // PORT SRC: src/libslic3r/Point.hpp:536
            grid_resolution >>= 4;
            // PORT SRC: src/libslic3r/Point.hpp:537
            grid_log2 += 4;
        }
        // PORT SRC: src/libslic3r/Point.hpp:539
        if grid_resolution > 1 {
            grid_resolution >>= 2;
            grid_log2 += 2;
        }
        if grid_resolution > 0 {
            grid_log2 += 1;
        }
        grid_resolution = 1 << grid_log2;
        ClosestPointInRadiusLookup {
            point_accessor,
            map: HashMap::new(),
            search_radius,
            grid_resolution,
            grid_log2,
        }
    }

    fn cell_of(&self, pt: Vec2crd) -> (i64, i64) {
        (pt.x() >> self.grid_log2, pt.y() >> self.grid_log2)
    }

    /// `void insert(const ValueType &value)` / `void insert(ValueType &&value)` (`:550-560`):
    /// collapse to one owned-`value` method, since Rust's move semantics already make passing
    /// `value` by value the "no extra copy" form both vendored overloads exist to provide.
    pub fn insert(&mut self, value: V) {
        if let Some(pt) = (self.point_accessor)(&value) {
            let cell = self.cell_of(pt);
            self.map.entry(cell).or_default().push(value);
        }
    }

    /// `bool erase(const ValueType &value)` (`:562-577`): removes the first stored value equal
    /// to `value`, returning whether one was found -- the vendored doc comment states
    /// `ValueType` "has to declare the `operator==`", which this port expresses as the
    /// `V: PartialEq` bound below rather than a runtime check.
    pub fn erase(&mut self, value: &V) -> bool
    where
        V: PartialEq,
    {
        if let Some(pt) = (self.point_accessor)(value) {
            let cell = self.cell_of(pt);
            if let Some(bucket) = self.map.get_mut(&cell) {
                if let Some(pos) = bucket.iter().position(|v| v == value) {
                    bucket.remove(pos);
                    return true;
                }
            }
        }
        false
    }

    /// `std::pair<const ValueType*, double> find(const Vec2crd &pt)` (`:580-610`): the stored
    /// value closest to `pt` within the search radius, or `None` -- scans the four grid cells
    /// nearest `pt`'s rounded grid corner, in the same `neighbor_y`/`neighbor_x` order the
    /// vendored loop uses, and keeps the first value seen with a strictly smaller squared
    /// distance than the running minimum (ties keep the earlier one, matching `d2 < dist_min`'s
    /// strict comparison upstream).
    pub fn find(&self, pt: Vec2crd) -> Option<(&V, f64)> {
        let mut value_min: Option<&V> = None;
        let mut dist_min = f64::MAX;
        // PORT SRC: src/libslic3r/Point.hpp:587
        let half = self.grid_resolution >> 1;
        let grid_corner = (
            (pt.x() + half) >> self.grid_log2,
            (pt.y() + half) >> self.grid_log2,
        );
        for neighbor_y in -1..1 {
            for neighbor_x in -1..1 {
                let cell = (grid_corner.0 + neighbor_x, grid_corner.1 + neighbor_y);
                if let Some(bucket) = self.map.get(&cell) {
                    for value in bucket {
                        if let Some(pt2) = (self.point_accessor)(value) {
                            let dx = (pt.x() - pt2.x()) as f64;
                            let dy = (pt.y() - pt2.y()) as f64;
                            let d2 = dx * dx + dy * dy;
                            if d2 < dist_min {
                                dist_min = d2;
                                value_min = Some(value);
                            }
                        }
                    }
                }
            }
        }
        let radius = self.search_radius as f64;
        if value_min.is_some() && dist_min < radius * radius {
            value_min.map(|v| (v, dist_min))
        } else {
            None
        }
    }

    /// `std::vector<std::pair<const ValueType*, double>> find_all(const Vec2crd &pt)`
    /// (`:613-637`): every stored value within the search radius of `pt`, each with its squared
    /// distance -- the non-strict `d2 <= r2` upstream (`:630`), unlike [`Self::find`]'s strict
    /// `<`.
    pub fn find_all(&self, pt: Vec2crd) -> Vec<(&V, f64)> {
        let half = self.grid_resolution >> 1;
        let grid_corner = (
            (pt.x() + half) >> self.grid_log2,
            (pt.y() + half) >> self.grid_log2,
        );
        let r2 = (self.search_radius as f64) * (self.search_radius as f64);
        let mut out = Vec::new();
        for neighbor_y in -1..1 {
            for neighbor_x in -1..1 {
                let cell = (grid_corner.0 + neighbor_x, grid_corner.1 + neighbor_y);
                if let Some(bucket) = self.map.get(&cell) {
                    for value in bucket {
                        if let Some(pt2) = (self.point_accessor)(value) {
                            let dx = (pt.x() - pt2.x()) as f64;
                            let dy = (pt.y() - pt2.y()) as f64;
                            let d2 = dx * dx + dy * dy;
                            // PORT SRC: src/libslic3r/Point.hpp:630
                            if d2 <= r2 {
                                out.push((value, d2));
                            }
                        }
                    }
                }
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use slic3r_proof::{compare_unit, Oracle};

    #[test]
    fn point_add_sub_match_component_wise() {
        let a = Point::new(3, 4);
        let b = Point::new(1, 2);
        assert_eq!(a + b, Point::new(4, 6));
        assert_eq!(a - b, Point::new(2, 2));
    }

    #[test]
    fn point_both_and_any_comp() {
        let a = Point::new(5, 5);
        let b = Point::new(1, 1);
        assert!(a.both_greater(&b));
        assert!(b.both_less(&a));
        assert!(!a.both_greater(&Point::new(1, 10)));
        assert!(a.any_greater(&Point::new(1, 10)));
    }

    #[test]
    fn rotate_90_degree_ccw_matches_vendored_formula() {
        let p = Point::new(3, 4);
        assert_eq!(p.rotate_90_degree_ccw(), Point::new(-4, 3));
    }

    #[test]
    fn ccw_is_positive_for_a_counter_clockwise_turn() {
        let origin = Point::new(0, 0);
        let p1 = Point::new(10, 0);
        let p2 = Point::new(0, 10);
        assert!(origin.ccw(p1, p2) > 0.0);
        assert!(origin.ccw(p2, p1) < 0.0);
    }

    #[test]
    fn has_duplicate_points_detects_a_repeat() {
        let pts = vec![Point::new(0, 0), Point::new(1, 1), Point::new(0, 0)];
        assert!(has_duplicate_points(pts));
        assert!(!has_duplicate_points(vec![
            Point::new(0, 0),
            Point::new(1, 1)
        ]));
    }

    #[test]
    fn collect_duplicates_reports_each_repeat_once() {
        let pts = vec![
            Point::new(0, 0),
            Point::new(1, 1),
            Point::new(0, 0),
            Point::new(0, 0),
            Point::new(2, 2),
        ];
        assert_eq!(collect_duplicates(pts), vec![Point::new(0, 0)]);
    }

    #[test]
    fn align_to_grid_rounds_toward_negative_infinity() {
        assert_eq!(align_to_grid(7, 5), 5);
        assert_eq!(align_to_grid(-7, 5), -10);
        assert_eq!(align_to_grid(-10, 5), -10);
    }

    #[test]
    fn scaled_coord_truncates_toward_zero_with_no_rounding() {
        // 1.9999999 scaled units is just under 2, and truncation (not rounding) must floor it
        // toward zero rather than round it up to 2.
        let v = 1.9999999 * slic3r_libslic3r::libslic3r::SCALING_FACTOR;
        assert_eq!(scaled_coord(v), 1);
        let neg = -1.9999999 * slic3r_libslic3r::libslic3r::SCALING_FACTOR;
        assert_eq!(scaled_coord(neg), -1);
    }

    #[test]
    fn unscaled_never_truncates() {
        assert_eq!(
            unscaled_f64(3),
            3.0 * slic3r_libslic3r::libslic3r::SCALING_FACTOR
        );
    }

    #[test]
    fn closest_point_in_radius_lookup_finds_the_nearer_of_two_points() {
        let mut lookup: ClosestPointInRadiusLookup<Point, _> =
            ClosestPointInRadiusLookup::new(1000, |p: &Point| Some(Vec2crd::new(p.x(), p.y())));
        lookup.insert(Point::new(0, 0));
        lookup.insert(Point::new(500, 0));
        let (closest, _dist) = lookup
            .find(Vec2crd::new(10, 0))
            .expect("one point in range");
        assert_eq!(*closest, Point::new(0, 0));
    }

    /// Proof path 1 (R19): `Slic3r::int128::orient`/`Slic3r::int128::cross`
    /// (`Point.cpp:253-267`) are externally linked, non-template free functions -- genuine
    /// `Point.cpp` code (not merely forwarding into `Int128.cpp`, which `slic3r-int128`'s own
    /// proof already covers) -- linked from the archive `just oracle-build` produces, called here
    /// through `crates/slic3r-ffi/cxx/proof_units/point.cpp`'s wrappers, reached through
    /// `slic3r-proof`'s re-export of the FFI crate's per-unit proof bridges (`docs/PORTING.md`,
    /// "Decisions recorded on 2026-11-25").
    // PROOF: 1
    #[test]
    fn point_int128_orient_matches_the_oracle() {
        let p1 = Vec2crd::new(0, 0);
        let p2 = Vec2crd::new(100, 0);
        let p3 = Vec2crd::new(50, 100);
        let actual = int128::orient(p1, p2, p3);
        let oracle = slic3r_proof::proof_units::point::path1_int128_orient(
            p1.x(),
            p1.y(),
            p2.x(),
            p2.y(),
            p3.x(),
            p3.y(),
        );
        compare_unit("Point.cpp", Oracle::Built, actual, oracle)
            .expect("int128::orient must match the oracle's exact-integer orientation predicate");
    }

    /// Proof path 2 (R19): `Point::rotate(double, const Point&)` is `Point.hpp`'s own declared
    /// method (the vendored class, not a free function), proven against the committed oracle
    /// build via `cxx/proof_units/point.cpp`'s direct call through a real `Slic3r::Point`,
    /// exactly as `execution.rs`'s own precedent generalises decision 1 to a `proof path: 2`
    /// row with no stage dump to read -- `Point.hpp` itself produces coordinates that *do* reach
    /// `posSlice.dump`, but the literal `rotate(angle, center)` call this proof drives is not one
    /// of that dump's own named fields, so this proof reaches the oracle through the per-unit
    /// shim bridge instead, the same route path 1 above uses, naming oracle `built`.
    // PROOF: 2
    #[test]
    fn point_rotate_about_matches_the_oracle() {
        let p = Point::new(1000, 0);
        let center = Point::new(0, 0);
        let angle = std::f64::consts::PI / 2.0;
        let actual = p.rotated_about(angle, center);
        let oracle = slic3r_proof::proof_units::point::path2_point_rotate(
            p.x(),
            p.y(),
            center.x(),
            center.y(),
            angle,
        );
        compare_unit(
            "Point.hpp",
            Oracle::Built,
            vec![actual.x(), actual.y()],
            oracle,
        )
        .expect(
            "Point::rotate(angle, center) must match the oracle's own fast_round_up-based rotation",
        );
    }

    /// Proof path 2 (R19), second marker for the `Point.hpp` row. **Finding from adversarial
    /// review**: the proof above drives only `Point::rotate`, which `Point.cpp` defines --
    /// `Point.hpp`'s own header-only surface (the scaling entry points this task exists to
    /// get right) was going unproven. This test drives `Slic3r::scaled<coord_t>()`'s integral
    /// overload at both input precisions and `Slic3r::unscaled<double>()` directly, through
    /// `cxx/proof_units/point.cpp`'s three shims (the same "instantiate the real header template
    /// in the shim TU" technique `mt_utils.cpp` uses for `Slic3r::grid<double>`).
    // PROOF: 2
    #[test]
    fn point_scaled_unscaled_match_the_oracle() {
        let v = 1234.5678_f64;
        let actual_f64 = scaled_coord(v);
        let oracle_f64 = slic3r_proof::proof_units::point::path2_scaled_coord_f64(v);
        assert_eq!(actual_f64, oracle_f64);

        let vf = 1234.5_f32;
        let actual_f32 = scaled_coord_f32(vf);
        let oracle_f32 = slic3r_proof::proof_units::point::path2_scaled_coord_f32(vf);
        assert_eq!(actual_f32, oracle_f32);

        let coord = 123_456_789_i64;
        let actual_unscaled = unscaled_f64(coord);
        let oracle_unscaled = slic3r_proof::proof_units::point::path2_unscaled_f64(coord);
        compare_unit("Point.hpp", Oracle::Built, actual_unscaled, oracle_unscaled).expect(
            "scaled<coord_t>()/unscaled<double>() must match the oracle's own header-only \
             templates exactly",
        );
    }
}

// PORT STATUS
// source: src/libslic3r/Point.cpp
// source: src/libslic3r/Point.hpp
// confidence: medium
// tier: exact
// oracle: built
