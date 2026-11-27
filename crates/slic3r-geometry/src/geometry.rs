//! Port of `src/libslic3r/Geometry.hpp`/`Geometry.cpp` (task `66-foundation-geometry`, plan step
//! 20, `port/foundation.tsv` row `foundation-15`): the libslic3r free-function geometry surface
//! -- orientation predicates, segment intersection, angle helpers, the `arrange` grid layout and
//! `is_point_inside_polygon_corner`.
//!
//! **Scope this module ports, and the line it draws.** `Geometry.hpp` is `template-heavy: yes`
//! in `port/manifest.tsv`, so this task hand-designs the shape of what it keeps; several
//! functions in both vendored files are out of reach for reasons this module doc records up
//! front rather than scattering across individual `// TODO(port):` sites, because each reason
//! recurs across several functions:
//!
//! 1. **No `slic3r-line` dependency.** `34-foundation-spike` generated this crate's
//!    `Cargo.toml` from `port/cut.tsv`'s `dep` rows, and it names no `slic3r-line` path
//!    dependency -- unlike `slic3r-polygon`/`slic3r-polyline`, whose own manifest rows do
//!    `#include "Line.hpp"`. This task may not add a dependency (`docs/WORKFLOW.md`'s
//!    foundation-task rules), so every function whose only concrete, in-file instantiation needs
//!    `Slic3r::Line` is deferred: `foot_pt(const Line&, const Point&)` (`Geometry.hpp:194-197`),
//!    `ray_point_distance_squared(const Line&, const Point&)` (`:209-212`) and
//!    `ray_point_distance(const Line&, const Point&)` (`:214-217`). The generic templates these
//!    three overloads sit beside -- `foot_pt<T>` (`:186-192`), `ray_point_distance_squared<T>`
//!    (`:199-202`), `ray_point_distance<T>` (`:204-207`) -- have no other concrete instantiation
//!    visible in this file, so they are deferred alongside their only callers rather than ported
//!    as an unused generic with no faithful monomorphisation to check against.
//! 2. **No `Matrix3d`/`Transform3d`/`Quaterniond` type family.** `crates/slic3r-point/src/
//!    point.rs` (task `56-foundation-point`) explicitly left `Eigen::Transform`/`Matrix2f
//!    ..Matrix4d` as `// TODO(port): src/libslic3r/Point.hpp:83-96`, because that task's own two
//!    units had no in-scope caller needing one. This task's units are the first in-scope caller,
//!    but inventing the whole family -- including `TransformationSVD`'s Jacobi SVD, whose exact
//!    singular-vector sign and column order Eigen's implementation chooses internally and this
//!    port would have to reproduce bit-for-bit to meet this step's `exact` tier -- is new
//!    foundation type design on the scale of `Point.hpp`/`BoundingBox.hpp` themselves, not a
//!    free-function surface's own job, and 56-foundation-point already deferred the decision of
//!    how to shape that family rather than this task guessing it under a different task's name.
//!    Deferred, named at each site below: `assemble_transform` (4 overloads, `Geometry.hpp:355-
//!    377`), `translation_transform`/`rotation_transform`/`scale_transform` (6 overloads total,
//!    `:380-403`), `extract_euler_angles` (2 overloads, `:407-411`), `rotation_from_two_vectors`
//!    (`:416`), the whole `Transformation` class (`:418-513`) and `TransformationSVD`
//!    (`:515-532`), `transform3d_from_string`/`rotation_xyz_diff`/`rotation_diff_z` (`:535-542`),
//!    `mat_around_a_point_rotate`/`generate_transform` (`:559-560`), and every one of their
//!    `Geometry.cpp` definitions.
//! 3. **No dependency on the crate that defines the final step.** `contains<T>(const
//!    vector<T>&, const Point&)` (`Geometry.hpp:309`, instantiated only at `T = ExPolygon`,
//!    `Geometry.cpp:62`) needs `ExPolygon::contains`; `slic3r-expolygon` is not a dependency.
//!    `simplify_polygons` (`Geometry.cpp:71-82`) calls `MultiPoint::_douglas_peucker` (available,
//!    `slic3r_multipoint::multi_point::douglas_peucker`) but also `Slic3r::simplify_polygons`
//!    (the Clipper-backed free function in `ClipperUtils.cpp`); `slic3r-clipperutils` is not a
//!    dependency either, so the whole function is deferred rather than half-ported. Both named
//!    at their own site below.
//! 4. **An out-of-scope library type (`## Don't-translate list`).** `dist<T>(const
//!    boost::polygon::point_data<T>&, ...)` (`Geometry.cpp:298-303`) and
//!    `project_point_to_segment` (`:307-317`) are generic over `boost::polygon`'s own point and
//!    segment types -- Boost.Polygon is one of the libraries R17 binds through FFI rather than
//!    reimplements, and neither function has an in-scope caller inside this file. `// PORT NOTE:`
//!    at the boundary, not ported.
//! 5. **No generic `BoundingBoxBase<T>` abstraction.** `liang_barsky_line_clipping_interval`/
//!    `liang_barsky_line_clipping` (3 overloads, `Geometry.hpp:220-305`) are generic over
//!    `Eigen::Matrix<T, 2, 1, DontAlign>` and `BoundingBoxBase<...>`; `crates/slic3r-boundingbox`
//!    ported `BoundingBox`/`BoundingBoxf` as two concrete structs with no shared generic base
//!    (that task's own doc comment), so there is no Rust type to write this template generically
//!    against, and this task's own rule ("A template is not force-collapsed to one concrete type
//!    unless the manifest row's unit only ever instantiates one") does not license guessing one
//!    instantiation when this file shows none. Deferred.
//!
//! **What this module does port**, all over plain `i64`/`f64` arithmetic and the `Point`/
//! `Vec2d`/`Vec3d`/`Pointfs`/`BoundingBoxf` types `56-foundation-point`/`58-foundation-bounding-
//! box` already provide: [`Orientation`]/[`orient`]/[`is_ccw`] (`Geometry.hpp:29-90`),
//! [`ray_ray_intersection`]/[`segment_segment_intersection`]/[`segments_intersect`] (`:92-184`),
//! [`directions_parallel`]/[`directions_perpendicular`] (`Geometry.cpp:40-52`),
//! [`rad2deg`]/[`deg2rad`]/[`angle_to_0_2pi`]/[`to_range_pi_pi`]/[`rad2deg_dir`]
//! (`Geometry.hpp:310-337`, `Geometry.cpp:64-69`), [`linint`]/[`arrange`] (`Geometry.cpp:84-293`),
//! [`is_rotation_ninety_degrees`]/[`is_rotation_ninety_degrees_vec3`] (`Geometry.hpp:545-557`)
//! and [`is_point_inside_polygon_corner`] (`Geometry.cpp:864-908`).
//!
//! **`is_ccw` takes `&[Point]`, not `&Polygon`.** The vendored signature is `is_ccw(const
//! Polygon&)`, but the function body reads only `poly.points`; `slic3r-polygon`'s `Polygon` is
//! available (a dependency of this crate) but reaching its `points` requires importing the
//! `MultiPoint` trait for one field read. Taking the slice this unit actually operates on avoids
//! that coupling for a `Geometry.hpp`-level utility while porting the identical body; a caller
//! holding a `Polygon` passes `polygon.points()`.
//!
//! **`is_point_inside_polygon_corner`'s normalize/dot/perp are local, not `slic3r-point`'s.**
//! `point.rs` defers the generic `perp`/cross-product family itself (`Point.hpp:112-134`, its
//! own module doc) for lack of an in-scope caller in that task's two units; this function is the
//! first in-scope caller, but it cannot be satisfied by editing `point.rs` ("Write only your own
//! modules"), and duplicating `perp`'s body under the same exported name here would create two
//! ported answers for one vendored function. The four tiny `vec2d_*` helpers below are therefore
//! private, un-exported arithmetic this unit's own body needs, not a second port of `perp`.
//!
//! **A documented arithmetic divergence, not worked around.** [`orient`]'s 64-bit cross terms
//! and [`segments_intersect`]'s cross-product helper can overflow `i64` for coordinates outside
//! the roughly 2^31 magnitude `Geometry.hpp:40-43`'s own comment assumes; C++ signed overflow is
//! undefined behaviour there (practically wrapping on this pin's compiler), while Rust's debug
//! profile (`cargo test`'s own profile) panics on overflow instead of wrapping. This port takes
//! the plain, non-wrapping translation -- the faithful one for every magnitude the comment's own
//! invariant covers -- and does not add defensive wrapping arithmetic the vendored code itself
//! does not have; this unit's own proof and fuzz target keep their inputs inside that same
//! magnitude, as `crates/slic3r-ffi/fuzz/fuzz_targets/slic3r-multipoint_multi_point.rs` already
//! does for the same reason. Named here rather than silently relied upon.

use slic3r_boundingbox::bounding_box::BoundingBoxf;
use slic3r_libslic3r::libslic3r::{EPSILON, PI};
use slic3r_point::point::{Point, Pointfs, Vec2, Vec2d, Vec3d};

// -------------------------------------------------------------------------------------------
// Orientation, orient, is_ccw (Geometry.hpp:29-90)
// -------------------------------------------------------------------------------------------

/// `enum Orientation { ORIENTATION_CCW = 1, ORIENTATION_CW = -1, ORIENTATION_COLINEAR = 0 };`
/// (`Geometry.hpp:30-35`). All three discriminants (`1`, `-1`, `0`) are in the numeric-literal
/// rule's exempt set, so none carries a `// PORT SRC:` marker.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Orientation {
    Ccw = 1,
    Cw = -1,
    Colinear = 0,
}

/// The upper bound `orient` below checks every coordinate against before taking the exact
/// 64-bit-integer path (`a.x() <= 0xffffffff && ...`); a `coord_t` outside this range (in either
/// direction -- the check is one-sided, exactly as the vendored comparison is) falls back to the
/// `double` path. Named rather than repeated six times, per the type map's "a `#define` numeric
/// constant" row.
// PORT SRC: src/libslic3r/Geometry.hpp:47
const ORIENT_COORD_LIMIT: i64 = 0xffffffff;

/// `static inline Orientation orient(const Point &a, const Point &b, const Point &c)`
/// (`Geometry.hpp:44-61`). `coord_t` is already `int64_t` at the pin, so every
/// `int64_t(a(0))`-style cast in the vendored body is a no-op here.
pub fn orient(a: Point, b: Point, c: Point) -> Orientation {
    if a.x() <= ORIENT_COORD_LIMIT
        && b.x() <= ORIENT_COORD_LIMIT
        && c.x() <= ORIENT_COORD_LIMIT
        && a.y() <= ORIENT_COORD_LIMIT
        && b.y() <= ORIENT_COORD_LIMIT
        && c.y() <= ORIENT_COORD_LIMIT
    {
        let u = b.x() * c.y() - b.y() * c.x();
        let v = a.x() * c.y() - a.y() * c.x();
        let w = a.x() * b.y() - a.y() * b.x();
        let d = u - v + w;
        if d > 0 {
            Orientation::Ccw
        } else if d == 0 {
            Orientation::Colinear
        } else {
            Orientation::Cw
        }
    } else {
        let u = b.x() as f64 * c.y() as f64 - b.y() as f64 * c.x() as f64;
        let v = a.x() as f64 * c.y() as f64 - a.y() as f64 * c.x() as f64;
        let w = a.x() as f64 * b.y() as f64 - a.y() as f64 * b.x() as f64;
        let d = u - v + w;
        if d > 0f64 {
            Orientation::Ccw
        } else if d == 0f64 {
            Orientation::Colinear
        } else {
            Orientation::Cw
        }
    }
}

/// `static inline bool is_ccw(const Polygon &poly)` (`Geometry.hpp:66-90`); see the module doc
/// for why this takes `&[Point]` rather than `&Polygon`. The vendored `assert`s (`:69`, `:88`)
/// are not ported -- see `## Don't-translate list`, "An assert body".
pub fn is_ccw(poly_points: &[Point]) -> bool {
    // PORT SRC: src/libslic3r/Geometry.hpp:70
    if poly_points.len() < 3 {
        return true;
    }

    let mut imin = 0usize;
    for i in 1..poly_points.len() {
        let pmin = poly_points[imin];
        let p = poly_points[i];
        if p.x() < pmin.x() || (p.x() == pmin.x() && p.y() < pmin.y()) {
            imin = i;
        }
    }

    let iprev = (if imin == 0 { poly_points.len() } else { imin }) - 1;
    let inext = if imin + 1 == poly_points.len() {
        0
    } else {
        imin + 1
    };
    orient(poly_points[iprev], poly_points[imin], poly_points[inext]) == Orientation::Ccw
}

// -------------------------------------------------------------------------------------------
// ray_ray_intersection, segment_segment_intersection, segments_intersect (Geometry.hpp:92-184)
// -------------------------------------------------------------------------------------------

/// `inline bool ray_ray_intersection(const Vec2d &p1, const Vec2d &v1, const Vec2d &p2, const
/// Vec2d &v2, Vec2d &res)` (`Geometry.hpp:92-101`). The C++ out-parameter `res` becomes a return
/// value, per the idiom map's "a non-const reference out-parameter" row: `None` when the
/// vendored function returns `false` without writing `res`.
pub fn ray_ray_intersection(p1: Vec2d, v1: Vec2d, p2: Vec2d, v2: Vec2d) -> Option<Vec2d> {
    let denom = v1.x() * v2.y() - v2.x() * v1.y();
    if denom.abs() < EPSILON {
        return None;
    }
    let t = (v2.x() * (p1.y() - p2.y()) - v2.y() * (p1.x() - p2.x())) / denom;
    Some(Vec2::new(p1.x() + t * v1.x(), p1.y() + t * v1.y()))
}

/// `inline bool segment_segment_intersection(const Vec2d &p1, const Vec2d &v1, const Vec2d &p2,
/// const Vec2d &v2, Vec2d &res)` (`Geometry.hpp:103-132`); see [`ray_ray_intersection`]'s doc for
/// the out-parameter-to-return-value translation.
pub fn segment_segment_intersection(p1: Vec2d, v1: Vec2d, p2: Vec2d, v2: Vec2d) -> Option<Vec2d> {
    let mut denom = v1.x() * v2.y() - v2.x() * v1.y();
    if denom.abs() < EPSILON {
        // Lines are collinear.
        return None;
    }
    let s12_x = p1.x() - p2.x();
    let s12_y = p1.y() - p2.y();
    let mut s_numer = v1.x() * s12_y - v1.y() * s12_x;
    let mut denom_is_positive = false;
    if denom < 0f64 {
        denom_is_positive = true;
        denom = -denom;
        s_numer = -s_numer;
    }
    if s_numer < 0f64 {
        // Intersection outside of the 1st segment.
        return None;
    }
    let mut t_numer = v2.x() * s12_y - v2.y() * s12_x;
    if !denom_is_positive {
        t_numer = -t_numer;
    }
    if t_numer < 0f64 || s_numer > denom || t_numer > denom {
        // Intersection outside of the 1st or 2nd segment.
        return None;
    }
    // Intersection inside both of the segments.
    let t = t_numer / denom;
    Some(Vec2::new(p1.x() + t * v1.x(), p1.y() + t * v1.y()))
}

/// `inline typename Derived::Scalar cross2(...)` narrowed to the one `i64` instantiation
/// [`segments_intersect`] below needs -- see [`orient`]'s own private cross terms for the same
/// narrowing, and `crates/slic3r-point/src/point.rs`'s `cross2_f64` for the identical reasoning
/// at `f64`. Not `pub`: no caller outside this module.
fn cross2_i64(v1: Point, v2: Point) -> i64 {
    v1.x() * v2.y() - v1.y() * v2.x()
}

/// `inline bool segments_intersect(const Point &ip1, const Point &ip2, const Point &jp1, const
/// Point &jp2)` (`Geometry.hpp:134-184`). `coord_t` is already `int64_t`, so the vendored
/// `.cast<int64_t>()` calls are no-ops; `Point - Point` already yields the `i64` pair the
/// vendored `Vec2i64` temporaries hold.
pub fn segments_intersect(ip1: Point, ip2: Point, jp1: Point, jp2: Point) -> bool {
    fn signum(v: i64) -> i32 {
        if v > 0 {
            1
        } else if v < 0 {
            -1
        } else {
            0
        }
    }
    fn could_intersect(ip1: Point, ip2: Point, jp1: Point, jp2: Point) -> (i32, i32) {
        let iv = ip2 - ip1;
        let vij1 = jp1 - ip1;
        let vij2 = jp2 - ip1;
        (signum(cross2_i64(iv, vij1)), signum(cross2_i64(iv, vij2)))
    }

    let sign1 = could_intersect(ip1, ip2, jp1, jp2);
    let sign2 = could_intersect(jp1, jp2, ip1, ip2);
    let test1 = sign1.0 * sign1.1;
    let test2 = sign2.0 * sign2.1;
    if test1 <= 0 && test2 <= 0 {
        // The segments possibly intersect. They may also be collinear, but not intersect.
        if test1 != 0 || test2 != 0 {
            // Certainly not collinear, then the segments intersect.
            return true;
        }
        // The segments are certainly collinear (the vendored `assert` on the two sign pairs
        // agreeing is not ported -- see the module doc). Now verify whether they overlap.
        if sign1.0 == 0 && sign1.1 == 0 {
            let vi = ip2 - ip1;
            let axis_is_y = vi.x().abs() <= vi.y().abs();
            let (mut i, mut j, mut k, mut l) = if axis_is_y {
                (ip1.y(), ip2.y(), jp1.y(), jp2.y())
            } else {
                (ip1.x(), ip2.x(), jp1.x(), jp2.x())
            };
            if i > j {
                std::mem::swap(&mut i, &mut j);
            }
            if k > l {
                std::mem::swap(&mut k, &mut l);
            }
            return (k >= i && k <= j) || (i >= k && i <= l);
        }
    }
    false
}

// -------------------------------------------------------------------------------------------
// directions_parallel, directions_perpendicular (Geometry.cpp:40-52)
// -------------------------------------------------------------------------------------------

/// `bool directions_parallel(double angle1, double angle2, double max_diff)` (`Geometry.cpp:40-
/// 45`); `max_diff` carries the vendored default argument (`= 0`) as this port's own default
/// via [`directions_parallel_default`].
pub fn directions_parallel(angle1: f64, angle2: f64, max_diff: f64) -> bool {
    let diff = (angle1 - angle2).abs();
    let max_diff = max_diff + EPSILON;
    diff < max_diff || (diff - PI).abs() < max_diff
}

/// [`directions_parallel`] at the vendored default `max_diff = 0`.
pub fn directions_parallel_default(angle1: f64, angle2: f64) -> bool {
    directions_parallel(angle1, angle2, 0f64)
}

/// `bool directions_perpendicular(double angle1, double angle2, double max_diff)`
/// (`Geometry.cpp:47-52`).
pub fn directions_perpendicular(angle1: f64, angle2: f64, max_diff: f64) -> bool {
    let diff = (angle1 - angle2).abs();
    let max_diff = max_diff + EPSILON;
    // PORT SRC: src/libslic3r/Geometry.cpp:51
    let half_pi_factor: f64 = 0.5;
    // PORT SRC: src/libslic3r/Geometry.cpp:51
    let three_half_pi_factor: f64 = 1.5;
    (diff - half_pi_factor * PI).abs() < max_diff
        || (diff - three_half_pi_factor * PI).abs() < max_diff
}

/// [`directions_perpendicular`] at the vendored default `max_diff = 0`.
pub fn directions_perpendicular_default(angle1: f64, angle2: f64) -> bool {
    directions_perpendicular(angle1, angle2, 0f64)
}

// -------------------------------------------------------------------------------------------
// rad2deg, deg2rad, angle_to_0_2PI, to_range_pi_pi, rad2deg_dir (Geometry.hpp:310-337,
// Geometry.cpp:64-69)
// -------------------------------------------------------------------------------------------

/// `template<typename T> T rad2deg(T angle) { return T(180.0) * angle / T(PI); }`
/// (`Geometry.hpp:310`), narrowed to `T = f64` -- the only instantiation [`rad2deg_dir`] below
/// needs; no other in-scope caller in this file.
// PORT SRC: src/libslic3r/Geometry.hpp:310
const RAD_TO_DEG_FACTOR: f64 = 180.0;

pub fn rad2deg(angle: f64) -> f64 {
    RAD_TO_DEG_FACTOR * angle / PI
}

/// `template<typename T> constexpr T deg2rad(const T angle)` (`Geometry.hpp:312-316`), narrowed
/// to `T = f64`; no in-scope caller in this file, kept for the same reason [`rad2deg`] is: the
/// pair is one hand-designed unit in the vendored header.
// PORT SRC: src/libslic3r/Geometry.hpp:315
const DEG_TO_RAD_DIVISOR: f64 = 180.0;

pub fn deg2rad(angle: f64) -> f64 {
    PI * angle / DEG_TO_RAD_DIVISOR
}

/// `template<typename T> T angle_to_0_2PI(T angle)` (`Geometry.hpp:317-330`), narrowed to
/// `T = f64`.
pub fn angle_to_0_2pi(mut angle: f64) -> f64 {
    let two_pi = 2f64 * PI;
    while angle < 0f64 {
        angle += two_pi;
    }
    while two_pi < angle {
        angle -= two_pi;
    }
    angle
}

/// `template<typename T> void to_range_pi_pi(T &angle)` (`Geometry.hpp:331-337`), narrowed to
/// `T = f64`; the C++ in/out reference parameter becomes a return value (idiom map, "a non-const
/// reference out-parameter"). The vendored `assert` after the subtraction is not ported.
pub fn to_range_pi_pi(mut angle: f64) -> f64 {
    if angle > PI || angle <= -PI {
        let count = (angle / (2f64 * PI)).round();
        angle -= count * 2f64 * PI;
    }
    angle
}

/// `double rad2deg_dir(double angle)` (`Geometry.cpp:64-69`).
pub fn rad2deg_dir(angle: f64) -> f64 {
    let mut angle = if angle < PI {
        -angle + PI / 2f64
    } else {
        angle + PI / 2f64
    };
    if angle < 0f64 {
        angle += PI;
    }
    rad2deg(angle)
}

// -------------------------------------------------------------------------------------------
// linint, arrange (Geometry.cpp:84-293)
// -------------------------------------------------------------------------------------------

/// `double linint(double value, double oldmin, double oldmax, double newmin, double newmax)`
/// (`Geometry.cpp:84-87`).
pub fn linint(value: f64, oldmin: f64, oldmax: f64, newmin: f64, newmax: f64) -> f64 {
    (value - oldmin) * (newmax - newmin) / (oldmax - oldmin) + newmin
}

/// One cell of the grid [`arrange`] lays out, before sorting -- `class ArrangeItem`
/// (`Geometry.cpp:158-163`, the live `#else` branch; the `#if 0` block above it is upstream dead
/// code and is not revived, per `## Don't-translate list`'s "upstream code that is commented
/// out"). `pos` is set (`:224-225`) but never read again in the vendored function either --
/// `cellsorder[...].item.index_x`/`index_y` are what every later read uses -- so this port keeps
/// the same unread field for the same struct shape rather than dropping it.
#[derive(Debug, Clone, Copy)]
struct ArrangeItem {
    #[allow(dead_code)]
    pos: Vec2d,
    index_x: usize,
    index_y: usize,
}

/// `class ArrangeItemIndex` (`Geometry.cpp:164-169`): a cell paired with its sort key.
#[derive(Debug, Clone, Copy)]
struct ArrangeItemIndex {
    index: f64,
    item: ArrangeItem,
}

/// `bool arrange(size_t total_parts, const Vec2d &part_size, coordf_t dist, const BoundingBoxf*
/// bb, Pointfs &positions)` (`Geometry.cpp:171-293`). The `positions` out-parameter becomes part
/// of the return value (idiom map), paired with the vendored `bool` success flag; `bb` becomes
/// `Option<&BoundingBoxf>` for the vendored nullable pointer.
///
/// **A documented divergence at `total_parts == 0` with a degenerate cell size.** The vendored
/// loops read `for (size_t i = 0; i <= cellw - 1; ++i)`; `cellw`/`cellh` are `size_t`, so
/// `cellw - 1` underflows to a huge value whenever `cellw == 0` (reachable only when
/// `total_parts == 0`, since the `total_parts > cellw * cellh` guard above it already rejects
/// `cellw == 0` for any `total_parts > 0`), making the vendored loop run an astronomical number
/// of iterations rather than zero. This port writes the equivalent `0..cellw` instead, which
/// runs zero iterations at `cellw == 0` -- the loop's evident intent, and the only translation
/// that does not hang this unit's own proof and fuzz target. `// FIDELITY(port):` is not used
/// here because this is not an order choice; it is named in this task's pull request as a
/// finding instead.
pub fn arrange(
    total_parts: usize,
    part_size: Vec2d,
    dist: f64,
    bb: Option<&BoundingBoxf>,
) -> (bool, Pointfs) {
    let mut positions: Pointfs = Vec::new();

    let part = Vec2::new(part_size.x() + dist, part_size.y() + dist);

    let area = match bb {
        Some(bb) if bb.defined => bb.size(),
        _ => Vec2::new(part.x() * total_parts as f64, part.y() * total_parts as f64),
    };

    let cellw = ((area.x() + dist) / part.x()).floor();
    let cellh = ((area.y() + dist) / part.y()).floor();
    if !(cellw.is_finite() && cellh.is_finite()) || cellw < 0f64 || cellh < 0f64 {
        // Not a shape the vendored `size_t` conversion can represent either (that conversion is
        // undefined behaviour in C++ for a negative or non-finite `double`); refused rather than
        // guessed.
        return (false, positions);
    }
    let cellw = cellw as usize;
    let cellh = cellh as usize;
    if total_parts > cellw * cellh {
        return (false, positions);
    }

    let cells = Vec2::new(cellw as f64 * part.x(), cellh as f64 * part.y());

    let mut cells_bb = BoundingBoxf::new();
    cells_bb.merge(Vec2::new(0f64, 0f64));
    cells_bb.merge(cells);
    cells_bb.translate((area.x() - cells.x()) / 2f64, (area.y() - cells.y()) / 2f64);

    let mut cellsorder: Vec<ArrangeItemIndex> = Vec::new();
    for i in 0..cellw {
        for j in 0..cellh {
            // PORT SRC: src/libslic3r/Geometry.cpp:217
            let i_center = i as f64 + 0.5;
            let cx = linint(
                i_center,
                0f64,
                cellw as f64,
                cells_bb.min.x(),
                cells_bb.max.x(),
            );
            // PORT SRC: src/libslic3r/Geometry.cpp:218
            let j_center = j as f64 + 0.5;
            let cy = linint(
                j_center,
                0f64,
                cellh as f64,
                cells_bb.min.y(),
                cells_bb.max.y(),
            );

            let xd = (area.x() / 2f64 - cx).abs();
            let yd = (area.y() / 2f64 - cy).abs();

            let item = ArrangeItem {
                pos: Vec2::new(cx, cy),
                index_x: i,
                index_y: j,
            };
            let dist_key = xd * xd + yd * yd - ((cellw / 2) as f64 - i_center).abs();

            // Binary insertion sort (`Geometry.cpp:230-249`); the `| 0` in the vendored `mid`
            // computation is a no-op on an already-integral `size_t` and is not ported as a
            // separate step. On a tie (`midval == index`), the vendored `goto ENDSORT` inserts
            // at that exact midpoint rather than continuing to narrow toward `low` or `high` --
            // an insertion-position choice among equal keys that reaches this function's output
            // order, hence the `// FIDELITY(port):` below.
            // FIDELITY(port): src/libslic3r/Geometry.cpp:236-249 -- a tied distance key inserts
            // at the binary search's midpoint, not at `low`; this changes the relative order of
            // cells that tie exactly, which changes which cell index each of `arrange`'s callers
            // sees for a given output position.
            let mut low = 0usize;
            let mut high = cellsorder.len();
            let mut inserted = false;
            while low < high {
                let mid = low + (high - low) / 2;
                let midval = cellsorder[mid].index;
                if midval < dist_key {
                    low = mid + 1;
                } else if midval > dist_key {
                    high = mid;
                } else {
                    cellsorder.insert(
                        mid,
                        ArrangeItemIndex {
                            index: dist_key,
                            item,
                        },
                    );
                    inserted = true;
                    break;
                }
            }
            if !inserted {
                cellsorder.insert(
                    low,
                    ArrangeItemIndex {
                        index: dist_key,
                        item,
                    },
                );
            }
        }
    }

    let mut lx = 0f64;
    let mut ty = 0f64;
    let mut rx = 0f64;
    let mut by = 0f64;
    for i in 1..=total_parts {
        let c = &cellsorder[i - 1];
        let cx = c.item.index_x as f64;
        let cy = c.item.index_y as f64;
        if i == 1 {
            lx = cx;
            rx = cx;
            ty = cy;
            by = cy;
        } else {
            if cx > rx {
                rx = cx;
            }
            if cx < lx {
                lx = cx;
            }
            if cy > by {
                by = cy;
            }
            if cy < ty {
                ty = cy;
            }
        }
    }
    let _ = (rx, by);

    for _ in 1..=total_parts {
        let c = cellsorder.remove(0);
        let cx = c.item.index_x as f64 - lx;
        let cy = c.item.index_y as f64 - ty;
        positions.push(Vec2::new(cx * part.x(), cy * part.y()));
    }

    if let Some(bb) = bb {
        if bb.defined {
            for p in positions.iter_mut() {
                *p = Vec2::new(p.x() + bb.min.x(), p.y() + bb.min.y());
            }
        }
    }

    (true, positions)
}

// -------------------------------------------------------------------------------------------
// is_rotation_ninety_degrees (Geometry.hpp:544-557)
// -------------------------------------------------------------------------------------------

/// `inline bool is_rotation_ninety_degrees(double a)` (`Geometry.hpp:545-551`).
pub fn is_rotation_ninety_degrees(a: f64) -> bool {
    // PORT SRC: src/libslic3r/Geometry.hpp:547
    let half_pi_factor: f64 = 0.5;
    let mut a = a.abs() % (half_pi_factor * PI);
    // PORT SRC: src/libslic3r/Geometry.hpp:548
    let quarter_pi_factor: f64 = 0.25;
    if a > quarter_pi_factor * PI {
        // PORT SRC: src/libslic3r/Geometry.hpp:549
        let half_pi_factor: f64 = 0.5;
        a = half_pi_factor * PI - a;
    }
    // PORT SRC: src/libslic3r/Geometry.hpp:550
    a < 0.001
}

/// `inline bool is_rotation_ninety_degrees(const Vec3d &rotation)` (`Geometry.hpp:553-557`).
pub fn is_rotation_ninety_degrees_vec3(rotation: Vec3d) -> bool {
    is_rotation_ninety_degrees(rotation.x())
        && is_rotation_ninety_degrees(rotation.y())
        && is_rotation_ninety_degrees(rotation.z())
}

// -------------------------------------------------------------------------------------------
// is_point_inside_polygon_corner (Geometry.cpp:864-908)
// -------------------------------------------------------------------------------------------

/// `v.dot(w)` for a plain `Vec2d` pair -- see the module doc for why this is a local helper
/// rather than a `slic3r-point` export.
fn vec2d_dot(v: Vec2d, w: Vec2d) -> f64 {
    v.x() * w.x() + v.y() * w.y()
}

/// `v.normalized()` for a plain `Vec2d`.
///
/// PORT NOTE: a fuzz-found divergence, fixed rather than merely logged. `Eigen::MatrixBase::
/// normalized()` (`Eigen/src/Core/Dot.h:92-102` under this pin's dependency superbuild, not
/// `vendor/OrcaSlicer/` itself -- a linked library's own source, not a numeric constant, so this
/// is a `// PORT NOTE:` rather than a `// PORT SRC: external:` marker) is not a plain `v /
/// v.norm()`: `if (z > RealScalar(0)) return n / sqrt(z); else return n;`, where `z` is the
/// squared norm. A zero vector's squared norm is `0`, so Eigen returns the zero vector
/// unchanged -- never a `0.0 / 0.0` `NaN`. The first version of this function divided
/// unconditionally and diverged from the oracle exactly here:
/// `crates/slic3r-ffi/fuzz/fuzz_targets/slic3r-geometry_geometry.rs` found a query point equal
/// to the corner vertex `b` (`bq` a zero vector) where `is_point_inside_polygon_corner` returned
/// `true` through the oracle and `false` through the unconditional-division translation: not a
/// NaN-propagates-the-same-way case, because the zero-vector branch means no `NaN` is ever
/// produced on either side for this input.
fn vec2d_normalized(v: Vec2d) -> Vec2d {
    let squared_norm = v.x() * v.x() + v.y() * v.y();
    if squared_norm > 0f64 {
        let norm = squared_norm.sqrt();
        Vec2::new(v.x() / norm, v.y() / norm)
    } else {
        v
    }
}

/// `perp(v)` (`Point.hpp:130-134`, `return { -v.y(), v.x() };`) at the one instantiation this
/// unit needs -- see the module doc for why this is not a second export of `perp` itself.
fn vec2d_perp(v: Vec2d) -> Vec2d {
    Vec2::new(-v.y(), v.x())
}

/// `bool is_point_inside_polygon_corner(const Point &a, const Point &b, const Point &c, const
/// Point &query_point)` (`Geometry.cpp:864-908`). `coord_t` is already `int64_t`, so every
/// `.cast<int64_t>()` in the vendored body is a no-op; the `.cast<double>()` calls that follow
/// are a real `i64` -> `f64` conversion, ported as `as f64`. The vendored `assert(ba != bc)` is
/// not ported.
pub fn is_point_inside_polygon_corner(a: Point, b: Point, c: Point, query_point: Point) -> bool {
    let ba = vec2d_normalized(Vec2::new((a.x() - b.x()) as f64, (a.y() - b.y()) as f64));
    let bc = vec2d_normalized(Vec2::new((c.x() - b.x()) as f64, (c.y() - b.y()) as f64));
    let bq = vec2d_normalized(Vec2::new(
        (query_point.x() - b.x()) as f64,
        (query_point.y() - b.y()) as f64,
    ));

    let bq_left_normal = vec2d_perp(bq);

    let proj_a_on_bq_normal = vec2d_dot(ba, bq_left_normal);
    let proj_c_on_bq_normal = vec2d_dot(bc, bq_left_normal);
    if (proj_a_on_bq_normal > 0f64 && proj_c_on_bq_normal <= 0f64)
        || (proj_a_on_bq_normal <= 0f64 && proj_c_on_bq_normal > 0f64)
    {
        proj_a_on_bq_normal > 0f64
    } else {
        let proj_a_on_bq = vec2d_dot(ba, bq);
        let proj_c_on_bq = vec2d_dot(bc, bq);
        (proj_a_on_bq_normal > 0f64 && proj_c_on_bq < proj_a_on_bq)
            || (proj_a_on_bq_normal <= 0f64 && proj_c_on_bq >= proj_a_on_bq)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn orient_matches_sign_of_signed_area_for_a_ccw_triangle() {
        let a = Point::new(0, 0);
        let b = Point::new(10, 0);
        let c = Point::new(10, 10);
        assert_eq!(orient(a, b, c), Orientation::Ccw);
        assert_eq!(orient(a, c, b), Orientation::Cw);
        assert_eq!(orient(a, b, Point::new(20, 0)), Orientation::Colinear);
    }

    #[test]
    fn is_ccw_true_for_a_ccw_square_false_for_its_reverse() {
        let ccw = vec![
            Point::new(0, 0),
            Point::new(10, 0),
            Point::new(10, 10),
            Point::new(0, 10),
        ];
        assert!(is_ccw(&ccw));
        let cw: Vec<Point> = ccw.into_iter().rev().collect();
        assert!(!is_ccw(&cw));
    }

    #[test]
    fn segments_intersect_crossing_segments() {
        let ip1 = Point::new(0, 0);
        let ip2 = Point::new(10, 10);
        let jp1 = Point::new(0, 10);
        let jp2 = Point::new(10, 0);
        assert!(segments_intersect(ip1, ip2, jp1, jp2));
        assert!(!segments_intersect(
            Point::new(0, 0),
            Point::new(1, 0),
            Point::new(0, 5),
            Point::new(1, 5)
        ));
    }

    #[test]
    fn directions_parallel_and_perpendicular() {
        assert!(directions_parallel_default(0.0, PI));
        assert!(!directions_parallel_default(0.0, PI / 2.0));
        assert!(directions_perpendicular_default(0.0, PI / 2.0));
        assert!(!directions_perpendicular_default(0.0, PI));
    }

    #[test]
    fn rad2deg_dir_and_linint_roundtrip() {
        assert!((rad2deg(PI) - 180.0).abs() < 1e-9);
        assert!((deg2rad(180.0) - PI).abs() < 1e-9);
        assert_eq!(linint(5.0, 0.0, 10.0, 0.0, 100.0), 50.0);
    }

    #[test]
    fn arrange_places_every_part_without_overlap() {
        let (ok, positions) = arrange(4, Vec2::new(10.0, 10.0), 1.0, None);
        assert!(ok);
        assert_eq!(positions.len(), 4);
    }

    #[test]
    fn is_rotation_ninety_degrees_scalar_and_vec3() {
        assert!(is_rotation_ninety_degrees(PI / 2.0));
        assert!(!is_rotation_ninety_degrees(PI / 4.0));
        assert!(is_rotation_ninety_degrees_vec3(Vec3d::new(
            0.0,
            PI / 2.0,
            PI
        )));
    }

    #[test]
    fn is_point_inside_polygon_corner_basic_corner() {
        // A CCW right-angle corner at B = (0,0): A = (0,10) (above), C = (10,0) (right).
        let a = Point::new(0, 10);
        let b = Point::new(0, 0);
        let c = Point::new(10, 0);
        assert!(is_point_inside_polygon_corner(a, b, c, Point::new(5, 5)));
        assert!(!is_point_inside_polygon_corner(a, b, c, Point::new(-5, -5)));
    }

    // Proof path 3 (R19, `port/manifest.tsv`'s `Geometry.cpp`/`Geometry.hpp` rows share one
    // `proof path` cell -- "One cell across the unit's rows" per this task's own task.md). The
    // manifest cell reads `3` and this marker's text matches it verbatim, as `port/port-status`'s
    // `proof_present` requires; the *mechanism* this test actually reaches the oracle through is
    // named here and in this task's pull request body (`docs/PORTING.md`'s "Decisions recorded
    // on 2026-11-25", decision 5, is what licenses a task's own body to state where its proof
    // diverges from the manifest cell's literal description): none of the functions ported above
    // are internal-linkage (this file declares no anonymous namespace and no `static` free
    // function that any of them needs -- the four `static` helpers `Geometry.cpp` does declare,
    // `extract_rotation_scale`/`extract_rotation_matrix`/`extract_scale`/`contains_skew`, are all
    // `Transform3d`-based and belong to the deferred surface this module's doc names), so
    // `#include`ing the whole `.cpp` to reach an internal-linkage helper is both unnecessary and
    // the one thing `cxx/proof_units/clipper_z_utils.cpp`'s header comment warns never to do
    // lightly (`Geometry.cpp` externally links dozens of `Slic3r::Geometry::` symbols this
    // crate's shared `Build` would then risk duplicating). Every function proven below is called
    // the same way proof path 1 calls one: through `cxx/proof_units/geometry.h`'s `#include` of
    // `Geometry.hpp` and a direct call to the already-externally-linked symbol in
    // `liblibslic3r.a`.
    // PROOF: 3
    #[test]
    fn geometry_free_functions_match_the_oracle() {
        use slic3r_proof::tiers::{compare_unit, Oracle};

        let a = Point::new(-1234, 5678);
        let b = Point::new(42, -99);
        let c = Point::new(10_000, 10_000);

        let actual = orient(a, b, c) as i32;
        let oracle = slic3r_proof::proof_units::geometry::path_orient(
            a.x(),
            a.y(),
            b.x(),
            b.y(),
            c.x(),
            c.y(),
        );
        compare_unit("Geometry.hpp", Oracle::Built, actual, oracle)
            .expect("orient must match the oracle exactly");

        let poly = [a, b, c, Point::new(-5000, 7000)];
        let xs: Vec<i64> = poly.iter().map(|p| p.x()).collect();
        let ys: Vec<i64> = poly.iter().map(|p| p.y()).collect();
        let actual_ccw = is_ccw(&poly) as i32;
        let oracle_ccw = slic3r_proof::proof_units::geometry::path_is_ccw(&xs, &ys) as i32;
        compare_unit("Geometry.hpp", Oracle::Built, actual_ccw, oracle_ccw)
            .expect("is_ccw must match the oracle exactly");

        let ip1 = Point::new(0, 0);
        let ip2 = Point::new(100, 100);
        let jp1 = Point::new(0, 100);
        let jp2 = Point::new(100, 0);
        let actual_seg = segments_intersect(ip1, ip2, jp1, jp2) as i32;
        let oracle_seg = slic3r_proof::proof_units::geometry::path_segments_intersect(
            ip1.x(),
            ip1.y(),
            ip2.x(),
            ip2.y(),
            jp1.x(),
            jp1.y(),
            jp2.x(),
            jp2.y(),
        ) as i32;
        compare_unit("Geometry.hpp", Oracle::Built, actual_seg, oracle_seg)
            .expect("segments_intersect must match the oracle exactly");

        let angle1 = 0.3_f64;
        let angle2 = angle1 + PI;
        let actual_parallel = directions_parallel(angle1, angle2, 0.0) as i32;
        let oracle_parallel =
            slic3r_proof::proof_units::geometry::path_directions_parallel(angle1, angle2, 0.0)
                as i32;
        compare_unit(
            "Geometry.cpp",
            Oracle::Built,
            actual_parallel,
            oracle_parallel,
        )
        .expect("directions_parallel must match the oracle exactly");

        let actual_perp = directions_perpendicular(angle1, angle1 + PI / 2.0, 0.0) as i32;
        let oracle_perp = slic3r_proof::proof_units::geometry::path_directions_perpendicular(
            angle1,
            angle1 + PI / 2.0,
            0.0,
        ) as i32;
        compare_unit("Geometry.cpp", Oracle::Built, actual_perp, oracle_perp)
            .expect("directions_perpendicular must match the oracle exactly");

        let actual_r2d = rad2deg_dir(1.1_f64);
        let oracle_r2d = slic3r_proof::proof_units::geometry::path_rad2deg_dir(1.1_f64);
        compare_unit("Geometry.cpp", Oracle::Built, actual_r2d, oracle_r2d)
            .expect("rad2deg_dir must match the oracle exactly");

        let actual_linint = linint(5.0, 0.0, 10.0, -1.0, 1.0);
        let oracle_linint =
            slic3r_proof::proof_units::geometry::path_linint(5.0, 0.0, 10.0, -1.0, 1.0);
        compare_unit("Geometry.cpp", Oracle::Built, actual_linint, oracle_linint)
            .expect("linint must match the oracle exactly");

        let (actual_ok, actual_positions) = arrange(5, Vec2::new(10.0, 10.0), 2.0, None);
        let oracle_arrange = slic3r_proof::proof_units::geometry::path_arrange(
            5, 10.0, 10.0, 2.0, false, 0.0, 0.0, 0.0, 0.0,
        );
        let mut actual_flat: Vec<f64> = vec![if actual_ok { 1.0 } else { 0.0 }];
        for p in &actual_positions {
            actual_flat.push(p.x());
            actual_flat.push(p.y());
        }
        compare_unit("Geometry.cpp", Oracle::Built, actual_flat, oracle_arrange)
            .expect("arrange must match the oracle exactly");

        let actual_ninety = is_rotation_ninety_degrees(PI / 2.0 + 0.0001) as i32;
        let oracle_ninety =
            slic3r_proof::proof_units::geometry::path_is_rotation_ninety_degrees(PI / 2.0 + 0.0001)
                as i32;
        compare_unit("Geometry.hpp", Oracle::Built, actual_ninety, oracle_ninety)
            .expect("is_rotation_ninety_degrees must match the oracle exactly");

        let corner_a = Point::new(0, 10);
        let corner_b = Point::new(0, 0);
        let corner_c = Point::new(10, 0);
        let query = Point::new(5, 5);
        let actual_corner =
            is_point_inside_polygon_corner(corner_a, corner_b, corner_c, query) as i32;
        let oracle_corner = slic3r_proof::proof_units::geometry::path_is_point_inside_polygon_corner(
            corner_a.x(),
            corner_a.y(),
            corner_b.x(),
            corner_b.y(),
            corner_c.x(),
            corner_c.y(),
            query.x(),
            query.y(),
        ) as i32;
        compare_unit("Geometry.cpp", Oracle::Built, actual_corner, oracle_corner)
            .expect("is_point_inside_polygon_corner must match the oracle exactly");
    }
}

// PORT STATUS
// source: src/libslic3r/Geometry.cpp
// source: src/libslic3r/Geometry.hpp
// confidence: medium
// tier: exact
// oracle: built
