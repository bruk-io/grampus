//! Proof paths 1 and 2 (R19, task `58-foundation-bounding-box`) for the
//! `BoundingBox.cpp`/`BoundingBox.hpp` unit.
//!
//! The `path1_*` functions call `BoundingBox.cpp`'s own, non-template methods, externally linked
//! in the archive `just oracle-build` produces. The `path2_*` functions drive `BoundingBox.hpp`'s
//! own header-only `BoundingBoxBase<Point, Points>` surface directly, instantiated in
//! `cxx/proof_units/bounding_box.cpp` -- the same technique `cxx/proof_units/point.cpp`'s
//! `scaled<coord_t>()`/`unscaled<double>()` shims use for header-only template code
//! (`## Decisions recorded on 2026-11-26`, decision 2: a `proof path: 2` row's oracle route is
//! not restricted to the committed stage dumps).
//!
//! Every function takes a `BoundingBox`'s three fields (`min.x()`, `min.y()`, `max.x()`,
//! `max.y()`, `defined`) as plain scalars, assigned directly onto the shim's own real
//! `Slic3r::BoundingBox` rather than through a constructor -- see `bounding_box.cpp`'s module
//! doc for why.

#[cxx::bridge(namespace = "slic3r_ffi::proof_units::bounding_box")]
mod ffi {
    // SAFETY: this is cxx's own unsafe-extern declaration block, not a hand-written unsafe
    // expression (see src/clipper2.rs's identical note) -- it carries no body, and the only
    // unsafety is in the bindings `cxx` generates from it.
    unsafe extern "C++" {
        include!("cxx/proof_units/bounding_box.h");

        /// `BoundingBox::rotated(double angle) const` (`BoundingBox.cpp:37-45`), returning
        /// `[min.x, min.y, max.x, max.y]`. Always `defined` on return (four `merge` calls), so
        /// `defined` is not part of the result.
        fn path1_rotated(min_x: i64, min_y: i64, max_x: i64, max_y: i64, angle: f64) -> Vec<i64>;
        /// `BoundingBox::rotated(double angle, const Point &center) const` (`:47-55`); see
        /// [`path1_rotated`]'s doc.
        fn path1_rotated_about(
            min_x: i64,
            min_y: i64,
            max_x: i64,
            max_y: i64,
            cx: i64,
            cy: i64,
            angle: f64,
        ) -> Vec<i64>;
        /// `BoundingBox::scaled(double factor) const` (`:57-62`), returning
        /// `[min.x, min.y, max.x, max.y, defined (0/1)]`.
        fn path1_scaled(
            min_x: i64,
            min_y: i64,
            max_x: i64,
            max_y: i64,
            defined: bool,
            factor: f64,
        ) -> Vec<i64>;
        /// `BoundingBox::align_to_grid(const coord_t cell_size)` (`:253-259`); `max` is
        /// irrelevant to it (only `min` is read or written), so the shim holds it fixed and the
        /// bridge returns only the resulting `[min.x, min.y]`.
        fn path1_align_to_grid(min_x: i64, min_y: i64, defined: bool, cell_size: i64) -> Vec<i64>;

        /// `BoundingBoxBase::merge(const PointType &point)` (`BoundingBox.cpp:74-85`), returning
        /// `[min.x, min.y, max.x, max.y, defined (0/1)]`.
        fn path2_merge_point(
            min_x: i64,
            min_y: i64,
            max_x: i64,
            max_y: i64,
            defined: bool,
            px: i64,
            py: i64,
        ) -> Vec<i64>;
        /// `BoundingBox(const Points &points)`, through `BoundingBoxBase::construct<false>`
        /// (`BoundingBox.hpp:98-112`); `xs`/`ys` are parallel slices (`cxx` has no
        /// `Vec<(i64, i64)>`), the same convention `cxx/proof_units/point.cpp`'s
        /// `path1_nearest_point_index` uses.
        fn path2_merge_points(xs: &[i64], ys: &[i64]) -> Vec<i64>;
        /// `BoundingBoxBase::merge(const BoundingBoxBase &bb)` (`BoundingBox.cpp:90-112`).
        fn path2_merge_bb(
            min_x: i64,
            min_y: i64,
            max_x: i64,
            max_y: i64,
            defined: bool,
            omin_x: i64,
            omin_y: i64,
            omax_x: i64,
            omax_y: i64,
            odefined: bool,
        ) -> Vec<i64>;
        /// `BoundingBoxBase::scale(double factor)` (`:64-69`), `this->min *= factor; this->max *=
        /// factor;` through `Point::operator*=(const double&)` (`Point.hpp:228`), which hides
        /// Eigen's own mixed-scalar `operator*=` -- returns `[min.x, min.y, max.x, max.y]` after
        /// the in-place mutation, `defined` untouched by the method so not returned.
        fn path2_scale(min_x: i64, min_y: i64, max_x: i64, max_y: i64, factor: f64) -> Vec<i64>;
        /// `BoundingBoxBase::size() const` (`:177-181`), `[size.x, size.y]`.
        fn path2_size(min_x: i64, min_y: i64, max_x: i64, max_y: i64) -> Vec<i64>;
        /// `BoundingBoxBase::radius() const` (`:194-198`).
        fn path2_radius(min_x: i64, min_y: i64, max_x: i64, max_y: i64) -> f64;
        /// `BoundingBoxBase::area() const` (`BoundingBox.hpp:45`).
        fn path2_area(min_x: i64, min_y: i64, max_x: i64, max_y: i64) -> f64;
        /// `BoundingBoxBase::center() const` (`BoundingBox.cpp:227-231`), `[center.x, center.y]`.
        fn path2_center(min_x: i64, min_y: i64, max_x: i64, max_y: i64) -> Vec<i64>;
        /// `BoundingBoxBase::translate(coordf_t x, coordf_t y)` (`BoundingBox.hpp:46`), the
        /// `PointType v(x, y)` overload -- for `Point`, `Point(double, double)` rounds
        /// (`Point.hpp:208`).
        fn path2_translate_xy(
            min_x: i64,
            min_y: i64,
            max_x: i64,
            max_y: i64,
            tx: f64,
            ty: f64,
        ) -> Vec<i64>;
        /// `BoundingBoxBase::translate(const PointType &v)` (`:47`), the exact-add overload with
        /// no rounding and no `assert(defined)`.
        fn path2_translate_point(
            min_x: i64,
            min_y: i64,
            max_x: i64,
            max_y: i64,
            vx: i64,
            vy: i64,
        ) -> Vec<i64>;
        /// `BoundingBoxBase::offset(coordf_t delta)` (`BoundingBox.cpp:208-214`).
        fn path2_offset(min_x: i64, min_y: i64, max_x: i64, max_y: i64, delta: f64) -> Vec<i64>;
        /// `BoundingBoxBase::contains(const PointType &point) const` (`BoundingBox.hpp:51-54`).
        fn path2_contains_point(
            min_x: i64,
            min_y: i64,
            max_x: i64,
            max_y: i64,
            px: i64,
            py: i64,
        ) -> bool;
        /// `BoundingBoxBase::contains(const BoundingBoxBase &other) const` (`:55-57`).
        fn path2_contains_bbox(
            min_x: i64,
            min_y: i64,
            max_x: i64,
            max_y: i64,
            omin_x: i64,
            omin_y: i64,
            omax_x: i64,
            omax_y: i64,
        ) -> bool;
        /// `BoundingBoxBase::overlap(const BoundingBoxBase &other) const` (`:58-61`).
        fn path2_overlap(
            min_x: i64,
            min_y: i64,
            max_x: i64,
            max_y: i64,
            omin_x: i64,
            omin_y: i64,
            omax_x: i64,
            omax_y: i64,
        ) -> bool;
        /// `BoundingBoxBase::operator[](size_t idx) const` (`:62-81`), `[point.x, point.y]`.
        fn path2_index(min_x: i64, min_y: i64, max_x: i64, max_y: i64, idx: i32) -> Vec<i64>;
        /// `BoundingBoxBase::operator==(const BoundingBoxBase &rhs)` (`:82`): compares `min`/`max`
        /// only, never `defined`.
        fn path2_eq(
            min_x: i64,
            min_y: i64,
            max_x: i64,
            max_y: i64,
            omin_x: i64,
            omin_y: i64,
            omax_x: i64,
            omax_y: i64,
        ) -> bool;
        /// `BoundingBoxBase::operator==` again, but with `defined`/`odefined` taken
        /// independently (unlike [`path2_eq`], which always passes `true` for both) so a
        /// differing `defined` actually reaches the oracle.
        fn path2_eq_defined(
            min_x: i64,
            min_y: i64,
            max_x: i64,
            max_y: i64,
            defined: bool,
            omin_x: i64,
            omin_y: i64,
            omax_x: i64,
            omax_y: i64,
            odefined: bool,
        ) -> bool;
        /// `BoundingBox(const Point&, const Point&)` (`BoundingBox.hpp:225`), through the base's
        /// `BoundingBoxBase(const PointType&, const PointType&)` (`:25-26`): drives `defined`
        /// through the real constructor rather than `make_bbox`'s direct field assignment.
        fn path2_from_min_max_defined(min_x: i64, min_y: i64, max_x: i64, max_y: i64) -> bool;
        /// `inline BoundingBox scaled(const BoundingBoxf &bb)` (`BoundingBox.hpp:284`), returning
        /// `[min.x, min.y, max.x, max.y, defined (0/1)]`.
        fn path2_scaled_bbf(min_x: f64, min_y: f64, max_x: f64, max_y: f64) -> Vec<i64>;
        /// `unscaled<double>(const BoundingBox &bb)` (`BoundingBox.hpp:292-293`) at its default
        /// `T`, returning `[min.x, min.y, max.x, max.y, defined (0.0/1.0)]`.
        fn path2_unscaled_bb(min_x: i64, min_y: i64, max_x: i64, max_y: i64) -> Vec<f64>;
    }
}

pub use ffi::{
    path1_align_to_grid, path1_rotated, path1_rotated_about, path1_scaled, path2_area,
    path2_center, path2_contains_bbox, path2_contains_point, path2_eq, path2_eq_defined,
    path2_from_min_max_defined, path2_index, path2_merge_bb, path2_merge_point, path2_merge_points,
    path2_offset, path2_overlap, path2_radius, path2_scale, path2_scaled_bbf, path2_size,
    path2_translate_point, path2_translate_xy, path2_unscaled_bb,
};
