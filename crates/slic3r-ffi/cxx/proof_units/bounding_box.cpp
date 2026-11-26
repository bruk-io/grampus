// Proof shims for the `BoundingBox.cpp`/`BoundingBox.hpp` unit (R19, proof paths 1 and 2, task
// `58-foundation-bounding-box`), compiled onto `crates/slic3r-ffi`'s shared `cxx_build::Build` by
// this crate's `build.rs` -- the one sanctioned route a ported unit reaches the oracle by
// (`docs/PORTING.md`, "Decisions recorded on 2026-11-25").
//
// Every function here takes a `Slic3r::BoundingBox`'s three fields (`min.x()`, `min.y()`,
// `max.x()`, `max.y()`, `defined`) as plain scalars and assigns them directly onto a real
// `Slic3r::BoundingBox` (never through a constructor, whose own invariants -- e.g. `defined =
// pmin.x() < pmax.x() && pmin.y() < pmax.y()` -- would silently override whatever state a caller
// asked for) so each shim drives the method under test against exactly the state the Rust side
// passed, including a `defined` value a constructor could never produce on its own. The `path1_*`
// functions call `BoundingBox.cpp`'s own, non-template methods, externally linked in the archive
// `just oracle-build` produces; the `path2_*` functions drive `BoundingBox.hpp`'s own header-only
// `BoundingBoxBase<Point, Points>` surface directly, instantiated in this translation unit (the
// same technique `cxx/proof_units/point.cpp`'s `scaled<coord_t>()`/`unscaled<double>()` shims use
// for header-only template code).
#include "bounding_box.h"

#include "BoundingBox.hpp"

namespace slic3r_ffi {
namespace proof_units {
namespace bounding_box {

namespace {

Slic3r::BoundingBox make_bbox(int64_t min_x, int64_t min_y, int64_t max_x, int64_t max_y,
                               bool defined) {
    Slic3r::BoundingBox bb;
    bb.min = Slic3r::Point(min_x, min_y);
    bb.max = Slic3r::Point(max_x, max_y);
    bb.defined = defined;
    return bb;
}

rust::Vec<int64_t> pack(const Slic3r::BoundingBox &bb) {
    rust::Vec<int64_t> out;
    out.push_back(static_cast<int64_t>(bb.min.x()));
    out.push_back(static_cast<int64_t>(bb.min.y()));
    out.push_back(static_cast<int64_t>(bb.max.x()));
    out.push_back(static_cast<int64_t>(bb.max.y()));
    out.push_back(bb.defined ? 1 : 0);
    return out;
}

} // namespace

rust::Vec<int64_t> path1_rotated(int64_t min_x, int64_t min_y, int64_t max_x, int64_t max_y,
                                  double angle) {
    Slic3r::BoundingBox bb = make_bbox(min_x, min_y, max_x, max_y, true);
    Slic3r::BoundingBox out = bb.rotated(angle);
    rust::Vec<int64_t> result;
    result.push_back(static_cast<int64_t>(out.min.x()));
    result.push_back(static_cast<int64_t>(out.min.y()));
    result.push_back(static_cast<int64_t>(out.max.x()));
    result.push_back(static_cast<int64_t>(out.max.y()));
    return result;
}

rust::Vec<int64_t> path1_rotated_about(int64_t min_x, int64_t min_y, int64_t max_x, int64_t max_y,
                                        int64_t cx, int64_t cy, double angle) {
    Slic3r::BoundingBox bb = make_bbox(min_x, min_y, max_x, max_y, true);
    Slic3r::BoundingBox out = bb.rotated(angle, Slic3r::Point(cx, cy));
    rust::Vec<int64_t> result;
    result.push_back(static_cast<int64_t>(out.min.x()));
    result.push_back(static_cast<int64_t>(out.min.y()));
    result.push_back(static_cast<int64_t>(out.max.x()));
    result.push_back(static_cast<int64_t>(out.max.y()));
    return result;
}

rust::Vec<int64_t> path1_scaled(int64_t min_x, int64_t min_y, int64_t max_x, int64_t max_y,
                                 bool defined, double factor) {
    Slic3r::BoundingBox bb = make_bbox(min_x, min_y, max_x, max_y, defined);
    Slic3r::BoundingBox out = bb.scaled(factor);
    return pack(out);
}

rust::Vec<int64_t> path1_align_to_grid(int64_t min_x, int64_t min_y, bool defined,
                                        int64_t cell_size) {
    // `align_to_grid` only ever reads/writes `min`; `max` is irrelevant to it, so it is held at
    // an arbitrary fixed value here.
    Slic3r::BoundingBox bb = make_bbox(min_x, min_y, min_x + 1, min_y + 1, defined);
    bb.align_to_grid(static_cast<coord_t>(cell_size));
    rust::Vec<int64_t> result;
    result.push_back(static_cast<int64_t>(bb.min.x()));
    result.push_back(static_cast<int64_t>(bb.min.y()));
    return result;
}

rust::Vec<int64_t> path2_merge_point(int64_t min_x, int64_t min_y, int64_t max_x, int64_t max_y,
                                      bool defined, int64_t px, int64_t py) {
    Slic3r::BoundingBox bb = make_bbox(min_x, min_y, max_x, max_y, defined);
    bb.merge(Slic3r::Point(px, py));
    return pack(bb);
}

rust::Vec<int64_t> path2_merge_points(rust::Slice<const int64_t> xs,
                                       rust::Slice<const int64_t> ys) {
    Slic3r::Points points;
    points.reserve(xs.size());
    for (size_t i = 0; i < xs.size(); ++i) {
        points.emplace_back(static_cast<coord_t>(xs[i]), static_cast<coord_t>(ys[i]));
    }
    // The public `BoundingBox(const Points&)` constructor -- `construct<false>` (the only
    // instantiation reachable from outside `BoundingBox.hpp`'s own `friend`-gated `get_extents`,
    // see the module's own doc for why `get_extents<true>` has no caller this unit can reach).
    Slic3r::BoundingBox bb(points);
    return pack(bb);
}

rust::Vec<int64_t> path2_merge_bb(int64_t min_x, int64_t min_y, int64_t max_x, int64_t max_y,
                                   bool defined, int64_t omin_x, int64_t omin_y, int64_t omax_x,
                                   int64_t omax_y, bool odefined) {
    Slic3r::BoundingBox bb = make_bbox(min_x, min_y, max_x, max_y, defined);
    Slic3r::BoundingBox other = make_bbox(omin_x, omin_y, omax_x, omax_y, odefined);
    bb.merge(other);
    return pack(bb);
}

rust::Vec<int64_t> path2_scale(int64_t min_x, int64_t min_y, int64_t max_x, int64_t max_y,
                                double factor) {
    Slic3r::BoundingBox bb = make_bbox(min_x, min_y, max_x, max_y, true);
    bb.scale(factor);
    rust::Vec<int64_t> result;
    result.push_back(static_cast<int64_t>(bb.min.x()));
    result.push_back(static_cast<int64_t>(bb.min.y()));
    result.push_back(static_cast<int64_t>(bb.max.x()));
    result.push_back(static_cast<int64_t>(bb.max.y()));
    return result;
}

rust::Vec<int64_t> path2_size(int64_t min_x, int64_t min_y, int64_t max_x, int64_t max_y) {
    Slic3r::BoundingBox bb = make_bbox(min_x, min_y, max_x, max_y, true);
    Slic3r::Point s = bb.size();
    rust::Vec<int64_t> result;
    result.push_back(static_cast<int64_t>(s.x()));
    result.push_back(static_cast<int64_t>(s.y()));
    return result;
}

double path2_radius(int64_t min_x, int64_t min_y, int64_t max_x, int64_t max_y) {
    Slic3r::BoundingBox bb = make_bbox(min_x, min_y, max_x, max_y, true);
    return bb.radius();
}

double path2_area(int64_t min_x, int64_t min_y, int64_t max_x, int64_t max_y) {
    Slic3r::BoundingBox bb = make_bbox(min_x, min_y, max_x, max_y, true);
    return bb.area();
}

rust::Vec<int64_t> path2_center(int64_t min_x, int64_t min_y, int64_t max_x, int64_t max_y) {
    Slic3r::BoundingBox bb = make_bbox(min_x, min_y, max_x, max_y, true);
    Slic3r::Point c = bb.center();
    rust::Vec<int64_t> result;
    result.push_back(static_cast<int64_t>(c.x()));
    result.push_back(static_cast<int64_t>(c.y()));
    return result;
}

rust::Vec<int64_t> path2_translate_xy(int64_t min_x, int64_t min_y, int64_t max_x, int64_t max_y,
                                       double tx, double ty) {
    Slic3r::BoundingBox bb = make_bbox(min_x, min_y, max_x, max_y, true);
    bb.translate(tx, ty);
    rust::Vec<int64_t> result;
    result.push_back(static_cast<int64_t>(bb.min.x()));
    result.push_back(static_cast<int64_t>(bb.min.y()));
    result.push_back(static_cast<int64_t>(bb.max.x()));
    result.push_back(static_cast<int64_t>(bb.max.y()));
    return result;
}

rust::Vec<int64_t> path2_translate_point(int64_t min_x, int64_t min_y, int64_t max_x,
                                          int64_t max_y, int64_t vx, int64_t vy) {
    Slic3r::BoundingBox bb = make_bbox(min_x, min_y, max_x, max_y, true);
    bb.translate(Slic3r::Point(vx, vy));
    rust::Vec<int64_t> result;
    result.push_back(static_cast<int64_t>(bb.min.x()));
    result.push_back(static_cast<int64_t>(bb.min.y()));
    result.push_back(static_cast<int64_t>(bb.max.x()));
    result.push_back(static_cast<int64_t>(bb.max.y()));
    return result;
}

rust::Vec<int64_t> path2_offset(int64_t min_x, int64_t min_y, int64_t max_x, int64_t max_y,
                                 double delta) {
    Slic3r::BoundingBox bb = make_bbox(min_x, min_y, max_x, max_y, true);
    bb.offset(delta);
    rust::Vec<int64_t> result;
    result.push_back(static_cast<int64_t>(bb.min.x()));
    result.push_back(static_cast<int64_t>(bb.min.y()));
    result.push_back(static_cast<int64_t>(bb.max.x()));
    result.push_back(static_cast<int64_t>(bb.max.y()));
    return result;
}

bool path2_contains_point(int64_t min_x, int64_t min_y, int64_t max_x, int64_t max_y, int64_t px,
                           int64_t py) {
    Slic3r::BoundingBox bb = make_bbox(min_x, min_y, max_x, max_y, true);
    return bb.contains(Slic3r::Point(px, py));
}

bool path2_contains_bbox(int64_t min_x, int64_t min_y, int64_t max_x, int64_t max_y,
                          int64_t omin_x, int64_t omin_y, int64_t omax_x, int64_t omax_y) {
    Slic3r::BoundingBox bb = make_bbox(min_x, min_y, max_x, max_y, true);
    Slic3r::BoundingBox other = make_bbox(omin_x, omin_y, omax_x, omax_y, true);
    return bb.contains(other);
}

bool path2_overlap(int64_t min_x, int64_t min_y, int64_t max_x, int64_t max_y, int64_t omin_x,
                    int64_t omin_y, int64_t omax_x, int64_t omax_y) {
    Slic3r::BoundingBox bb = make_bbox(min_x, min_y, max_x, max_y, true);
    Slic3r::BoundingBox other = make_bbox(omin_x, omin_y, omax_x, omax_y, true);
    return bb.overlap(other);
}

rust::Vec<int64_t> path2_index(int64_t min_x, int64_t min_y, int64_t max_x, int64_t max_y,
                                int32_t idx) {
    Slic3r::BoundingBox bb = make_bbox(min_x, min_y, max_x, max_y, true);
    Slic3r::Point p = bb[static_cast<size_t>(idx)];
    rust::Vec<int64_t> result;
    result.push_back(static_cast<int64_t>(p.x()));
    result.push_back(static_cast<int64_t>(p.y()));
    return result;
}

bool path2_eq(int64_t min_x, int64_t min_y, int64_t max_x, int64_t max_y, int64_t omin_x,
              int64_t omin_y, int64_t omax_x, int64_t omax_y) {
    Slic3r::BoundingBox bb = make_bbox(min_x, min_y, max_x, max_y, true);
    Slic3r::BoundingBox other = make_bbox(omin_x, omin_y, omax_x, omax_y, true);
    return bb == other;
}

// Unlike `path2_eq` above (both boxes always `defined = true`, so the oracle never sees a
// differing `defined` and the "operator== ignores defined" claim could not fail), this leg
// takes `defined`/`odefined` independently so a differing value actually reaches the oracle.
bool path2_eq_defined(int64_t min_x, int64_t min_y, int64_t max_x, int64_t max_y, bool defined,
                       int64_t omin_x, int64_t omin_y, int64_t omax_x, int64_t omax_y,
                       bool odefined) {
    Slic3r::BoundingBox bb = make_bbox(min_x, min_y, max_x, max_y, defined);
    Slic3r::BoundingBox other = make_bbox(omin_x, omin_y, omax_x, omax_y, odefined);
    return bb == other;
}

// Drives `defined` through the real `BoundingBox(const Point&, const Point&)` constructor --
// via its base `BoundingBoxBase(const PointType&, const PointType&)` (`BoundingBox.hpp:25-26`)
// -- rather than `make_bbox`'s direct field assignment, so the constructor's own `defined`
// derivation rule is itself compared against the oracle (every other shim in this file
// deliberately bypasses the constructor; see this file's own module doc for why).
bool path2_from_min_max_defined(int64_t min_x, int64_t min_y, int64_t max_x, int64_t max_y) {
    Slic3r::BoundingBox bb(Slic3r::Point(min_x, min_y), Slic3r::Point(max_x, max_y));
    return bb.defined;
}

// `inline BoundingBox scaled(const BoundingBoxf &bb)` (`BoundingBox.hpp:284`), the free
// function this module's `scaled` ports -- constructed directly (never through
// `BoundingBoxf`'s own two-point constructor) so `defined` on the way in is irrelevant to the
// call, the same convention `make_bbox` uses above.
rust::Vec<int64_t> path2_scaled_bbf(double min_x, double min_y, double max_x, double max_y) {
    Slic3r::BoundingBoxf bb;
    bb.min = Slic3r::Vec2d(min_x, min_y);
    bb.max = Slic3r::Vec2d(max_x, max_y);
    bb.defined = true;
    Slic3r::BoundingBox out = Slic3r::scaled(bb);
    return pack(out);
}

// `template<class T = double> BoundingBoxBase<Vec<2, T>> unscaled(const BoundingBox &bb)`
// (`BoundingBox.hpp:292-293`) at its default `T = double`, the free function this module's
// `unscaled` ports.
rust::Vec<double> path2_unscaled_bb(int64_t min_x, int64_t min_y, int64_t max_x, int64_t max_y) {
    Slic3r::BoundingBox bb = make_bbox(min_x, min_y, max_x, max_y, true);
    // `unscaled<double>` is templated on `BoundingBoxBase<Vec<2, double>>`, not the nominal
    // derived `Slic3r::BoundingBoxf` -- same fields, but no implicit conversion between the two
    // distinct C++ types, so `out`'s type is left to `auto` rather than named as `BoundingBoxf`.
    auto out = Slic3r::unscaled(bb);
    rust::Vec<double> result;
    result.push_back(out.min.x());
    result.push_back(out.min.y());
    result.push_back(out.max.x());
    result.push_back(out.max.y());
    result.push_back(out.defined ? 1.0 : 0.0);
    return result;
}

} // namespace bounding_box
} // namespace proof_units
} // namespace slic3r_ffi
