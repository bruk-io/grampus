//! Port of `src/libslic3r/Geometry/Circle.cpp`/`Circle.hpp` (task `76-foundation-voronoi-curves`,
//! plan step 20, `port/foundation.tsv`'s `foundation-25` row). `Circle.hpp` is
//! `template-heavy: yes`; `Circle.cpp` is `template-heavy: no` but hand-designed alongside it in
//! this same task because the two are one unit.
//!
//! **Scope, measured against the whole of `vendor/OrcaSlicer/` -- `src/`, `tests/`,
//! `sandboxes/` and `tools/`, the only directories that could plausibly name a libslic3r
//! symbol:**
//!
//! - `circle_center<Vector>` (`:17-36`), `CircleSq<Vector>`/`Circle<Vector>` (`:40-93`) and
//!   `smallest_enclosing_circle2_welzl`/`smallest_enclosing_circle_welzl` (`:111-154`) are ported
//!   below, monomorphised at `Vector = Vec2d` (`Circled`) for the algorithmic machinery -- the
//!   only instantiation any in-scope caller reaches (`BuildVolume.cpp:79`,
//!   `src/slic3r/GUI/Selection.cpp:1559`, GUI, R10) -- plus `Vector = Vec2f` (`Circlef`) for
//!   [`Circle`]'s struct shape alone, which `BuildVolume.cpp:412,467` constructs directly with
//!   the two-argument `Circle(center, radius)` constructor (never via `from_two_points`,
//!   `from_three_points` or the `CircleSq` conversion, which only `Vec2d` call sites reach) --
//!   `docs/PORTING.md`'s idiom-map "a template is not force-collapsed to one concrete type
//!   unless the manifest row's unit only ever instantiates one" licenses porting a true Rust
//!   generic here, since more than one instantiation is actually used.
//! - `ray_circle_intersections_r2_lv2_c<T>` (`:161-175`) is ported at `T = f64`, the only
//!   instantiation any in-scope caller reaches (`src/libslic3r/Fill/FillBase.cpp:928`, manifest
//!   `in`, crate `slic3r-fill`, not-started at the pin).
//! - `circle_center_taubin_newton` (both overloads) and `circle_taubin_newton` are ported below
//!   (`Circle.cpp:19-117`), `Circle.cpp`'s own externally-linked row.
//!
//! **Left unported, each a named finding for the pull request rather than a tolerance or a
//! guess:**
//!
//! - **`ray_circle_intersections` (`Circle.hpp:176-186`), the 4-argument convenience wrapper.**
//!   Its body calls `ray_circle_intersections_r2_lv2_c2(...)` (`:185`) -- a name with no
//!   declaration anywhere in the vendored tree (the real primitive is spelled
//!   `ray_circle_intersections_r2_lv2_c`, a stray trailing `2` after `_c`); this is a latent
//!   bug in the vendored header. It is never instantiated anywhere in the tree --
//!   `FillBase.cpp:928` calls `ray_circle_intersections_r2_lv2_c` directly, never this
//!   wrapper -- so, like
//!   `intersect_triangle_epsilon` in `docs/PORTING.md`'s "Decisions recorded on 2026-11-26",
//!   decision 9, it has no caller anywhere in the vendored tree and does not compile when
//!   instantiated. Left unported for exactly that reason.
//! - **`circle_ransac` (`Circle.cpp:119-146`).** It is live (`BuildVolume.cpp:50,142`,
//!   `src/libslic3r/Measure.cpp:79` -- the latter itself out of scope, manifest `out: No caller
//!   in the four entry points' closure`), but its body draws from `std::mt19937 rng;`
//!   (default-seeded) through `std::sample(input.begin(), input.end(), ..., 3, rng)`
//!   (`:130`). `std::mt19937`'s state transition and output are fixed bit-for-bit by the C++
//!   standard ([rand.eng.mers]), but `std::sample`'s algorithm and its calls into
//!   `std::uniform_int_distribution` are **not** standardised to one bit-exact shape across
//!   standard-library implementations -- matching this row's `exact` tier with no tolerance cell
//!   would mean reverse-engineering the specific libc++ build `just oracle-build` links against,
//!   which is a different, much larger undertaking than hand-designing this unit's own shape.
//!   `docs/WORKFLOW.md`'s own rule: "A unit that cannot be held to exact equality is a finding
//!   for the pull request, not a tolerance invented on the branch." Left as `// TODO(port):`.

use slic3r_point::point::{Point, Vec2, Vec2d, Vec2f};

// -------------------------------------------------------------------------------------------
// Small shared helpers (not in the vendored source; this unit's own arithmetic shorthand)
// -------------------------------------------------------------------------------------------

fn squared_norm(v: Vec2d) -> f64 {
    v.x() * v.x() + v.y() * v.y()
}

/// A plain numeric cast, `Point.template cast<double>()` (no `SCALING_FACTOR` division) --
/// `smallest_enclosing_circle2_welzl`'s own `points[i].template cast<Scalar>()` (`Circle.hpp:118,
/// 125, 129, 133`).
fn to_vec2d(p: Point) -> Vec2d {
    Vec2::new(p.x() as f64, p.y() as f64)
}

// -------------------------------------------------------------------------------------------
// circle_center (Circle.hpp:17-36), monomorphised at Vec2d (see the module doc)
// -------------------------------------------------------------------------------------------

/// `Vector circle_center(const Vector&, const Vector&, const Vector&, Scalar)`
/// (`Circle.hpp:17-36`). Collinear input returns the midpoint of the two points furthest apart
/// from each other (the vendored comment at `:26-27`), not an arbitrary tie-break.
pub fn circle_center(a: Vec2d, bsrc: Vec2d, csrc: Vec2d, epsilon: f64) -> Vec2d {
    let b = bsrc - a;
    let c = csrc - a;
    let lb = squared_norm(b);
    let lc = squared_norm(c);
    // `Scalar d = b.x() * c.y() - b.y() * c.x();` (`:25`).
    let d = b.x() * c.y() - b.y() * c.x();
    if d.abs() < epsilon {
        let lbc = squared_norm(csrc - bsrc);
        let sum = if lb > lc && lb > lbc {
            a + bsrc
        } else if lc > lb && lc > lbc {
            a + csrc
        } else {
            bsrc + csrc
        };
        // `return Scalar(0.5) * (...)` (`:29-31`).
        // PORT SRC: src/libslic3r/Geometry/Circle.hpp:29
        Vec2::new(0.5 * sum.x(), 0.5 * sum.y())
    } else {
        // `Vector v = lc * b - lb * c;` (`:33`).
        let v = Vec2::new(lc * b.x() - lb * c.x(), lc * b.y() - lb * c.y());
        // `return a + Vector(-v.y(), v.x()) / (2 * d);` (`:34`) -- component-wise division by
        // `2 * d`, never a multiply by its reciprocal (`docs/PORTING.md`, banned-substitution
        // list). `2f64` (no decimal point) so the exempt value `2` normalises into R16's exempt
        // set.
        let two_d = 2f64 * d;
        a + Vec2::new(-v.y() / two_d, v.x() / two_d)
    }
}

// -------------------------------------------------------------------------------------------
// CircleSq<Vec2d> (Circle.hpp:39-63), monomorphised at Vec2d (see the module doc)
// -------------------------------------------------------------------------------------------

/// `CircleSq<Vector>` (`Circle.hpp:40-63`), monomorphised at `Vec2d` -- the only instantiation
/// used anywhere in-scope (internally, by [`smallest_enclosing_circle2_welzl`] below).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CircleSq {
    pub center: Vec2d,
    pub radius2: f64,
}

impl CircleSq {
    pub fn new(center: Vec2d, radius2: f64) -> Self {
        CircleSq { center, radius2 }
    }

    /// `CircleSq(const Vector &a, const Vector &b)` (`:48`).
    pub fn from_two_points(a: Vec2d, b: Vec2d) -> Self {
        // PORT SRC: src/libslic3r/Geometry/Circle.hpp:48
        let center = Vec2::new(0.5 * (a.x() + b.x()), 0.5 * (a.y() + b.y()));
        let d = a - center;
        CircleSq {
            center,
            radius2: squared_norm(d),
        }
    }

    /// `CircleSq(const Vector &a, const Vector &b, const Vector &c, Scalar epsilon)` (`:49-52`).
    pub fn from_three_points(a: Vec2d, b: Vec2d, c: Vec2d, epsilon: f64) -> Self {
        let center = circle_center(a, b, c, epsilon);
        let d = a - center;
        CircleSq {
            center,
            radius2: squared_norm(d),
        }
    }

    /// `bool invalid() const` (`:54`).
    pub fn invalid(&self) -> bool {
        self.radius2 < 0f64
    }

    /// `bool valid() const` (`:55`).
    pub fn valid(&self) -> bool {
        !self.invalid()
    }

    /// `bool contains(const Vector&) const` (`:56`).
    pub fn contains(&self, p: Vec2d) -> bool {
        squared_norm(p - self.center) < self.radius2
    }

    /// `bool contains(const Vector&, Scalar epsilon2) const` (`:57`).
    pub fn contains_eps(&self, p: Vec2d, epsilon2: f64) -> bool {
        squared_norm(p - self.center) < self.radius2 + epsilon2
    }

    /// `CircleSq inflated(Scalar epsilon) const` (`:59-60`).
    pub fn inflated(&self, epsilon: f64) -> Self {
        debug_assert!(self.radius2 >= 0f64);
        let r = self.radius2.sqrt() + epsilon;
        CircleSq {
            center: self.center,
            radius2: r * r,
        }
    }

    /// `static CircleSq make_invalid()` (`:62`): `CircleSq { { 0, 0 }, -1 }`, spelled with no
    /// decimal point so the two exempt values `0`/`1` normalise into R16's exempt set rather
    /// than needing a marker.
    pub fn make_invalid() -> Self {
        CircleSq {
            center: Vec2::new(0f64, 0f64),
            radius2: -1f64,
        }
    }
}

// -------------------------------------------------------------------------------------------
// Circle<Vector> (Circle.hpp:66-93): Vec2d (Circled) and Vec2f (Circlef) -- see the module doc.
// -------------------------------------------------------------------------------------------

/// `Circle<Vector>` (`Circle.hpp:67-93`). The two-argument constructor, `invalid`/`valid`,
/// `contains`/`contains(_, epsilon)` and `inflated` are generic (both `Circled`/`Circlef`
/// instantiate them in-scope); `from_two_points` and the `CircleSq` conversion
/// (`from_circle_sq`) are `Vec2d`-only inherent methods below them, because no in-scope caller
/// ever reaches those two at `Vec2f` (see the module doc). The three-point constructor
/// (`Circle.hpp:76`, `Circle(const Vector&, const Vector&, const Vector&, const Scalar)`) and
/// its `operator=` from `CircleSq` (`:82`) are unported: `Circle` is only ever constructed with
/// two arguments anywhere in the vendored tree (`BuildVolume.hpp:42,165`,
/// `BuildVolume.cpp:412,467`), so the three/four-argument forms have no caller to preserve
/// behaviour for.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Circle<T> {
    pub center: Vec2<T>,
    pub radius: T,
}

impl<T: Copy> Circle<T> {
    pub fn new(center: Vec2<T>, radius: T) -> Self {
        Circle { center, radius }
    }
}

impl Circle<f64> {
    /// `bool invalid() const` (`:84`).
    pub fn invalid(&self) -> bool {
        self.radius < 0f64
    }
    /// `bool valid() const` (`:85`).
    pub fn valid(&self) -> bool {
        !self.invalid()
    }
    /// `bool contains(const Vector&) const` (`:86`).
    pub fn contains(&self, p: Vec2d) -> bool {
        squared_norm(p - self.center) <= self.radius * self.radius
    }
    /// `bool contains(const Vector&, Scalar) const` (`:87-88`).
    pub fn contains_eps(&self, p: Vec2d, epsilon: f64) -> bool {
        let re = self.radius + epsilon;
        squared_norm(p - self.center) < re * re
    }
    /// `Circle inflated(Scalar) const` (`:90`).
    pub fn inflated(&self, epsilon: f64) -> Self {
        debug_assert!(self.radius >= 0f64);
        Circle {
            center: self.center,
            radius: self.radius + epsilon,
        }
    }
    /// `Circle(const Vector &a, const Vector &b)` (`:75`).
    pub fn from_two_points(a: Vec2d, b: Vec2d) -> Self {
        // PORT SRC: src/libslic3r/Geometry/Circle.hpp:75
        let center = Vec2::new(0.5 * (a.x() + b.x()), 0.5 * (a.y() + b.y()));
        let d = a - center;
        Circle {
            center,
            radius: squared_norm(d).sqrt(),
        }
    }
    /// `explicit Circle(const CircleSq<Vector2>&)` (`:79-80`): `radius2 <= 0` is carried through
    /// unsquared (a negative-or-zero "radius" is the invalid-circle sentinel, not a length), and
    /// is square-rooted only when positive.
    pub fn from_circle_sq(c: CircleSq) -> Self {
        Circle {
            center: c.center,
            radius: if c.radius2 <= 0f64 {
                c.radius2
            } else {
                c.radius2.sqrt()
            },
        }
    }
    /// `static Circle make_invalid()` (`:92`): see [`CircleSq::make_invalid`]'s doc for why `0`/
    /// `-1` are spelled with no decimal point.
    pub fn make_invalid() -> Self {
        Circle {
            center: Vec2::new(0f64, 0f64),
            radius: -1f64,
        }
    }
}

impl Circle<f32> {
    pub fn invalid(&self) -> bool {
        self.radius < 0f32
    }
    pub fn valid(&self) -> bool {
        !self.invalid()
    }
    pub fn contains(&self, p: Vec2f) -> bool {
        let d = p - self.center;
        (d.x() * d.x() + d.y() * d.y()) <= self.radius * self.radius
    }
    pub fn contains_eps(&self, p: Vec2f, epsilon: f32) -> bool {
        let re = self.radius + epsilon;
        let d = p - self.center;
        (d.x() * d.x() + d.y() * d.y()) < re * re
    }
    pub fn inflated(&self, epsilon: f32) -> Self {
        debug_assert!(self.radius >= 0f32);
        Circle {
            center: self.center,
            radius: self.radius + epsilon,
        }
    }
    /// See [`CircleSq::make_invalid`]'s doc for why `0`/`-1` are spelled with no decimal point.
    pub fn make_invalid() -> Self {
        Circle {
            center: Vec2::new(0f32, 0f32),
            radius: -1f32,
        }
    }
}

/// `using Circled = Circle<Vec2d>;` (`Circle.hpp:96`).
pub type Circled = Circle<f64>;
/// `using Circlef = Circle<Vec2f>;` (`:95`).
pub type Circlef = Circle<f32>;

// -------------------------------------------------------------------------------------------
// smallest_enclosing_circle2_welzl / smallest_enclosing_circle_welzl (Circle.hpp:111-154)
// -------------------------------------------------------------------------------------------

/// `CircleSq<Vector> smallest_enclosing_circle2_welzl(const Points&, Scalar)`
/// (`Circle.hpp:111-141`), monomorphised at `Vector = Vec2d`, `Points = &[Point]` -- the only
/// instantiation any in-scope caller reaches (see the module doc).
///
/// `// PORT NOTE:` the vendored `CircleSq<Vector> circle;` (`:115`) is default-constructed
/// (`CircleSq() {}`, no member-initialiser list), so an empty `points` returns it
/// **uninitialized** at the pin; Rust has no uninitialized-struct equivalent, so this
/// zero-initializes instead for that input, which no in-scope caller passes (every one gives a
/// convex hull or an extruder shape with at least one point).
pub fn smallest_enclosing_circle2_welzl(points: &[Point], epsilon: f64) -> CircleSq {
    let mut circle = CircleSq {
        center: Vec2::new(0f64, 0f64),
        radius2: 0f64,
    };
    if points.is_empty() {
        return circle;
    }
    let p0 = to_vec2d(points[0]);
    if points.len() == 1 {
        circle.center = p0;
        circle.radius2 = epsilon * epsilon;
        return circle;
    }
    circle = CircleSq::from_two_points(p0, to_vec2d(points[1])).inflated(epsilon);
    for i in 2..points.len() {
        let p = to_vec2d(points[i]);
        if !circle.contains(p) {
            // p is the first point on the smallest circle enclosing points[0..i]
            circle = CircleSq::from_two_points(p0, p).inflated(epsilon);
            for j in 1..i {
                let q = to_vec2d(points[j]);
                if !circle.contains(q) {
                    // q is the second point on the smallest circle enclosing points[0..i]
                    circle = CircleSq::from_two_points(p, q).inflated(epsilon);
                    for k in 0..j {
                        let r = to_vec2d(points[k]);
                        if !circle.contains(r) {
                            circle =
                                CircleSq::from_three_points(p, q, r, epsilon).inflated(epsilon);
                        }
                    }
                }
            }
        }
    }
    circle
}

/// `Circle<Vector> smallest_enclosing_circle_welzl(const Points&, Scalar)` (`Circle.hpp:144-148`),
/// at the same `Vec2d` instantiation.
pub fn smallest_enclosing_circle_welzl_with_epsilon(points: &[Point], epsilon: f64) -> Circled {
    Circle::from_circle_sq(smallest_enclosing_circle2_welzl(points, epsilon))
}

/// `inline Circled smallest_enclosing_circle_welzl(const Points&)` (`:151-154`): the real
/// in-scope entry point, `BuildVolume.cpp:79`.
pub fn smallest_enclosing_circle_welzl(points: &[Point]) -> Circled {
    smallest_enclosing_circle_welzl_with_epsilon(
        points,
        slic3r_libslic3r::libslic3r::scaled_epsilon(),
    )
}

// -------------------------------------------------------------------------------------------
// ray_circle_intersections_r2_lv2_c (Circle.hpp:161-175), monomorphised at T = f64
// -------------------------------------------------------------------------------------------

/// `int ray_circle_intersections_r2_lv2_c(T, T, T, T, T, std::pair<Vec,Vec>&)`
/// (`Circle.hpp:161-175`), at `T = f64` -- the only instantiation reached in-scope
/// (`src/libslic3r/Fill/FillBase.cpp:928`). The non-const-reference out-parameter becomes a
/// return value (`docs/PORTING.md` idiom map): `(count, [first, second])`, `first`/`second`
/// meaningful only while `count > 0` -- the vendored pair is left *untouched* (whatever the
/// caller's own storage already held) when `d2 < 0` and the function returns `0` (`:167-168`);
/// the two zero `Vec2d`s this port returns in that case are this port's own fill, chosen
/// because Rust has no uninitialised-out-parameter equivalent, and are not a value the oracle
/// ever produced, so no caller may read them as meaningful.
pub fn ray_circle_intersections_r2_lv2_c(
    r2: f64,
    a: f64,
    b: f64,
    lv2: f64,
    c: f64,
) -> (i32, [Vec2d; 2]) {
    let x0 = -a * c;
    let y0 = -b * c;
    let d2 = r2 * lv2 - c * c;
    if d2 < 0f64 {
        return (0, [Vec2::new(0f64, 0f64), Vec2::new(0f64, 0f64)]);
    }
    let d = d2.sqrt();
    let first = Vec2::new((x0 + b * d) / lv2, (y0 - a * d) / lv2);
    let second = Vec2::new((x0 - b * d) / lv2, (y0 + a * d) / lv2);
    (if d == 0f64 { 1 } else { 2 }, [first, second])
}

// TODO(port): src/libslic3r/Geometry/Circle.hpp:176-186 -- `ray_circle_intersections`, the
// 4-argument convenience wrapper; see the module doc for why it is left unported (calls an
// undefined name, never instantiated anywhere in the vendored tree).

// -------------------------------------------------------------------------------------------
// circle_center_taubin_newton / circle_taubin_newton (Circle.cpp:19-117)
// -------------------------------------------------------------------------------------------

/// `Vec2d circle_center_taubin_newton(Vec2ds::const_iterator, Vec2ds::const_iterator, size_t)`
/// (`Circle.cpp:31-104`). Adapted from "Circular and Linear Regression: Fitting circles and
/// lines by least squares", pg 126.
pub fn circle_center_taubin_newton_vec2d(input: &[Vec2d], cycles: usize) -> Vec2d {
    let n = input.len();
    let n_flt = n as f64;
    let mut sum = Vec2::new(0f64, 0f64);
    for &p in input {
        sum = sum + p;
    }
    // `const Vec2d centroid { sum / n_flt };` (`:37`) -- component-wise division, never a
    // multiply by `1.0 / n_flt`.
    let centroid = Vec2::new(sum.x() / n_flt, sum.y() / n_flt);

    let mut mxx = 0f64;
    let mut myy = 0f64;
    let mut mxy = 0f64;
    let mut mxz = 0f64;
    let mut myz = 0f64;
    let mut mzz = 0f64;
    for &p in input {
        let xi = p.x() - centroid.x();
        let yi = p.y() - centroid.y();
        let zi = xi * xi + yi * yi;
        mxy += xi * yi;
        mxx += xi * xi;
        myy += yi * yi;
        mxz += xi * zi;
        myz += yi * zi;
        mzz += zi * zi;
    }
    mxx /= n_flt;
    myy /= n_flt;
    mxy /= n_flt;
    mxz /= n_flt;
    myz /= n_flt;
    mzz /= n_flt;

    let mz = mxx + myy;
    let cov_xy = mxx * myy - mxy * mxy;
    // PORT SRC: src/libslic3r/Geometry/Circle.cpp:66
    let c3 = 4.0 * mz;
    // PORT SRC: src/libslic3r/Geometry/Circle.cpp:67
    let c2 = -3.0 * (mz * mz) - mzz;
    // PORT SRC: src/libslic3r/Geometry/Circle.cpp:68
    let c1 = mz * (mzz - (mz * mz)) + 4.0 * mz * cov_xy - (mxz * mxz) - (myz * myz);
    // PORT SRC: src/libslic3r/Geometry/Circle.cpp:69
    let two_mxz_myz_mxy = 2.0 * mxz * myz * mxy;
    let c0 = (mxz * mxz) * myy + (myz * myz) * mxx - two_mxz_myz_mxy - cov_xy * (mzz - (mz * mz));

    let c22 = c2 + c2;
    let c33 = c3 + c3 + c3;

    // `double xnew = 0.0;` (`:75`) -- decimal-spelled in the vendored line, so kept as `0.0`
    // (not the shorter `0.`, whose normalised text is a prefix of the vendored line's `0.0`
    // and so fails the marker's own non-digit/non-dot boundary check) rather than the
    // exempt-suffix form (`docs/PORTING.md`, "Decisions recorded on 2026-11-26", decision 8).
    // PORT SRC: src/libslic3r/Geometry/Circle.cpp:75
    let mut xnew = 0.0;
    // PORT SRC: src/libslic3r/Geometry/Circle.cpp:76
    let mut ynew = 1e20f64;

    for _ in 0..cycles {
        let yold = ynew;
        ynew = c0 + xnew * (c1 + xnew * (c2 + xnew * c3));
        if ynew.abs() > yold.abs() {
            // `BOOST_LOG_TRIVIAL(error) << "...";` (`:82`) -- a log side effect, not ported.
            return Vec2::new(f64::NAN, f64::NAN);
        }
        let dy = c1 + xnew * (c22 + xnew * c33);

        let xold = xnew;
        xnew = xold - (ynew / dy);

        // `if (std::abs((xnew-xold) / xnew) < 1e-12) i = cycles;` (`:90`) -- setting the loop
        // variable to `cycles` does not skip the rest of this iteration's body: the
        // "reset if negative" check two lines below still runs on the converging iteration
        // before the `for` loop's own condition ends it. An earlier draft of this function
        // used a bare `break` at this point, which would skip that check whenever convergence
        // and `xnew < 0` land on the same iteration -- found on review against the vendored
        // source, not by a failing test (no fixture in this module's own proof or fuzz target
        // happens to hit that exact combination), and fixed here before it could. The reset
        // below always runs first; only then does this flag the loop to stop.
        // PORT SRC: src/libslic3r/Geometry/Circle.cpp:90
        let converged = ((xnew - xold) / xnew).abs() < 1e-12;

        // `if (xnew < 0)` (`:92`) -- bare `0`, exempt as-is.
        if xnew < 0f64 {
            // `xnew = 0.0;` (`:94`) -- decimal-spelled, unlike the comparison above; kept as
            // `0.0` plus its own marker (see the `xnew` initialiser's identical note above).
            // PORT SRC: src/libslic3r/Geometry/Circle.cpp:94
            xnew = 0.0;
        }

        if converged {
            break;
        }
    }

    let det = xnew * xnew - xnew * mz + cov_xy;
    let unscaled_center = Vec2::new(
        mxz * (myy - xnew) - myz * mxy,
        myz * (mxx - xnew) - mxz * mxy,
    );
    // `center /= (DET * 2.);` (`:102`) -- component-wise division by `DET * 2.`. The vendored
    // line spells this `2.` (a decimal), not a bare integer, so it keeps that spelling plus a
    // marker rather than the exempt-suffix form (`docs/PORTING.md`, "Decisions recorded on
    // 2026-11-26", decision 8; contrast `circle_center`'s `2f64 * d` above, whose own cited line,
    // `Circle.hpp:34`, spells its `2` bare).
    // PORT SRC: src/libslic3r/Geometry/Circle.cpp:102
    let two_det = det * 2.;
    let center = Vec2::new(unscaled_center.x() / two_det, unscaled_center.y() / two_det);
    center + centroid
}

/// `Point circle_center_taubin_newton(Points::const_iterator, Points::const_iterator, size_t)`
/// (`Circle.cpp:19-26`): unscales to real-world (mm) coordinates, fits in that space, then
/// re-scales the result -- `unscale(in)` (the multiplicative family), never a plain numeric
/// cast, unlike [`smallest_enclosing_circle2_welzl`]'s `points[i].template cast<Scalar>()`.
pub fn circle_center_taubin_newton(input: &[Point], cycles: usize) -> Point {
    let tmp: Vec<Vec2d> = input
        .iter()
        .map(|&p| slic3r_point::point::unscale2(p.x(), p.y()))
        .collect();
    let center = circle_center_taubin_newton_vec2d(&tmp, cycles);
    Point::new_scale(center.x(), center.y())
}

/// `Circled circle_taubin_newton(const Vec2ds&, size_t)` (`Circle.cpp:106-117`).
pub fn circle_taubin_newton(input: &[Vec2d], cycles: usize) -> Circled {
    // PORT SRC: src/libslic3r/Geometry/Circle.cpp:109
    if input.len() < 3 {
        return Circled::make_invalid();
    }
    let center = circle_center_taubin_newton_vec2d(input, cycles);
    // `std::accumulate(input.begin(), input.end(), 0., ...)` (`:113`) -- the accumulator's
    // initial value is decimal-spelled in the vendored line, kept as `0.` plus its own marker
    // rather than the exempt-suffix form (same reason as the `xnew` sites above).
    // PORT SRC: src/libslic3r/Geometry/Circle.cpp:113
    let mut radius = 0.;
    for &pt in input {
        let d = pt - center;
        radius += (d.x() * d.x() + d.y() * d.y()).sqrt();
    }
    radius /= input.len() as f64;
    Circled { center, radius }
}

// TODO(port): src/libslic3r/Geometry/Circle.cpp:119-146 -- `circle_ransac`; see the module doc
// for why it is left unported (`std::mt19937`/`std::sample`, not bit-exact across standard
// library implementations).

#[cfg(test)]
mod tests {
    use super::*;
    use slic3r_proof::{compare_unit, Oracle};

    /// `circle_center<Vec2d>`, `CircleSq<Vec2d>` and `smallest_enclosing_circle_welzl` against
    /// the oracle, each instantiated directly in the shim
    /// (`crates/slic3r-ffi/cxx/proof_units/circle.cpp`) the same way
    /// `bounding_box.cpp`'s `path2_*` methods instantiate header-only template code
    /// (`docs/PORTING.md`, "Decisions recorded on 2026-11-26", decision 2 -- a `proof path: 2`
    /// row's oracle route is the per-unit bridge, not restricted to the committed stage dumps).
    // PROOF: 2
    #[test]
    fn circle_header_only_methods_match_the_oracle() {
        // circle_center, three non-collinear points.
        for &(ax, ay, bx, by, cx, cy) in &[
            (0.0f64, 0.0, 4.0, 0.0, 0.0, 4.0),
            (-3.0, -3.0, 5.0, -1.0, 1.0, 6.0),
            (10.0, 10.0, -7.0, 3.0, 2.0, -9.0),
        ] {
            let actual = circle_center(
                Vec2::new(ax, ay),
                Vec2::new(bx, by),
                Vec2::new(cx, cy),
                1e-9,
            );
            let expected = slic3r_proof::proof_units::circle::path2_circle_center(
                ax, ay, bx, by, cx, cy, 1e-9,
            );
            compare_unit(
                "Geometry/Circle.hpp",
                Oracle::Built,
                vec![actual.x(), actual.y()],
                expected,
            )
            .unwrap_or_else(|e| panic!("circle_center: {e}"));
        }

        // circle_center, collinear points (exercises the tie-break branch).
        let actual = circle_center(
            Vec2::new(0.0, 0.0),
            Vec2::new(10.0, 0.0),
            Vec2::new(4.0, 0.0),
            1e-6,
        );
        let expected = slic3r_proof::proof_units::circle::path2_circle_center(
            0.0, 0.0, 10.0, 0.0, 4.0, 0.0, 1e-6,
        );
        compare_unit(
            "Geometry/Circle.hpp",
            Oracle::Built,
            vec![actual.x(), actual.y()],
            expected,
        )
        .unwrap_or_else(|e| panic!("circle_center (collinear): {e}"));

        // smallest_enclosing_circle_welzl over a small integer point set.
        let points = [
            Point::new(0, 0),
            Point::new(100_000, 0),
            Point::new(0, 100_000),
            Point::new(50_000, 50_000),
            Point::new(20_000, 80_000),
        ];
        let actual = smallest_enclosing_circle_welzl(&points);
        let flat: Vec<i64> = points.iter().flat_map(|p| [p.x(), p.y()]).collect();
        let expected =
            slic3r_proof::proof_units::circle::path2_smallest_enclosing_circle_welzl(flat);
        compare_unit(
            "Geometry/Circle.hpp",
            Oracle::Built,
            vec![actual.center.x(), actual.center.y(), actual.radius],
            expected,
        )
        .unwrap_or_else(|e| panic!("smallest_enclosing_circle_welzl: {e}"));

        // ray_circle_intersections_r2_lv2_c, the two-root case: compare the full result.
        let (count, pts) = ray_circle_intersections_r2_lv2_c(25.0, 1.0, 0.0, 1.0, -3.0);
        let expected = slic3r_proof::proof_units::circle::path2_ray_circle_intersections(
            25.0, 1.0, 0.0, 1.0, -3.0,
        );
        let actual = vec![count as f64, pts[0].x(), pts[0].y(), pts[1].x(), pts[1].y()];
        compare_unit("Geometry/Circle.hpp", Oracle::Built, actual, expected)
            .unwrap_or_else(|e| panic!("ray_circle_intersections_r2_lv2_c (two roots): {e}"));

        // The no-root case (`d2 < 0`): only `count` is well defined on the oracle's own side
        // (the vendored function leaves its out-param untouched when it returns `0`), so only
        // `count` is compared -- see `cxx/proof_units/circle.cpp`'s own comment on this case.
        let (count, _pts) = ray_circle_intersections_r2_lv2_c(1.0, 1.0, 0.0, 1.0, 10.0);
        let expected = slic3r_proof::proof_units::circle::path2_ray_circle_intersections(
            1.0, 1.0, 0.0, 1.0, 10.0,
        );
        compare_unit(
            "Geometry/Circle.hpp",
            Oracle::Built,
            vec![count as f64],
            vec![expected[0]],
        )
        .unwrap_or_else(|e| panic!("ray_circle_intersections_r2_lv2_c (no roots): {e}"));
    }

    /// `circle_center_taubin_newton`/`circle_taubin_newton` against the oracle, both externally
    /// linked (`Circle.cpp`'s own row). Covers both overloads of
    /// `circle_center_taubin_newton`: the `Vec2ds` one (real-world, mm coordinates) and the
    /// `Points` one (`circle.rs:526-533` -- this unit's only integer-coordinate-producing
    /// function and its only scaling conversion), compared as integers rather than through the
    /// `Vec2ds` overload's `f64` result, since a 1-ULP difference in the fit crosses to a
    /// different integer on truncation and the `f64` comparison above cannot establish that the
    /// two agree.
    // PROOF: 1
    #[test]
    fn circle_cpp_functions_match_the_oracle() {
        let points = [
            Vec2::new(10.0f64, 0.0),
            Vec2::new(7.0, 7.0),
            Vec2::new(0.0, 10.0),
            Vec2::new(-7.0, 7.0),
            Vec2::new(-10.0, 0.0),
            Vec2::new(-7.0, -7.0),
        ];
        let flat: Vec<f64> = points.iter().flat_map(|p| [p.x(), p.y()]).collect();

        let actual = circle_center_taubin_newton_vec2d(&points, 20);
        let expected =
            slic3r_proof::proof_units::circle::path1_circle_center_taubin_newton(flat.clone(), 20);
        compare_unit(
            "Geometry/Circle.cpp",
            Oracle::Built,
            vec![actual.x(), actual.y()],
            expected,
        )
        .unwrap_or_else(|e| panic!("circle_center_taubin_newton: {e}"));

        let actual = circle_taubin_newton(&points, 20);
        let expected = slic3r_proof::proof_units::circle::path1_circle_taubin_newton(flat, 20);
        compare_unit(
            "Geometry/Circle.cpp",
            Oracle::Built,
            vec![actual.center.x(), actual.center.y(), actual.radius],
            expected,
        )
        .unwrap_or_else(|e| panic!("circle_taubin_newton: {e}"));

        let scaled_points: Vec<Point> = points
            .iter()
            .map(|p| Point::new_scale(p.x(), p.y()))
            .collect();
        let scaled_flat: Vec<i64> = scaled_points.iter().flat_map(|p| [p.x(), p.y()]).collect();
        let actual = circle_center_taubin_newton(&scaled_points, 20);
        let expected = slic3r_proof::proof_units::circle::path1_circle_center_taubin_newton_point(
            scaled_flat,
            20,
        );
        compare_unit(
            "Geometry/Circle.cpp",
            Oracle::Built,
            vec![actual.x(), actual.y()],
            expected,
        )
        .unwrap_or_else(|e| panic!("circle_center_taubin_newton (Points overload): {e}"));
    }
}

// PORT STATUS
// source: src/libslic3r/Geometry/Circle.cpp
// source: src/libslic3r/Geometry/Circle.hpp
// confidence: medium
// tier: exact
// oracle: built
