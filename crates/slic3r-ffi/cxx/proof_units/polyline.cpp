// Proof shims for the `Polyline.cpp`/`Polyline.hpp` unit (R19, proof paths 1 and 2, task
// `63-foundation-polyline`), compiled onto `crates/slic3r-ffi`'s shared `cxx_build::Build` by this
// crate's `build.rs` -- the one sanctioned route a ported unit reaches the oracle by
// (`docs/PORTING.md`, "Decisions recorded on 2026-11-25").
//
// `path1_clip_end` drives `Slic3r::Polyline::clip_end(double)` (`Polyline.cpp:61-101`), an
// externally linked, non-template method genuinely defined in the `.cpp` -- proof path 1.
//
// `path2_total_length`/`path2_is_closed`/`path2_new_scale` drive `Polyline.hpp`'s own
// declared-and-defined-in-the-header surface (`total_length`, `Polyline::is_closed`,
// `Polyline::new_scale`) directly through a real `Slic3r::Polyline` -- `Polyline.hpp`'s own row is
// `proof path: 2` (its committed pipeline stage, `posSlice`, writes no field this header-only
// surface produces: every `posSlice.dump` field is closed-ring `PrintObject`/`Layer` geometry),
// read through the oracle's real, compiled behaviour rather than a stage dump, the same route
// `cxx/proof_units/point.cpp`'s `path2_point_rotate` uses and `docs/PORTING.md`'s "Decisions
// recorded on 2026-11-26" (decision 2) settles as conforming for a `proof path: 2` row whose unit
// writes nothing any dump records.
#include "polyline.h"

#include "Polyline.hpp"

namespace slic3r_ffi {
namespace proof_units {
namespace polyline {

static Slic3r::Polyline make_polyline(rust::Slice<const int64_t> xs,
                                       rust::Slice<const int64_t> ys) {
    Slic3r::Points pts;
    pts.reserve(xs.size());
    for (size_t i = 0; i < xs.size(); ++i) {
        pts.emplace_back(static_cast<coord_t>(xs[i]), static_cast<coord_t>(ys[i]));
    }
    return Slic3r::Polyline(pts);
}

static rust::Vec<int64_t> flatten(const Slic3r::Polyline &pl) {
    rust::Vec<int64_t> out;
    for (const Slic3r::Point &p : pl.points) {
        out.push_back(static_cast<int64_t>(p.x()));
        out.push_back(static_cast<int64_t>(p.y()));
    }
    return out;
}

rust::Vec<int64_t> path1_clip_end(rust::Slice<const int64_t> xs, rust::Slice<const int64_t> ys,
                                   double distance) {
    Slic3r::Polyline pl = make_polyline(xs, ys);
    pl.clip_end(distance);
    return flatten(pl);
}

double path2_total_length(rust::Slice<const int64_t> xs, rust::Slice<const int64_t> ys) {
    Slic3r::Polylines pls;
    pls.push_back(make_polyline(xs, ys));
    return Slic3r::total_length(pls);
}

bool path2_is_closed(rust::Slice<const int64_t> xs, rust::Slice<const int64_t> ys) {
    Slic3r::Polyline pl = make_polyline(xs, ys);
    return pl.is_closed();
}

rust::Vec<int64_t> path2_new_scale(rust::Slice<const double> xs, rust::Slice<const double> ys) {
    std::vector<Slic3r::Vec2d> pts;
    pts.reserve(xs.size());
    for (size_t i = 0; i < xs.size(); ++i) {
        pts.emplace_back(xs[i], ys[i]);
    }
    Slic3r::Polyline pl = Slic3r::Polyline::new_scale(pts);
    return flatten(pl);
}

} // namespace polyline
} // namespace proof_units
} // namespace slic3r_ffi
