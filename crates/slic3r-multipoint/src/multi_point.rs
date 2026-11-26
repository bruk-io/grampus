//! Port of `src/libslic3r/MultiPoint.hpp`/`MultiPoint.cpp` (task `62-foundation-polygon`, plan
//! step 20, `port/foundation.tsv` row `foundation-11`): the port's shared polyline/polygon base
//! -- the concrete behaviour every 2D point-sequence type (`Polygon`, and later `Polyline`)
//! inherits unchanged, plus the separate, unrelated 3D base `MultiPoint3`.
//!
//! **Why a trait, not a struct.** `Slic3r::MultiPoint` is an abstract C++ base: `points` is a
//! public data member every subclass inherits, `last_point()`/`lines()` are pure virtual (each
//! subclass's own shape), and every other method is concrete and shared verbatim by every
//! subclass. `docs/PORTING.md`'s idiom map, "Virtual dispatch over a class hierarchy" row:
//! *"a Rust trait with one implementing type per concrete subclass ... a port task does not
//! introduce a second dispatch mechanism for the same hierarchy."* [`MultiPoint`] below is that
//! trait: [`MultiPoint::points`]/[`MultiPoint::points_mut`]/[`MultiPoint::last_point`]/
//! [`MultiPoint::lines`] are its required methods (the inherited `points` field becomes a
//! required accessor pair, since Rust has no inherited struct field; the two pure virtuals stay
//! required), and every other method below is a trait *default* method, ported once, shared by
//! every implementor exactly as the C++ base shares it. `crates/slic3r-polygon/src/polygon.rs`
//! (this same task) is this trait's first implementor; a future `Polyline` task implements it the
//! same way, writing no line of this file (`grep -c "public MultiPoint" vendor/OrcaSlicer/src/
//! libslic3r/*.hpp` finds exactly `Polygon.hpp`/`Polyline.hpp`, confirming there are only ever
//! two). `MultiPoint`'s own four `static` methods (`_douglas_peucker`, `visivalingam`,
//! `concave_hull_2d`, `minimumDistanceBetweenLinesDefinedByPoints`) are not virtual at all in
//! C++, so they are plain module functions below, not trait members.
//!
//! `MultiPoint3` (`MultiPoint.hpp:126-175`) is an unrelated base for an unrelated 3D hierarchy
//! (`Polyline3`, `Polyline.hpp:297`, not this task's own file). Only its `lines()` is pure
//! virtual (`first_point`/`last_point` are concrete, non-virtual accessors on the base itself,
//! unlike `MultiPoint`'s own `last_point`), so [`MultiPoint3`] below needs only `points`/
//! `points_mut`/`lines` as required methods.
//!
//! **Deferred for the same reason `Point.rs`/`BoundingBox.rs` each defer their own cross-crate
//! methods -- needs a type from a crate this task does not own, or a dependency edge the cut
//! graph forbids, and this task's own rule forbids inventing one:**
//! - [`MultiPoint::has_boundary_point`] (`MultiPoint.cpp:94-98`) needs
//!   `Point::projection_onto(const MultiPoint&)`, which `crates/slic3r-point/src/point.rs`
//!   (task `56-foundation-point`, a sibling this task does not own) already left as
//!   `// TODO(port): src/libslic3r/Point.hpp:266` -- built before `MultiPoint` existed to depend
//!   on. This task cannot fill that gap by writing into `point.rs` ("Write only your own
//!   modules"), and duplicating `Point::projection_onto`'s body here under a different name would
//!   invent a second implementation of a method that properly belongs to `Point`, not to
//!   `MultiPoint` -- the same kind of workaround the rejection rule (`docs/WORKFLOW.md`) asks to
//!   be named rather than taken. Left as `// TODO(port):`, named in this task's pull request.
//!
//! **Closing a gap `BoundingBox.rs` named instead of filling** (that task's own words: *"this
//! unit's own file"*): [`get_extents`] (`MultiPoint.cpp:545-548`) and both
//! [`get_extents_rotated`]/[`get_extents_rotated_points`] overloads (`:550-579`) are
//! `MultiPoint.cpp`'s own rows, deferred by `crates/slic3r-boundingbox/src/bounding_box.rs`
//! (task `58-foundation-bounding-box`) only because that task does not own this file. They are
//! ported here, against their own manifest row, closing that named gap.
//!
//! **A disclosed finding, not a silent substitution: `MultiPoint::visivalingam`
//! (`MultiPoint.cpp:265-361`) is undefined behaviour at the pin, with no in-scope (or any)
//! caller anywhere in the vendored tree** (`grep -rn visivalingam vendor/OrcaSlicer/src/` finds
//! only this declaration and definition). `areas.reserve(pts.size())` (`:284-285`) only
//! reserves capacity -- the vector's length stays `0` -- and the loop body then writes
//! `areas[curr->pt_idx] = min_area` (`:332`) and `use_point` reads `areas[idx]` (`:348`) through
//! operator[], which is **unchecked** (not `.at()`) and so out-of-bounds UB for every `pts` of
//! two or more points, the only sizes the function's own leading `assert(pts.size() >= 2)`
//! (`:268`) admits. There is no defined oracle behaviour here to be held to exact equality
//! against -- a crashing or memory-garbage comparison is not a proof of anything. [`visivalingam`]
//! below ports the function's evident intent with the one-line fix the C++ needed
//! (`areas.resize(pts.size(), 0.0)` in place of `.reserve`, marked `// PORT NOTE:` at the site)
//! rather than reproducing a crash; it is exercised by no test and no fuzz target, and is named
//! as a finding in this task's pull request rather than silently fixed.
//!
//! `// PROOF: 1` proves the `MultiPoint.cpp` row (through `Slic3r::Polygon`, the only concrete,
//! instantiable subclass this task's own units provide -- `MultiPoint` itself has no public
//! constructor, being abstract). `// PROOF: 2` proves the `MultiPoint.hpp` row the same way, for
//! its own header-only methods.

use slic3r_boundingbox::bounding_box::{BoundingBox, BoundingBox3};
use slic3r_line::line::{Line, Lines};
use slic3r_point::point::{Point, Point3, Points, Points3, Vec3};

// -------------------------------------------------------------------------------------------
// MultiPoint (MultiPoint.hpp:19-124, MultiPoint.cpp)
// -------------------------------------------------------------------------------------------

/// `class MultiPoint { public: Points points; ... virtual const Point& last_point() const = 0;
/// virtual Lines lines() const = 0; ... };` (`MultiPoint.hpp:19-124`). See the module doc for why
/// this is a trait, not a struct: [`points`](MultiPoint::points)/
/// [`points_mut`](MultiPoint::points_mut) stand in for the inherited public field, and
/// [`last_point`](MultiPoint::last_point)/[`lines`](MultiPoint::lines) are the two pure virtuals.
/// Every other method is a default method below, ported once and shared by every implementor
/// unchanged, exactly as the C++ base shares its own concrete methods.
pub trait MultiPoint {
    /// The inherited `points` field, read-only.
    fn points(&self) -> &Points;
    /// The inherited `points` field, mutable.
    fn points_mut(&mut self) -> &mut Points;
    /// `virtual const Point& last_point() const = 0;` (`:48`).
    fn last_point(&self) -> Point;
    /// `virtual Lines lines() const = 0;` (`:49`).
    fn lines(&self) -> Lines;

    /// `void scale(double factor)` (`MultiPoint.cpp:17-21`): `pt *= factor` through
    /// [`Point::mul_assign`](slic3r_point::point::Point::mul_assign).
    fn scale(&mut self, factor: f64) {
        for pt in self.points_mut() {
            pt.mul_assign(factor);
        }
    }

    /// `void scale(double factor_x, double factor_y)` (`:23-30`): each component truncated
    /// toward zero independently, unlike the single-factor overload above -- `Point` exposes no
    /// mutable per-component indexing (`task 56-foundation-point`'s own scope: Eigen's `.x()`/
    /// `.y()` kept as read accessors only), so this reconstructs the point rather than indexing
    /// `pt(0)`/`pt(1)` in place, the simplest translation that keeps the same two independent
    /// truncating multiplications.
    fn scale_xy(&mut self, factor_x: f64, factor_y: f64) {
        for pt in self.points_mut() {
            *pt = Point::new(
                (pt.x() as f64 * factor_x) as i64,
                (pt.y() as f64 * factor_y) as i64,
            );
        }
    }

    /// `void translate(double x, double y) { this->translate(Point(coord_t(x), coord_t(y))); }`
    /// (`:38`): `Point(double, double)` here is the `coord_t(x)`/`coord_t(y)` C-style casts
    /// written at the call site, truncating toward zero -- **not**
    /// [`Point::from_f64_rounded`](slic3r_point::point::Point::from_f64_rounded)'s rounding
    /// constructor, which this call site does not use.
    fn translate_xy(&mut self, x: f64, y: f64) {
        self.translate(Point::new(x as i64, y as i64));
    }

    /// `void translate(const Point &v)` (`MultiPoint.cpp:32-36`).
    fn translate(&mut self, v: Point) {
        for pt in self.points_mut() {
            pt.add_assign(v);
        }
    }

    /// `void rotate(double angle) { this->rotate(cos(angle), sin(angle)); }` (`:40`).
    fn rotate(&mut self, angle: f64) {
        self.rotate_cos_sin(angle.cos(), angle.sin());
    }

    /// `void rotate(double cos_angle, double sin_angle)` (`MultiPoint.cpp:38-46`): `coord_t(round(...))`
    /// -- `std::round` (round-half-away-from-zero), matching
    /// [`Point::from_f64_rounded`](slic3r_point::point::Point::from_f64_rounded)'s own choice of
    /// rounding function, not [`fast_round_up`](slic3r_libslic3r::libslic3r::fast_round_up)'s.
    fn rotate_cos_sin(&mut self, cos_angle: f64, sin_angle: f64) {
        for pt in self.points_mut() {
            let cur_x = pt.x() as f64;
            let cur_y = pt.y() as f64;
            *pt = Point::new(
                (cos_angle * cur_x - sin_angle * cur_y).round() as i64,
                (cos_angle * cur_y + sin_angle * cur_x).round() as i64,
            );
        }
    }

    /// `void rotate(double angle, const Point &center)` (`MultiPoint.cpp:48-57`).
    fn rotate_about(&mut self, angle: f64, center: Point) {
        let s = angle.sin();
        let c = angle.cos();
        for pt in self.points_mut() {
            let v = *pt - center;
            *pt = Point::new(
                (center.x() as f64 + c * v.x() as f64 - s * v.y() as f64).round() as i64,
                (center.y() as f64 + c * v.y() as f64 + s * v.x() as f64).round() as i64,
            );
        }
    }

    /// `void reverse() { std::reverse(this->points.begin(), this->points.end()); }` (`:43`).
    fn reverse(&mut self) {
        self.points_mut().reverse();
    }

    /// `const Point& front() const { return this->points.front(); }` (`:45`).
    fn front(&self) -> Point {
        self.points()[0]
    }

    /// `const Point& back() const { return this->points.back(); }` (`:46`).
    fn back(&self) -> Point {
        let pts = self.points();
        pts[pts.len() - 1]
    }

    /// `const Point& first_point() const { return this->front(); }` (`:47`).
    fn first_point(&self) -> Point {
        self.front()
    }

    /// `size_t size() const { return points.size(); }` (`:50`).
    fn size(&self) -> usize {
        self.points().len()
    }

    /// `bool empty() const { return points.empty(); }` (`:51`).
    fn is_empty(&self) -> bool {
        self.points().is_empty()
    }

    /// `double length() const` (`MultiPoint.cpp:59-67`): sums each edge's own `Line::length()`,
    /// over `this->lines()` -- the one default method that calls the trait's own virtual
    /// [`lines`](MultiPoint::lines), so each implementor's `length()` differs only through its
    /// own `lines()` shape, never through a second copy of this summation.
    fn length(&self) -> f64 {
        let lines = self.lines();
        // `0f64`: the vendored `double len = 0;` (`MultiPoint.cpp:61,445`) spells its own
        // literal as the bare integer `0`; see the `find_point_eps`-style comment above for why
        // this stays exempt rather than taking a marker.
        let mut len = 0f64;
        for line in &lines {
            len += line.length();
        }
        len
    }

    /// `bool is_valid() const { return this->points.size() >= 2; }` (`:53`).
    fn is_valid(&self) -> bool {
        self.points().len() >= 2
    }

    /// `int find_point(const Point &point) const` (`MultiPoint.cpp:69-75`): `-1` if not found,
    /// else the index -- a plain linear scan, `int` kept as `i32` per the vendored return type.
    fn find_point(&self, point: Point) -> i32 {
        for (i, pt) in self.points().iter().enumerate() {
            if *pt == point {
                return i as i32;
            }
        }
        -1
    }

    /// `int find_point(const Point &point, double scaled_epsilon) const` (`:77-92`): nearest
    /// point within `scaled_epsilon`, or `-1`. `scaled_epsilon == 0` delegates to the exact
    /// overload above, exactly as the vendored body does.
    fn find_point_eps(&self, point: Point, scaled_epsilon: f64) -> i32 {
        // `0f64`, not `0.0`: the vendored comparison's own literal is the bare integer `0`
        // (`MultiPoint.cpp:79,622,647`), implicitly converted to `double`; `0f64` normalises to
        // the same exempt `"0"` text the vendored literal already is, so no marker is needed
        // (`docs/PORTING.md` decision 8's `f64`-suffix technique, used here to keep an already-
        // exempt value exempt rather than to transcribe one).
        if scaled_epsilon == 0f64 {
            return self.find_point(point);
        }
        let mut dist2_min = f64::MAX;
        let eps2 = scaled_epsilon * scaled_epsilon;
        let mut idx_min: i32 = -1;
        for (i, pt) in self.points().iter().enumerate() {
            let d2 = diff_squared_norm(*pt, point);
            if d2 < dist2_min {
                idx_min = i as i32;
                dist2_min = d2;
            }
        }
        if dist2_min < eps2 {
            idx_min
        } else {
            -1
        }
    }

    // TODO(port): src/libslic3r/MultiPoint.cpp:94-98 -- `bool has_boundary_point(const Point
    // &point) const`, needs `Point::projection_onto(const MultiPoint&)`, left unported in
    // `crates/slic3r-point/src/point.rs:554` (task `56-foundation-point`, built before
    // `MultiPoint` existed for it to depend on). See this module's own doc.

    /// `int closest_point_index(const Point &point) const` (`MultiPoint.hpp:62-76`, inline):
    /// `-1` on an empty sequence, else the index of the nearest point (ties keep the first,
    /// `<` not `<=`).
    fn closest_point_index(&self, point: Point) -> i32 {
        let pts = self.points();
        if pts.is_empty() {
            return -1;
        }
        let mut idx: i32 = 0;
        let mut dist_min = diff_norm(point, pts[0]);
        for (i, pt) in pts.iter().enumerate().skip(1) {
            let d = diff_norm(*pt, point);
            if d < dist_min {
                dist_min = d;
                idx = i as i32;
            }
        }
        idx
    }

    /// `const Point* closest_point(const Point &point) const` (`:77`, inline): `None` on an
    /// empty sequence (the vendored `nullptr`), else a copy of the nearest point (`Point` is
    /// `Copy`, so a borrowed `*` return has no Rust counterpart worth keeping).
    fn closest_point(&self, point: Point) -> Option<Point> {
        if self.points().is_empty() {
            None
        } else {
            let idx = self.closest_point_index(point);
            Some(self.points()[idx as usize])
        }
    }

    /// `double distance_to(const Point& point) const` (`:78-83`, inline): the distance to
    /// [`closest_point`](MultiPoint::closest_point) -- the vendored body dereferences its
    /// `nullptr` case unconditionally (no empty check), which this port does not reproduce as a
    /// panic-equivalent; no in-scope caller reaches this on an empty sequence.
    fn distance_to(&self, point: Point) -> f64 {
        let cl = self.closest_point(point).unwrap_or(point);
        diff_norm(cl, point)
    }

    /// `BoundingBox bounding_box() const { return BoundingBox(this->points); }`
    /// (`MultiPoint.cpp:100-103`).
    fn bounding_box(&self) -> BoundingBox {
        BoundingBox::from_points(self.points())
    }

    /// `bool has_duplicate_points() const` (`:105-111`): adjacent duplicates only (the vendored
    /// body's own scan, not a global dedup).
    fn has_duplicate_points(&self) -> bool {
        let pts = self.points();
        for i in 1..pts.len() {
            if pts[i - 1] == pts[i] {
                return true;
            }
        }
        false
    }

    /// `bool remove_duplicate_points()` (`:113-130`): collapses adjacent duplicates in place,
    /// returning whether anything was removed.
    fn remove_duplicate_points(&mut self) -> bool {
        let pts = self.points_mut();
        if pts.len() < 2 {
            return false;
        }
        let mut j = 0usize;
        for i in 1..pts.len() {
            if pts[j] == pts[i] {
                // Just increase index i.
            } else {
                j += 1;
                if j < i {
                    pts[j] = pts[i];
                }
            }
        }
        j += 1;
        if j < pts.len() {
            pts.truncate(j);
            true
        } else {
            false
        }
    }

    /// `void clear() { this->points.clear(); }` (`:89`).
    fn clear(&mut self) {
        self.points_mut().clear();
    }

    /// `void append(const Point &point) { this->points.push_back(point); }` (`:90`).
    fn append(&mut self, point: Point) {
        self.points_mut().push(point);
    }

    /// `void append(const Points &src)` (`:91`, through the iterator-pair overload at `:92`).
    fn append_points(&mut self, src: &Points) {
        self.points_mut().extend_from_slice(src);
    }

    /// `void append(Points &&src)` (`:93-101`): the move overload swaps in `src` when `this` is
    /// empty, else appends and clears `src` -- ported with the same two-branch shape rather than
    /// collapsed to a plain `extend`, since the vendored body's own branch is what avoids copying
    /// `src`'s elements in the empty case (Rust's `Vec::append` gives the identical effect; kept
    /// explicit to mirror the vendored branch, not for a performance reason of its own).
    fn append_moved(&mut self, mut src: Points) {
        let pts = self.points_mut();
        if pts.is_empty() {
            *pts = src;
        } else {
            pts.append(&mut src);
        }
    }

    /// `bool intersection(const Line& line, Point* intersection) const` (`MultiPoint.cpp:132-139`):
    /// the out-parameter becomes the return value per the idiom map's "non-const reference
    /// out-parameter" row.
    fn intersection(&self, line: &Line) -> Option<Point> {
        for l in self.lines() {
            if let Some(p) = l.intersection(line) {
                return Some(p);
            }
        }
        None
    }

    /// `bool first_intersection(const Line& line, Point* intersection) const` (`:141-162`): the
    /// *nearest* (to `line.a`) intersection over every edge, not merely the first found.
    fn first_intersection(&self, line: &Line) -> Option<Point> {
        let mut found: Option<Point> = None;
        // PORT SRC: src/libslic3r/MultiPoint.cpp:144
        let mut dmin = 0.;
        for l in self.lines() {
            if let Some(ip) = l.intersection(line) {
                let d = diff_norm(line.a, ip);
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
        }
        found
    }

    /// `bool intersections(const Line &line, Points *intersections) const` (`:164-173`): the
    /// vendored signature appends to a caller-owned, possibly non-empty vector and returns
    /// whether its size grew; no in-scope caller seeds a non-empty vector, so this returns just
    /// the newly found intersections -- a non-empty result is exactly the vendored `true` case.
    fn intersections(&self, line: &Line) -> Points {
        let mut out = Points::new();
        for l in self.lines() {
            if let Some(p) = l.intersection(line) {
                out.push(p);
            }
        }
        out
    }

    /// `static Points _douglas_peucker(const Points &points, const double tolerance);`
    /// (`:107`, defined `MultiPoint.cpp:175-241`) is **not** a trait member here: a C++ `static`
    /// member is never virtual, so it carries no per-subclass dispatch to port -- it is
    /// [`douglas_peucker`] below, a plain module function, called the same way `Polygon::simplify`/
    /// `douglas_peucker` call `MultiPoint::_douglas_peucker` directly in the vendored source.
    ///
    /// `void symmetric_y(const coord_t &x_axis)` (`:581-586`; the header spells the same
    /// parameter `y_axis` at its declaration, `:106` -- the two names disagree and the
    /// implementation's own `x_axis` is what the body actually uses, mirroring the X coordinate
    /// about the line `x = x_axis`).
    fn symmetric_y(&mut self, x_axis: i64) {
        for pt in self.points_mut() {
            // PORT SRC: src/libslic3r/MultiPoint.cpp:584 -- `wrapping_*`, not plain `*`/`-`: the
            // vendored `coord_t` (`int64_t`) arithmetic itself overflows for an `x_axis`/`pt.x()`
            // pair large enough that `2 * x_axis` or the subtraction passes `i64::MAX`/`MIN`,
            // which is a deterministic two's-complement wrap in the compiled oracle rather than a
            // panic -- the same reasoning `crates/slic3r-polygon/src/polygon.rs`'s
            // `polygon_is_convex` doc already states for the identical shape.
            *pt = Point::new(2i64.wrapping_mul(x_axis).wrapping_sub(pt.x()), pt.y());
        }
    }
}

/// `(a - b).cast<double>().norm()`, this module's own helper since `Point` exposes no `.cast()`/
/// `.norm()` of its own (`crates/slic3r-point/src/point.rs`'s module doc names exactly this
/// scope line).
fn diff_norm(a: Point, b: Point) -> f64 {
    diff_squared_norm(a, b).sqrt()
}

/// `(a - b).cast<double>().squaredNorm()`; see [`diff_norm`]'s doc.
fn diff_squared_norm(a: Point, b: Point) -> f64 {
    let d = a - b;
    let dx = d.x() as f64;
    let dy = d.y() as f64;
    dx * dx + dy * dy
}

/// `(a - b).norm()` with **no** `.cast<double>()` -- Eigen's `norm()` on an integer-typed vector
/// (`Vec2crd`/`Point`, `int64_t` components) returns `NumTraits<int64_t>::Real`, i.e. `int64_t`:
/// `numext::sqrt(squaredNorm())`. `squaredNorm()` itself is **exact** `int64_t` arithmetic
/// (`wrapping_*`, same reason as [`squared_distance_to_line_segment`]'s own `dot`/`l2`) --
/// unlike [`diff_squared_norm`], which widens each coordinate to `f64` *before* squaring and so
/// double-rounds (correct only where the vendored call site itself carries `.cast<double>()`,
/// which first casts to `double` and so makes the two paths equivalent -- not the case here).
/// `numext::sqrt` then converts that one exact integer to `double`, takes the true square root,
/// and narrows back to `int64_t` by the function's own return type -- a single widen-then-sqrt-
/// then-truncate, not a dedicated integer-square-root algorithm. The one call site that needs
/// this is [`concave_hull_2d`]'s own divisor (`MultiPoint.cpp:372`); every other `norm()`/
/// `squaredNorm()` call in this unit carries the vendored `.cast<double>()` and stays
/// [`diff_norm`]/[`diff_squared_norm`].
fn diff_norm_eigen_int_truncated(a: Point, b: Point) -> f64 {
    let d = a - b;
    let sq = (d.x().wrapping_mul(d.x())).wrapping_add(d.y().wrapping_mul(d.y()));
    ((sq as f64).sqrt() as i64) as f64
}

/// `static Points _douglas_peucker(const Points &pts, const double tolerance);`
/// (`MultiPoint.hpp:107`, defined `MultiPoint.cpp:175-241`): the Douglas-Peucker polyline
/// simplification, over an explicit stack rather than recursion, exactly as the vendored body is
/// structured. Debug-only `#if 0` SVG dump (`:222-238`) is not ported (don't-translate list).
pub fn douglas_peucker(pts: &Points, tolerance: f64) -> Points {
    let mut result_pts = Points::new();
    let tolerance_sq = tolerance * tolerance;
    if !pts.is_empty() {
        let mut anchor_idx = 0usize;
        let mut anchor = pts[0];
        let mut floater_idx = pts.len() - 1;
        let mut floater = pts[floater_idx];
        result_pts.reserve(pts.len());
        result_pts.push(anchor);
        if anchor_idx != floater_idx {
            let mut dp_stack: Vec<usize> = Vec::with_capacity(pts.len());
            dp_stack.push(floater_idx);
            loop {
                // PORT SRC: src/libslic3r/MultiPoint.cpp:192
                let mut max_dist_sq = 0.0;
                let mut furthest_idx = anchor_idx;
                for i in (anchor_idx + 1)..floater_idx {
                    let dist_sq = Line::distance_to_squared_static(pts[i], anchor, floater);
                    if dist_sq > max_dist_sq {
                        max_dist_sq = dist_sq;
                        furthest_idx = i;
                    }
                }
                if max_dist_sq <= tolerance_sq {
                    result_pts.push(floater);
                    anchor_idx = floater_idx;
                    anchor = floater;
                    dp_stack.pop();
                    match dp_stack.last() {
                        Some(&top) => floater_idx = top,
                        None => break,
                    }
                } else {
                    floater_idx = furthest_idx;
                    dp_stack.push(floater_idx);
                }
                floater = pts[floater_idx];
            }
        }
    }
    result_pts
}

/// `_douglas_peucker` over `Points3`/`Line3`, `MultiPoint3::_douglas_peucker`
/// (`MultiPoint.hpp:174`, defined `MultiPoint.cpp:477-543`); see [`douglas_peucker`]'s doc for
/// the shared structure.
pub fn douglas_peucker3(pts: &Points3, tolerance: f64) -> Points3 {
    use slic3r_line::line::Line3;
    let mut result_pts = Points3::new();
    let tolerance_sq = tolerance * tolerance;
    if !pts.is_empty() {
        let mut anchor_idx = 0usize;
        let mut anchor = pts[0];
        let mut floater_idx = pts.len() - 1;
        let mut floater = pts[floater_idx];
        result_pts.reserve(pts.len());
        result_pts.push(anchor);
        if anchor_idx != floater_idx {
            let mut dp_stack: Vec<usize> = Vec::with_capacity(pts.len());
            dp_stack.push(floater_idx);
            loop {
                // PORT SRC: src/libslic3r/MultiPoint.cpp:494
                let mut max_dist_sq = 0.0;
                let mut furthest_idx = anchor_idx;
                for i in (anchor_idx + 1)..floater_idx {
                    let dist_sq = Line3::distance_to_squared(pts[i], anchor, floater);
                    if dist_sq > max_dist_sq {
                        max_dist_sq = dist_sq;
                        furthest_idx = i;
                    }
                }
                if max_dist_sq <= tolerance_sq {
                    result_pts.push(floater);
                    anchor_idx = floater_idx;
                    anchor = floater;
                    dp_stack.pop();
                    match dp_stack.last() {
                        Some(&top) => floater_idx = top,
                        None => break,
                    }
                } else {
                    floater_idx = furthest_idx;
                    dp_stack.push(floater_idx);
                }
                floater = pts[floater_idx];
            }
        }
    }
    result_pts
}

/// One heap entry for `MultiPoint::visivalingam`'s own binary heap (`MultiPoint.cpp:254-264`,
/// `vis_node`): sorted "greater area = more important", the same field the vendored
/// `vis_node::operator<` reads. Rust's `BinaryHeap` is already a max-heap over `Ord`, so popping
/// gives the vendored `std::make_heap`/`pop_heap` family's own "largest area first" order; `idx`
/// is carried as an explicit, deterministic tie-breaker and as the key this port uses for lazy
/// deletion (a stale entry -- superseded by a later push for the same `idx` after a neighbour's
/// area was recomputed -- is detected by comparing against `area[idx]`'s current value, checked
/// where entries are popped below), since a `vis_node*` pointer identity has no Rust counterpart
/// for a value-typed heap entry.
// FIDELITY(port): two divergences from the vendored `vis_node`/heap shape, not one. First, the
// restructuring: `MultiPoint.cpp:318-331` mutates a neighbour's `vis_node::area` *in place*
// through its `vis_node*` pointer and then calls `std::make_heap` again over the same storage, so
// each index holds exactly one live heap entry at all times; this port instead pushes a new
// `VisHeapEntry` every time an index's area is recomputed (`:654-665` below) and treats an older
// entry for the same `idx` as stale by comparing it against `area[idx]`'s current value at pop
// time (`:644` below) -- lazy deletion in place of true in-place update, because a value-typed
// Rust heap has no pointer identity to mutate through. Second, and only within that
// restructuring: this port's heap breaks a tie between two equal-area entries by `idx`, a total
// order; the vendored `std::make_heap`/`pop_heap` family breaks the same tie by heap position,
// which depends on push/pop history and so is not reproducible without reimplementing
// libstdc++'s own heap algorithm. `visivalingam` has no in-scope caller and no row of its own in
// `port/reports/order-comparators.md` (that report covers only MultiPoint.cpp's in-scope
// callers' comparators), so this divergence is recorded here rather than in that report.
#[derive(PartialEq)]
struct VisHeapEntry {
    area: f64,
    idx: usize,
}
impl Eq for VisHeapEntry {}
impl PartialOrd for VisHeapEntry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for VisHeapEntry {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.area
            .total_cmp(&other.area)
            .then(self.idx.cmp(&other.idx))
    }
}

/// `static Points visivalingam(const Points& pts, const double tolerance);` (`MultiPoint.hpp:108`,
/// defined `MultiPoint.cpp:265-361`). **Not held to any oracle comparison**: see this module's
/// own doc for why (the vendored body is undefined behaviour at the pin, reached by no in-scope
/// or any caller).
pub fn visivalingam(pts: &Points, tolerance: f64) -> Points {
    // PORT SRC: src/libslic3r/MultiPoint.cpp:268 -- an `assert` body (`docs/PORTING.md`'s
    // Don't-translate list; its own cross-reference, `### Error sets`, gives the concrete
    // rendering: "becomes `debug_assert!` and never a live `panic!`"), so this is
    // `debug_assert!`, never a live `assert!`/`panic!` the oracle's NDEBUG release build has
    // compiled out.
    debug_assert!(pts.len() >= 2);
    let n = pts.len();
    let effective_area = |curr: usize, prev: usize, next: usize| -> f64 {
        let c = pts[curr];
        let p = pts[prev];
        let nx = pts[next];
        let curr_to_next_x = (nx.x() - c.x()) as f64;
        let curr_to_next_y = (nx.y() - c.y()) as f64;
        let prev_to_next_x = (p.x() - c.x()) as f64;
        let prev_to_next_y = (p.y() - c.y()) as f64;
        // PORT SRC: src/libslic3r/MultiPoint.cpp:281
        0.50 * (curr_to_next_x * prev_to_next_y - curr_to_next_y * prev_to_next_x).abs()
    };

    // `node_list[0]`/`node_list[n-1]` are never created in the vendored body (the loop below
    // runs `i = 1 .. pts.size() - 1`), so both endpoints start "removed" here -- the same
    // meaning the vendored `nullptr` check at `:319,327` reads for them.
    let mut prev_idx: Vec<usize> = (0..n).collect();
    let mut next_idx: Vec<usize> = (0..n).collect();
    // PORT SRC: external: this port's own per-index area cache, not a value transcribed from
    // any single vendored literal -- the vendored `vis_node::area` field has no zero-initialiser
    // of its own (every `vis_node` is constructed with a real, computed area at `:296`).
    let mut area: Vec<f64> = vec![0.0; n];
    let mut removed = vec![true; n];
    // PORT NOTE: src/libslic3r/MultiPoint.cpp:284-285 -- the vendored `areas.reserve(pts.size())`
    // leaves `areas` at length 0 and is undefined behaviour the first time the loop below writes
    // `areas[idx]`; this port sizes `areas_out` to `n` up front, the evident fix for an
    // unreachable function (see this module's own doc).
    // PORT SRC: external: `0.0`'s own value substitutes for the vendored `areas` vector's
    // undefined (never-initialised) element value at this same site -- not a value transcribed
    // from any defined vendored literal, since no defined value exists there to transcribe.
    let mut areas_out: Vec<f64> = vec![0.0; n];

    let mut heap: std::collections::BinaryHeap<VisHeapEntry> = std::collections::BinaryHeap::new();
    for i in 1..n.saturating_sub(1) {
        prev_idx[i] = i - 1;
        next_idx[i] = i + 1;
        let a = effective_area(i, i - 1, i + 1);
        area[i] = a;
        removed[i] = false;
        heap.push(VisHeapEntry { area: a, idx: i });
    }

    let mut min_area = f64::MIN;
    while let Some(top) = heap.pop() {
        if removed[top.idx] || area[top.idx].total_cmp(&top.area) != std::cmp::Ordering::Equal {
            // Stale: this node was already removed, or its area was recomputed and re-pushed
            // since this entry was queued.
            continue;
        }
        let idx = top.idx;
        removed[idx] = true;
        min_area = min_area.max(area[idx]);
        let p = prev_idx[idx];
        let nx = next_idx[idx];
        if !removed[p] {
            next_idx[p] = nx;
            let a = effective_area(p, prev_idx[p], next_idx[p]);
            area[p] = a;
            heap.push(VisHeapEntry { area: a, idx: p });
        }
        if !removed[nx] {
            prev_idx[nx] = p;
            let a = effective_area(nx, prev_idx[nx], next_idx[nx]);
            area[nx] = a;
            heap.push(VisHeapEntry { area: a, idx: nx });
        }
        areas_out[idx] = min_area;
    }

    let use_point = |idx: usize| -> bool {
        if idx == 0 || idx == n - 1 {
            true
        } else {
            areas_out[idx] > tolerance
        }
    };
    let mut results = Points::new();
    for i in 0..n {
        if use_point(i) {
            results.push(pts[i]);
        }
    }
    results
}

/// `static Points concave_hull_2d(const Points& pts, const double tolerence);`
/// (`MultiPoint.hpp:109`, defined `MultiPoint.cpp:364-381`). No in-scope caller
/// (`grep -rn concave_hull_2d vendor/OrcaSlicer/src/` finds only this unit), but -- unlike
/// `visivalingam` -- this body has no undefined behaviour, so it is ported plainly.
pub fn concave_hull_2d(pts: &Points, tolerance: f64) -> Points {
    let n = pts.len();
    let mut hull = Points::new();
    // PORT SRC: src/libslic3r/MultiPoint.cpp:368
    if n >= 3 {
        let mut k = 0usize;
        hull.resize(n, Point::zero());
        for i in 0..n {
            while k >= 2
                // `0.0`'s own text matches the vendored `<= 0` only at its value, not its
                // spelling (`MultiPoint.cpp:372`'s own literal is the bare integer `0`); `<=`
                // against `ccw`'s `f64` result needs a float-typed literal, and `0.0`/`0f64`
                // both normalise to the exempt `"0"` either way -- see the `find_point_eps`
                // comment above for the same technique.
                && pts[i].ccw(hull[k - 2], hull[k - 1]) <= 0f64
                // `(pts[i] - hull[k - 1]).norm()` (`MultiPoint.cpp:372`) carries no
                // `.cast<double>()`, unlike every other `norm()`/`squaredNorm()` call in this
                // unit's own `f64` arithmetic -- the operand is `Eigen::Matrix<int64_t, 2, 1>`,
                // so `norm()` is `numext::sqrt(squaredNorm())` returning
                // `NumTraits<int64_t>::Real`, i.e. `int64_t`: the true square root truncated back
                // to an integer by the return type, not the exact `f64` `diff_norm` the rest of
                // this module uses. `diff_norm_eigen_int_truncated` reproduces that truncation.
                && -pts[i].ccw(hull[k - 2], hull[k - 1])
                    / diff_norm_eigen_int_truncated(pts[i], hull[k - 1])
                    < tolerance
            {
                k -= 1;
            }
            hull[k] = pts[i];
            k += 1;
        }
        hull.truncate(k);
        if hull.first() == hull.last() {
            hull.pop();
        }
    }
    hull
}

/// `static double minimumDistanceBetweenLinesDefinedByPoints(const Points& A, const Points& B);`
/// (`MultiPoint.hpp:112`, defined `MultiPoint.cpp:414-434`), an Orca addition (comment: "IOI wall
/// ordering algorithm for arachne"). No in-scope caller found
/// (`grep -rn minimumDistanceBetweenLinesDefinedByPoints vendor/OrcaSlicer/src/` finds only this
/// unit), ported plainly -- unlike `A.size() - 1`'s C++ `size_t` wraparound on an empty `A`
/// (also present for `B`), Rust's debug-mode `usize` subtraction panics instead of wrapping; no
/// caller passes an empty slice today, so this divergence is disclosed rather than guarded
/// against with no vendored counterpart to guard against it with.
pub fn minimum_distance_between_lines_defined_by_points(a: &Points, b: &Points) -> f64 {
    let mut min_distance = f64::INFINITY;
    for i in 0..(a.len() - 1) {
        for &bp in b {
            let distance = squared_distance_to_line_segment(bp, a[i], a[i + 1]);
            min_distance = min_distance.min(distance.sqrt());
        }
    }
    for i in 0..(b.len() - 1) {
        for &ap in a {
            let distance = squared_distance_to_line_segment(ap, b[i], b[i + 1]);
            min_distance = min_distance.min(distance.sqrt());
        }
    }
    min_distance
}

/// `private: static double squaredDistanceToLineSegment(const Point& p, const Point& v, const
/// Point& w);` (`MultiPoint.hpp:123`, defined `MultiPoint.cpp:393-404`): `private:` and reachable
/// only transitively through
/// [`minimum_distance_between_lines_defined_by_points`], the same "reachable only through its
/// public caller" shape `cxx/proof_units/instance_lock.cpp`'s header comment describes for
/// `InstanceLock::open_lock_file` -- kept module-private here too (idiom map: a `private` free/
/// static function is a non-`pub` module function).
fn squared_distance_to_line_segment(p: Point, v: Point, w: Point) -> f64 {
    // `double l2 = (v - w).squaredNorm();` (`MultiPoint.cpp:397`): **no** `.cast<double>()`
    // -- `v - w` is `Eigen::Matrix<int64_t, 2, 1>`, so this `squaredNorm()` is exact `int64_t`
    // arithmetic (`wrapping_*`, same reason as `dot` below), converted to `double` only once, at
    // this assignment -- not [`diff_squared_norm`]'s "widen each coordinate to `f64`, then
    // square" (which *this* module's own other call sites use precisely because their vendored
    // counterparts do carry `.cast<double>()`). The two are not interchangeable here: for
    // coordinates whose squared sum exceeds `f64`'s 53-bit exact-integer range, squaring each
    // `f64`-widened coordinate separately and then adding rounds twice, which can differ from
    // converting the one exact `int64_t` sum to `f64` a single time.
    let dvw = v - w;
    let l2 = (dvw.x().wrapping_mul(dvw.x())).wrapping_add(dvw.y().wrapping_mul(dvw.y())) as f64;
    // PORT SRC: src/libslic3r/MultiPoint.cpp:397
    if l2 == 0.0 {
        // `(p - v).squaredNorm()` (`:398`): the same uncast, exact `int64_t` squared norm as
        // `l2` above, not [`diff_squared_norm`].
        let dpv = p - v;
        return (dpv.x().wrapping_mul(dpv.x())).wrapping_add(dpv.y().wrapping_mul(dpv.y())) as f64;
    }
    let pv = p - v;
    let wv = w - v;
    // `(p - v).dot(w - v)` (`MultiPoint.cpp:399`): an exact `int64_t` dot product in the vendored
    // Eigen expression (no `.cast<double>()`); `wrapping_*`, not plain `*`/`+`, for the same
    // overflow-vs-panic reason `crates/slic3r-polygon/src/polygon.rs`'s `polygon_is_convex` doc
    // states for its own `int64_t` cross product.
    let dot = (pv.x().wrapping_mul(wv.x())).wrapping_add(pv.y().wrapping_mul(wv.y())) as f64;
    // PORT SRC: src/libslic3r/MultiPoint.cpp:399
    let t = 0.0_f64.max(1.0_f64.min(dot / l2));
    // `Point projection{v.x() + t * (w.x() - v.x()), v.y() + t * (w.y() - v.y())};`
    // (`MultiPoint.cpp:401`): both initialisers have type `double` (`int64_t + double`), so
    // overload resolution selects the *rounding* `Point(double, double)` constructor
    // (`Point.hpp:208`), not an unrounded `f64` point -- the same rounding
    // `crates/slic3r-polygon/src/polygon.rs`'s `densify` already reproduces with
    // `Point::from_f64_rounded` for the identical constructor shape.
    let proj = Point::from_f64_rounded(
        v.x() as f64 + t * (w.x() - v.x()) as f64,
        v.y() as f64 + t * (w.y() - v.y()) as f64,
    );
    // `(p - projection).squaredNorm()` (`MultiPoint.cpp:403`): `projection` is now an integer
    // `Point`, so this squared norm is exact `int64_t` arithmetic -- `wrapping_*`, matching `dot`
    // above, not the `f64`-widening [`diff_squared_norm`] every other call in this unit uses on a
    // genuinely `f64` difference.
    let d = p - proj;
    (d.x().wrapping_mul(d.x())).wrapping_add(d.y().wrapping_mul(d.y())) as f64
}

/// `extern BoundingBox get_extents(const MultiPoint &mp);` (`MultiPoint.hpp:177`, defined
/// `MultiPoint.cpp:545-548`): `BoundingBox(mp.points)`, generic over any [`MultiPoint`]
/// implementor through the trait's own [`points`](MultiPoint::points) accessor -- the public
/// field access the vendored free function reads directly needs no virtual dispatch of its own.
pub fn get_extents<M: MultiPoint + ?Sized>(mp: &M) -> BoundingBox {
    BoundingBox::from_points(mp.points())
}

/// `extern BoundingBox get_extents_rotated(const Points &points, double angle);`
/// (`:178`, defined `MultiPoint.cpp:550-574`).
pub fn get_extents_rotated_points(points: &Points, angle: f64) -> BoundingBox {
    let mut bbox = BoundingBox::new();
    if !points.is_empty() {
        let s = angle.sin();
        let c = angle.cos();
        let mut it = points.iter();
        let first = it.next().unwrap();
        let cur_x = first.x() as f64;
        let cur_y = first.y() as f64;
        let min_x = (c * cur_x - s * cur_y).round() as i64;
        let min_y = (c * cur_y + s * cur_x).round() as i64;
        let mut min = Point::new(min_x, min_y);
        let mut max = min;
        for pt in it {
            let cur_x = pt.x() as f64;
            let cur_y = pt.y() as f64;
            let x = (c * cur_x - s * cur_y).round() as i64;
            let y = (c * cur_y + s * cur_x).round() as i64;
            min = Point::new(min.x().min(x), min.y().min(y));
            max = Point::new(max.x().max(x), max.y().max(y));
        }
        // PORT SRC: src/libslic3r/MultiPoint.cpp:571 -- the vendored body default-constructs
        // `bbox` (`defined = false`), assigns `min`/`max` componentwise, then sets
        // `bbox.defined = true` unconditionally here, for *any* non-empty `points` -- including a
        // single point or a rotated extent that is degenerate (zero-width or zero-height) in x
        // or y. `BoundingBox::from_min_max` ports a *different* vendored constructor
        // (`BoundingBox.hpp:25-26`, correctly strict-`<`, see that function's own doc) and is not
        // this function's own assignment, so this constructs the box directly rather than
        // borrowing that constructor's rule.
        bbox = BoundingBox {
            min,
            max,
            defined: true,
        };
    }
    bbox
}

/// `extern BoundingBox get_extents_rotated(const MultiPoint &mp, double angle);`
/// (`:179`, defined `MultiPoint.cpp:576-579`).
pub fn get_extents_rotated<M: MultiPoint + ?Sized>(mp: &M, angle: f64) -> BoundingBox {
    get_extents_rotated_points(mp.points(), angle)
}

/// `inline double length(const Points &pts)` (`MultiPoint.hpp:181-189`): the free function, over
/// a bare `Points`, distinct from [`MultiPoint::length`]'s own (over `this->lines()`).
pub fn length(pts: &Points) -> f64 {
    // `0f64`: the vendored `double total = 0;` (`MultiPoint.hpp:182`) spells its own literal as
    // the bare integer `0`; see the `find_point_eps` comment above for why this stays exempt.
    let mut total = 0f64;
    if !pts.is_empty() {
        for i in 1..pts.len() {
            total += diff_norm(pts[i], pts[i - 1]);
        }
    }
    total
}

/// `inline double area(const Points &polygon)` (`MultiPoint.hpp:191-196`): a different formula
/// from [`crates::slic3r-polygon`]'s own `Polygon::area` (no `0.5` factor here) -- the two are
/// separate vendored functions, ported separately rather than one reused for the other.
pub fn area(polygon: &Points) -> f64 {
    // PORT SRC: src/libslic3r/MultiPoint.hpp:192
    let mut area = 0.;
    let n = polygon.len();
    if n == 0 {
        return area;
    }
    let mut j = n - 1;
    for i in 0..n {
        area += (polygon[i].x() + polygon[j].x()) as f64 * (polygon[i].y() - polygon[j].y()) as f64;
        j = i;
    }
    area
}

// -------------------------------------------------------------------------------------------
// MultiPoint3 (MultiPoint.hpp:126-175, MultiPoint.cpp)
// -------------------------------------------------------------------------------------------

/// `class MultiPoint3 { public: Points3 points; ... virtual Lines3 lines() const = 0; ... };`
/// (`MultiPoint.hpp:126-175`). Only `lines()` is pure virtual here (unlike [`MultiPoint`]'s own
/// `last_point`, this base's `first_point`/`last_point` are concrete, non-virtual accessors on
/// `points` itself) -- see the module doc.
pub trait MultiPoint3 {
    /// The inherited `points` field, read-only.
    fn points(&self) -> &Points3;
    /// The inherited `points` field, mutable.
    fn points_mut(&mut self) -> &mut Points3;
    /// `virtual Lines3 lines() const = 0;` (`:158`).
    fn lines(&self) -> slic3r_line::line::Lines3;

    /// `void append(const Point3& point) { this->points.push_back(point); }` (`:131`).
    fn append(&mut self, point: Point3) {
        self.points_mut().push(point);
    }

    /// `void translate(double x, double y, double z = 0)` (`:138`).
    fn translate_xyz(&mut self, x: f64, y: f64, z: f64) {
        self.translate(Point3::new(x as i64, y as i64, z as i64));
    }

    /// `void translate(const Point3& vector)` (`MultiPoint.cpp:436-440`).
    fn translate(&mut self, v: Point3) {
        for pt in self.points_mut() {
            pt.add_assign(v);
        }
    }

    /// `void reverse() { std::reverse(this->points.begin(), this->points.end()); }` (`:140`).
    fn reverse(&mut self) {
        self.points_mut().reverse();
    }

    /// `void rotate(double angle) { this->rotate(cos(angle), sin(angle)); }` (`:141`).
    fn rotate(&mut self, angle: f64) {
        self.rotate_cos_sin(angle.cos(), angle.sin());
    }

    /// `void rotate(double cos_angle, double sin_angle)` (`MultiPoint.cpp:589-598`): `x`/`y`
    /// rotated in-plane, `z` kept unchanged.
    fn rotate_cos_sin(&mut self, cos_angle: f64, sin_angle: f64) {
        for pt in self.points_mut() {
            let cur_x = pt.x() as f64;
            let cur_y = pt.y() as f64;
            *pt = Point3::new(
                (cos_angle * cur_x - sin_angle * cur_y).round() as i64,
                (cos_angle * cur_y + sin_angle * cur_x).round() as i64,
                pt.z(),
            );
        }
    }

    /// `void rotate(double angle, const Point3 &center)` (`MultiPoint.cpp:600-610`): `z` kept
    /// unchanged, same as the overload above.
    fn rotate_about(&mut self, angle: f64, center: Point3) {
        let s = angle.sin();
        let c = angle.cos();
        for pt in self.points_mut() {
            let v = *pt - center;
            *pt = Point3::new(
                (center.x() as f64 + c * v.x() as f64 - s * v.y() as f64).round() as i64,
                (center.y() as f64 + c * v.y() as f64 + s * v.x() as f64).round() as i64,
                pt.z(),
            );
        }
    }

    /// `Point3& first_point() { return this->points.front(); }` / its `const` overload
    /// (`:145,147`): the mutable overload has no in-scope caller, so only the `const` shape is
    /// ported.
    fn first_point(&self) -> Point3 {
        self.points()[0]
    }

    /// `const Point3& last_point() const { return this->points.back(); }` (`:148`); see
    /// [`first_point`](MultiPoint3::first_point)'s doc for why only the `const` overload is
    /// ported.
    fn last_point(&self) -> Point3 {
        let pts = self.points();
        pts[pts.len() - 1]
    }

    /// `size_t size() const { return this->points.size(); }` (`:149`).
    fn size(&self) -> usize {
        self.points().len()
    }

    /// `bool empty() const { return this->points.empty(); }` (`:150`).
    fn is_empty(&self) -> bool {
        self.points().is_empty()
    }

    /// `void clear() { this->points.clear(); }` (`:151`).
    fn clear(&mut self) {
        self.points_mut().clear();
    }

    /// `double length() const` (`MultiPoint.cpp:442-450`); see [`MultiPoint::length`]'s doc for
    /// the shared shape.
    fn length(&self) -> f64 {
        let lines = self.lines();
        // `0f64`: the vendored `double len = 0;` (`MultiPoint.cpp:61,445`) spells its own
        // literal as the bare integer `0`; see the `find_point_eps`-style comment above for why
        // this stays exempt rather than taking a marker.
        let mut len = 0f64;
        for line in &lines {
            len += line.length();
        }
        len
    }

    /// `bool is_valid() const { return this->points.size() >= 2; }` (`:160`).
    fn is_valid(&self) -> bool {
        self.points().len() >= 2
    }

    /// `BoundingBox3 bounding_box() const { return BoundingBox3(this->points); }`
    /// (`MultiPoint.cpp:452`).
    fn bounding_box(&self) -> BoundingBox3 {
        let pts: Vec<Vec3<i64>> = self
            .points()
            .iter()
            .map(|p| Vec3::new(p.x(), p.y(), p.z()))
            .collect();
        BoundingBox3::from_points3(&pts)
    }

    /// `int find_point(const Point &point) const` (`:165`, defined `MultiPoint.cpp:612-618`):
    /// compares through `Point3::to_point()`.
    fn find_point(&self, point: Point) -> i32 {
        for (i, pt) in self.points().iter().enumerate() {
            if pt.to_point() == point {
                return i as i32;
            }
        }
        -1
    }

    /// `int find_point(const Point &point, double scaled_epsilon) const` (`:166`, defined
    /// `MultiPoint.cpp:620-635`).
    fn find_point_eps(&self, point: Point, scaled_epsilon: f64) -> i32 {
        // `0f64`, not `0.0`: the vendored comparison's own literal is the bare integer `0`
        // (`MultiPoint.cpp:79,622,647`), implicitly converted to `double`; `0f64` normalises to
        // the same exempt `"0"` text the vendored literal already is, so no marker is needed
        // (`docs/PORTING.md` decision 8's `f64`-suffix technique, used here to keep an already-
        // exempt value exempt rather than to transcribe one).
        if scaled_epsilon == 0f64 {
            return self.find_point(point);
        }
        let mut dist2_min = f64::MAX;
        let eps2 = scaled_epsilon * scaled_epsilon;
        let mut idx_min: i32 = -1;
        for (i, pt) in self.points().iter().enumerate() {
            let d = pt.to_point() - point;
            let dx = d.x() as f64;
            let dy = d.y() as f64;
            let d2 = dx * dx + dy * dy;
            if d2 < dist2_min {
                idx_min = i as i32;
                dist2_min = d2;
            }
        }
        if dist2_min < eps2 {
            idx_min
        } else {
            -1
        }
    }

    /// `int find_point(const Point3 &point) const` (`:167`, defined `MultiPoint.cpp:637-643`).
    fn find_point3(&self, point: Point3) -> i32 {
        for (i, pt) in self.points().iter().enumerate() {
            if *pt == point {
                return i as i32;
            }
        }
        -1
    }

    /// `int find_point(const Point3 &point, double scaled_epsilon) const` (`:168`, defined
    /// `MultiPoint.cpp:645-660`).
    fn find_point3_eps(&self, point: Point3, scaled_epsilon: f64) -> i32 {
        // `0f64`, not `0.0`: the vendored comparison's own literal is the bare integer `0`
        // (`MultiPoint.cpp:79,622,647`), implicitly converted to `double`; `0f64` normalises to
        // the same exempt `"0"` text the vendored literal already is, so no marker is needed
        // (`docs/PORTING.md` decision 8's `f64`-suffix technique, used here to keep an already-
        // exempt value exempt rather than to transcribe one).
        if scaled_epsilon == 0f64 {
            return self.find_point3(point);
        }
        let mut dist2_min = f64::MAX;
        let eps2 = scaled_epsilon * scaled_epsilon;
        let mut idx_min: i32 = -1;
        for (i, pt) in self.points().iter().enumerate() {
            let d = *pt - point;
            let dx = d.x() as f64;
            let dy = d.y() as f64;
            let dz = d.z() as f64;
            let d2 = dx * dx + dy * dy + dz * dz;
            if d2 < dist2_min {
                idx_min = i as i32;
                dist2_min = d2;
            }
        }
        if dist2_min < eps2 {
            idx_min
        } else {
            -1
        }
    }

    /// `bool remove_duplicate_points()` (`:171`, defined `MultiPoint.cpp:454-474`); see
    /// [`MultiPoint::remove_duplicate_points`]'s doc for the shared shape.
    fn remove_duplicate_points(&mut self) -> bool {
        let pts = self.points_mut();
        if pts.len() < 2 {
            return false;
        }
        let mut j = 0usize;
        for i in 1..pts.len() {
            if pts[j] == pts[i] {
                // Just increase index i.
            } else {
                j += 1;
                if j < i {
                    pts[j] = pts[i];
                }
            }
        }
        j += 1;
        if j < pts.len() {
            pts.truncate(j);
            true
        } else {
            false
        }
    }
}

// TODO(port): src/libslic3r/MultiPoint.hpp:114-119 -- `begin()`/`end()`/`cbegin()`/`cend()` on
// `MultiPoint` (and the equivalent plain `points()`/`points_mut()` iteration this trait's own
// accessors already give for `MultiPoint3`): a C++ range-for convenience with no separate Rust
// item to port -- `impl_m.points().iter()`/`.points_mut().iter_mut()` already give every
// in-scope caller the same iteration, so this is noted rather than given a marker site of its
// own.

#[cfg(test)]
mod tests {
    use super::*;
    use slic3r_proof::tiers::{compare_unit, Oracle};

    /// A minimal, faithful `MultiPoint` implementor for exercising the trait's default methods:
    /// `last_point()`/`lines()` ported exactly as `Polygon`'s own overrides
    /// (`Polygon::last_point()`/`Polygon::lines()`, `Polygon.hpp:52,55`, `Polygon.cpp:35-38`) --
    /// a closed ring -- so the FFI proofs below, which drive the oracle through a real
    /// `Slic3r::Polygon`, compare against exactly the same shape. This task's own crate
    /// (`slic3r-multipoint`) cannot depend on `slic3r-polygon` to use the real `Polygon` type
    /// here: the cut graph runs the other way (`slic3r-polygon` depends on `slic3r-multipoint`),
    /// so a reverse dependency would be a cycle.
    struct ClosedRing {
        points: Points,
    }

    impl MultiPoint for ClosedRing {
        fn points(&self) -> &Points {
            &self.points
        }
        fn points_mut(&mut self) -> &mut Points {
            &mut self.points
        }
        /// `const Point& last_point() const { return this->points.front(); }`
        /// (`Polygon.hpp:52`).
        fn last_point(&self) -> Point {
            self.points[0]
        }
        /// `Lines lines() const { return to_lines(*this); }` (`Polygon.hpp:55`,
        /// `Polygon.cpp:35-38`), through `inline Lines to_lines(const Polygon &poly)`
        /// (`Polygon.hpp:203-213`): every consecutive pair, plus the closing edge back to the
        /// first point.
        fn lines(&self) -> Lines {
            let pts = &self.points;
            let mut lines = Lines::new();
            if pts.len() > 2 {
                for i in 0..pts.len() - 1 {
                    lines.push(Line::new(pts[i], pts[i + 1]));
                }
                lines.push(Line::new(pts[pts.len() - 1], pts[0]));
            }
            lines
        }
    }

    fn ring(xs: &[i64], ys: &[i64]) -> ClosedRing {
        ClosedRing {
            points: xs.iter().zip(ys).map(|(&x, &y)| Point::new(x, y)).collect(),
        }
    }

    fn flatten(points: &Points) -> Vec<i64> {
        points.iter().flat_map(|p| [p.x(), p.y()]).collect()
    }

    #[test]
    fn multi_point_append_reverse_clear_are_plain_vec_ops() {
        let mut r = ring(&[0, 10, 10], &[0, 0, 10]);
        r.append(Point::new(0, 10));
        assert_eq!(r.size(), 4);
        assert!(!r.is_empty());
        r.reverse();
        assert_eq!(r.front(), Point::new(0, 10));
        assert_eq!(r.back(), Point::new(0, 0));
        r.clear();
        assert!(r.is_empty());
    }

    #[test]
    fn multi_point_has_duplicate_and_remove_duplicate_points() {
        let mut r = ring(&[0, 0, 10, 20], &[0, 0, 0, 0]);
        assert!(r.has_duplicate_points());
        assert!(r.remove_duplicate_points());
        assert_eq!(r.size(), 3);
        assert!(!r.has_duplicate_points());
    }

    #[test]
    fn multi_point_find_point_eps_and_symmetric_y() {
        let r = ring(&[0, 100], &[0, 0]);
        assert_eq!(r.find_point(Point::new(100, 0)), 1);
        assert_eq!(r.find_point(Point::new(5, 0)), -1);
        assert_eq!(r.find_point_eps(Point::new(3, 0), 10.0), 0);

        let mut r = ring(&[0, 100], &[0, 0]);
        r.symmetric_y(50);
        assert_eq!(r.points()[0], Point::new(100, 0));
        assert_eq!(r.points()[1], Point::new(0, 0));
    }

    #[test]
    fn multi_point_douglas_peucker_concave_hull_and_min_distance() {
        let pts: Points = vec![Point::new(0, 0), Point::new(50, 1), Point::new(100, 0)];
        let simplified = douglas_peucker(&pts, 5.0);
        assert_eq!(simplified, vec![Point::new(0, 0), Point::new(100, 0)]);

        let hull_input: Points = vec![
            Point::new(0, 0),
            Point::new(50, 0),
            Point::new(100, 0),
            Point::new(100, 100),
            Point::new(0, 100),
        ];
        let hull = concave_hull_2d(&hull_input, 0.0);
        assert!(hull.len() >= 4);

        let a: Points = vec![Point::new(0, 0), Point::new(100, 0)];
        let b: Points = vec![Point::new(0, 10), Point::new(100, 10)];
        assert_eq!(
            minimum_distance_between_lines_defined_by_points(&a, &b),
            10.0
        );
    }

    #[test]
    fn multi_point_free_get_extents_and_free_length_area() {
        let r = ring(&[0, 100, 50], &[0, 0, 100]);
        let bb = get_extents(&r);
        assert_eq!(bb.min, Point::new(0, 0));
        assert_eq!(bb.max, Point::new(100, 100));

        let rotated = get_extents_rotated(&r, 0.0);
        assert_eq!(rotated.min, Point::new(0, 0));
        assert_eq!(rotated.max, Point::new(100, 100));

        assert_eq!(
            length(r.points()),
            0.0 + diff_norm(Point::new(100, 0), Point::new(0, 0))
                + diff_norm(Point::new(50, 100), Point::new(100, 0))
        );
    }

    /// Proof path 1 (R19, `port/manifest.tsv`'s `MultiPoint.cpp` row): every function below is
    /// externally linked, genuine `MultiPoint.cpp` code (or, for `get_extents`/
    /// `get_extents_rotated`, its own free functions), called through
    /// `crates/slic3r-ffi/cxx/proof_units/multi_point.cpp`'s wrapper against a real
    /// `Slic3r::Polygon`, reached through `slic3r-proof`'s re-export of the FFI crate's per-unit
    /// proof bridges (`docs/PORTING.md`, "Decisions recorded on 2026-11-25"; the shared-file
    /// list that decision names is itself stale -- see this task's pull request body).
    // PROOF: 1
    #[test]
    fn multi_point_cpp_methods_match_the_oracle() {
        let xs = [0i64, 100, 100, 0];
        let ys = [0i64, 0, 100, 100];

        let mut r = ring(&xs, &ys);
        r.scale(2.5);
        let oracle = slic3r_proof::proof_units::multi_point::path1_scale(&xs, &ys, 2.5);
        compare_unit("MultiPoint.cpp", Oracle::Built, flatten(r.points()), oracle)
            .expect("MultiPoint::scale must match the oracle exactly");

        let mut r = ring(&xs, &ys);
        r.translate(Point::new(7, -3));
        let oracle = slic3r_proof::proof_units::multi_point::path1_translate(&xs, &ys, 7, -3);
        compare_unit("MultiPoint.cpp", Oracle::Built, flatten(r.points()), oracle)
            .expect("MultiPoint::translate must match the oracle exactly");

        let mut r = ring(&xs, &ys);
        r.rotate_cos_sin(0.6, 0.8);
        let oracle =
            slic3r_proof::proof_units::multi_point::path1_rotate_cos_sin(&xs, &ys, 0.6, 0.8);
        compare_unit("MultiPoint.cpp", Oracle::Built, flatten(r.points()), oracle)
            .expect("MultiPoint::rotate must match the oracle exactly");

        let r = ring(&xs, &ys);
        let actual_len = r.length();
        let oracle_len = slic3r_proof::proof_units::multi_point::path1_length(&xs, &ys);
        compare_unit("MultiPoint.cpp", Oracle::Built, actual_len, oracle_len)
            .expect("MultiPoint::length must match the oracle exactly");

        let actual_find = r.find_point(Point::new(100, 100));
        let oracle_find =
            slic3r_proof::proof_units::multi_point::path1_find_point(&xs, &ys, 100, 100);
        assert_eq!(actual_find, oracle_find);

        let bb = r.bounding_box();
        let actual_bb = vec![bb.min.x(), bb.min.y(), bb.max.x(), bb.max.y()];
        let oracle_bb = slic3r_proof::proof_units::multi_point::path1_bounding_box(&xs, &ys);
        compare_unit("MultiPoint.cpp", Oracle::Built, actual_bb, oracle_bb)
            .expect("MultiPoint::bounding_box must match the oracle exactly");

        let line = Line::new(Point::new(-10, 50), Point::new(200, 50));
        let actual_intersection = r.intersection(&line);
        let oracle_intersection = slic3r_proof::proof_units::multi_point::path1_intersection(
            &xs,
            &ys,
            line.a.x(),
            line.a.y(),
            line.b.x(),
            line.b.y(),
        );
        let actual_vec = match actual_intersection {
            Some(p) => vec![1i64, p.x(), p.y()],
            None => vec![0i64, 0, 0],
        };
        compare_unit(
            "MultiPoint.cpp",
            Oracle::Built,
            actual_vec,
            oracle_intersection,
        )
        .expect("MultiPoint::intersection must match the oracle exactly");

        let mut r2 = ring(&xs, &ys);
        r2.symmetric_y(50);
        let oracle = slic3r_proof::proof_units::multi_point::path1_symmetric_y(&xs, &ys, 50);
        compare_unit(
            "MultiPoint.cpp",
            Oracle::Built,
            flatten(r2.points()),
            oracle,
        )
        .expect("MultiPoint::symmetric_y must match the oracle exactly");

        // `concave_hull_2d` (review finding on this task's own pull request: no comparison
        // reached this function), first at `tolerance = 0.0`. At that tolerance the vendored
        // pop condition (`MultiPoint.cpp:372`) is unsatisfiable: `ccw <= 0` forces `-ccw >= 0`,
        // so `-ccw / norm` is `>= 0`, `+inf` or `NaN` and never `< 0.0`. This sub-arm therefore
        // pins the *no-pop* path only -- every input point is kept -- and it does not exercise
        // the divisor at all. The sub-arm below is the one that does.
        let hull_input: Points = vec![
            Point::new(0, 0),
            Point::new(50, 0),
            Point::new(50, 10),
            Point::new(50, 0),
            Point::new(100, 0),
            Point::new(100, 100),
            Point::new(0, 100),
        ];
        let hull_xs: Vec<i64> = hull_input.iter().map(|p| p.x()).collect();
        let hull_ys: Vec<i64> = hull_input.iter().map(|p| p.y()).collect();
        let actual_hull = flatten(&concave_hull_2d(&hull_input, 0.0));
        let oracle_hull =
            slic3r_proof::proof_units::multi_point::path1_concave_hull_2d(&hull_xs, &hull_ys, 0.0);
        compare_unit("MultiPoint.cpp", Oracle::Built, actual_hull, oracle_hull)
            .expect("MultiPoint::concave_hull_2d must match the oracle exactly");

        // `concave_hull_2d` again, at `tolerance = 3.25` -- a positive tolerance chosen so that
        // the `--k` pop branch runs *and* the divisor at `MultiPoint.cpp:372` is load-bearing on
        // this fixed input: the integer-truncated Eigen `norm()` this module reproduces with
        // `diff_norm_eigen_int_truncated`, not the exact `f64` `diff_norm`. Walked through the
        // vendored loop:
        //   - `(50,-1)` against the chain `(0,0)`,`(10,0)`: `-ccw = 10`, divisor `|(40,-1)|`
        //     (truncated `40`), so `0.25 < 3.25` and `--k` runs.
        //   - `(13,-1)` against `(0,0)`,`(50,-1)`: `-ccw = 37`, divisor `|(-37,0)| = 37` exactly,
        //     so `1.0 < 3.25` and `--k` runs again.
        //   - `(16,-2)` against `(0,0)`,`(13,-1)`: `-ccw = 10`, divisor `|(3,-1)|`, whose
        //     truncated norm `3` gives `10/3 = 3.333... >= 3.25` (no pop, `(13,-1)` kept) while
        //     the exact `f64` norm `3.1622...` would give `3.1622... < 3.25` (pop, `(13,-1)`
        //     dropped). The two divisors therefore produce different hulls here.
        // Confirmed by negative control: replacing `diff_norm_eigen_int_truncated` with
        // `diff_norm` in `concave_hull_2d` makes this sub-arm fail against the oracle. Arbitrary
        // finite tolerances are additionally covered by fuzz arm 7 of
        // `crates/slic3r-ffi/fuzz/fuzz_targets/slic3r-multipoint_multi_point.rs`.
        let hull_pop_input: Points = vec![
            Point::new(0, 0),
            Point::new(10, 0),
            Point::new(50, -1),
            Point::new(13, -1),
            Point::new(16, -2),
        ];
        let hull_pop_xs: Vec<i64> = hull_pop_input.iter().map(|p| p.x()).collect();
        let hull_pop_ys: Vec<i64> = hull_pop_input.iter().map(|p| p.y()).collect();
        let actual_hull_pop = flatten(&concave_hull_2d(&hull_pop_input, 3.25));
        let oracle_hull_pop = slic3r_proof::proof_units::multi_point::path1_concave_hull_2d(
            &hull_pop_xs,
            &hull_pop_ys,
            3.25,
        );
        compare_unit(
            "MultiPoint.cpp",
            Oracle::Built,
            actual_hull_pop,
            oracle_hull_pop,
        )
        .expect(
            "MultiPoint::concave_hull_2d must match the oracle exactly where the pop branch runs",
        );

        // `minimumDistanceBetweenLinesDefinedByPoints` (review finding: no comparison reached
        // this function or its only caller, `squaredDistanceToLineSegment`) -- the reviewer's
        // own worked example, whose nearest foot (`t = 0.8`) does not land on an integer
        // coordinate, so it discriminates the rounding this proof holds the port to.
        let min_dist_a: Points = vec![Point::new(0, 0), Point::new(3, 1)];
        let min_dist_b: Points = vec![Point::new(2, 2)];
        let actual_min_dist =
            minimum_distance_between_lines_defined_by_points(&min_dist_a, &min_dist_b);
        let oracle_min_dist =
            slic3r_proof::proof_units::multi_point::path1_min_distance_between_lines(
                &[0, 3],
                &[0, 1],
                &[2],
                &[2],
            );
        compare_unit(
            "MultiPoint.cpp",
            Oracle::Built,
            actual_min_dist,
            oracle_min_dist,
        )
        .expect(
            "MultiPoint::minimumDistanceBetweenLinesDefinedByPoints (and the private \
             squaredDistanceToLineSegment it alone calls) must match the oracle exactly",
        );

        // `get_extents_rotated(const Points&, double)` (review finding: no comparison reached
        // this function) -- a single point, so `defined` is exercised at its own degenerate case.
        let single = vec![Point::new(7, 9)];
        let single_xs: Vec<i64> = single.iter().map(|p| p.x()).collect();
        let single_ys: Vec<i64> = single.iter().map(|p| p.y()).collect();
        let rotated = get_extents_rotated_points(&single, 0.3);
        let actual_rotated = vec![
            rotated.min.x(),
            rotated.min.y(),
            rotated.max.x(),
            rotated.max.y(),
            if rotated.defined { 1 } else { 0 },
        ];
        let oracle_rotated = slic3r_proof::proof_units::multi_point::path1_get_extents_rotated(
            &single_xs, &single_ys, 0.3,
        );
        compare_unit(
            "MultiPoint.cpp",
            Oracle::Built,
            actual_rotated,
            oracle_rotated,
        )
        .expect("get_extents_rotated(Points, angle) must match the oracle exactly");
    }

    /// Proof path 2 (R19, `port/manifest.tsv`'s `MultiPoint.hpp` row): `front()`/`back()`/
    /// `closest_point()`/`distance_to()`/`is_valid()` are `MultiPoint.hpp`'s own header-only,
    /// inline-defined methods, and `length(const Points&)`/`area(const Points&)` its own
    /// header-only free functions -- no stage dump field is any one of these calls' own output
    /// in isolation, so this proof reaches the oracle through the per-unit bridge instead, naming
    /// oracle `built`, the same `proof path: 2`-with-no-dump-coverage shape
    /// `docs/PORTING.md`'s "Decisions recorded on 2026-11-26", decision 2, accepts.
    // PROOF: 2
    #[test]
    fn multi_point_hpp_methods_match_the_oracle() {
        let xs = [0i64, 100, 100, 0];
        let ys = [0i64, 0, 100, 100];
        let r = ring(&xs, &ys);

        let actual = vec![r.front().x(), r.front().y(), r.back().x(), r.back().y()];
        let oracle = slic3r_proof::proof_units::multi_point::path2_front_back(&xs, &ys);
        compare_unit("MultiPoint.hpp", Oracle::Built, actual, oracle)
            .expect("MultiPoint::front/back must match the oracle exactly");

        let actual_cp = r.closest_point(Point::new(90, 5));
        let actual_cp_vec = match actual_cp {
            Some(p) => vec![1i64, p.x(), p.y()],
            None => vec![0, 0, 0],
        };
        let oracle_cp =
            slic3r_proof::proof_units::multi_point::path2_closest_point(&xs, &ys, 90, 5);
        compare_unit("MultiPoint.hpp", Oracle::Built, actual_cp_vec, oracle_cp)
            .expect("MultiPoint::closest_point must match the oracle exactly");

        let actual_dist = r.distance_to(Point::new(90, 5));
        let oracle_dist =
            slic3r_proof::proof_units::multi_point::path2_distance_to(&xs, &ys, 90, 5);
        compare_unit("MultiPoint.hpp", Oracle::Built, actual_dist, oracle_dist)
            .expect("MultiPoint::distance_to must match the oracle exactly");

        let actual_valid = r.is_valid();
        let oracle_valid = slic3r_proof::proof_units::multi_point::path2_is_valid(&xs, &ys);
        assert_eq!(actual_valid, oracle_valid);

        let actual_length_free = length(r.points());
        let oracle_length_free =
            slic3r_proof::proof_units::multi_point::path2_length_free(&xs, &ys);
        compare_unit(
            "MultiPoint.hpp",
            Oracle::Built,
            actual_length_free,
            oracle_length_free,
        )
        .expect("the free length(Points) must match the oracle exactly");

        let actual_area_free = area(r.points());
        let oracle_area_free = slic3r_proof::proof_units::multi_point::path2_area_free(&xs, &ys);
        compare_unit(
            "MultiPoint.hpp",
            Oracle::Built,
            actual_area_free,
            oracle_area_free,
        )
        .expect("the free area(Points) must match the oracle exactly");
    }
}

// PORT STATUS
// source: src/libslic3r/MultiPoint.cpp
// source: src/libslic3r/MultiPoint.hpp
// confidence: medium
// tier: exact
// oracle: built
