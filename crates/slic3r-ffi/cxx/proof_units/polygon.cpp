// Proof path 3 (R19, plan step 18, task 31-proof-harness) for the `Polygon.cpp`/`Polygon.hpp`
// unit (task 62-foundation-polygon): a test translation unit that `#include`s `Polygon.cpp`
// whole, reaching its file-scope `static int point_in_polygon(const Point&, const Points&)`
// (`:738-766`), which has no external symbol to call any other way.
//
// SPIKE, kept as this unit's actual proof: the duplicate-symbol measurement
// `cxx/proof_units/clipper_z_utils.cpp`'s header comment requires before `#include`ing any
// vendored `.cpp` here. `Polygon.cpp` exports far more than `ClipperZUtils.cpp` did
// (`Polygon::area`, `contains`, `get_extents`, `remove_sticks`, `polygons_simplify`, ...) and
// pulls in `ClipperUtils.hpp`/`Polyline.hpp`, so this was measured empirically rather than
// assumed: `cargo build -p slic3r-ffi` with only this shim present links clean, with no
// duplicate-symbol or undefined-reference error (see this task's pull request body for the
// exact command and its output). The mechanism is the one that comment states: on this pin's
// macOS `ld`, a static-archive member is only extracted when something still unresolved needs
// it, and nothing else this crate links calls into `Polygon.cpp`'s own externally linked
// surface, so `liblibslic3r.a`'s copy of `Polygon.cpp.o` is never pulled in and this
// translation unit's copy is the only one in the final binary.
//
// Angle-bracket `#include`, not a quoted one: a quoted `#include "Polygon.cpp"` written inside
// a file named `polygon.cpp` resolves to *itself* on this case-insensitive filesystem, exactly
// the trap `cxx/proof_units/config.cpp`'s header comment documents for `Config.cpp`/`config.cpp`.
#include "polygon.h"

#include <Polygon.cpp>

namespace slic3r_ffi {
namespace proof_units {
namespace polygon {

namespace {

Slic3r::Polygon make_polygon(rust::Slice<const int64_t> xs, rust::Slice<const int64_t> ys) {
    Slic3r::Points pts;
    pts.reserve(xs.size());
    for (size_t i = 0; i < xs.size(); ++i)
        pts.emplace_back(xs[i], ys[i]);
    return Slic3r::Polygon(pts);
}

rust::Vec<int64_t> pack_points(const Slic3r::Points &pts) {
    rust::Vec<int64_t> out;
    out.reserve(pts.size() * 2);
    for (const Slic3r::Point &p : pts) {
        out.push_back(static_cast<int64_t>(p.x()));
        out.push_back(static_cast<int64_t>(p.y()));
    }
    return out;
}

} // namespace

int32_t path3_point_in_polygon(int64_t px, int64_t py, rust::Slice<const int64_t> xs,
                                rust::Slice<const int64_t> ys) {
    Slic3r::Points path;
    path.reserve(xs.size());
    for (size_t i = 0; i < xs.size(); ++i)
        path.emplace_back(xs[i], ys[i]);
    return static_cast<int32_t>(Slic3r::point_in_polygon(Slic3r::Point(px, py), path));
}

double path3_area(rust::Slice<const int64_t> xs, rust::Slice<const int64_t> ys) {
    return make_polygon(xs, ys).area();
}

bool path3_is_counter_clockwise(rust::Slice<const int64_t> xs, rust::Slice<const int64_t> ys) {
    return make_polygon(xs, ys).is_counter_clockwise();
}

rust::Vec<int64_t> path3_centroid(rust::Slice<const int64_t> xs, rust::Slice<const int64_t> ys) {
    Slic3r::Point c = make_polygon(xs, ys).centroid();
    rust::Vec<int64_t> out;
    out.push_back(static_cast<int64_t>(c.x()));
    out.push_back(static_cast<int64_t>(c.y()));
    return out;
}

bool path3_contains(rust::Slice<const int64_t> xs, rust::Slice<const int64_t> ys, int64_t px,
                     int64_t py) {
    return make_polygon(xs, ys).contains(Slic3r::Point(px, py));
}

rust::Vec<int64_t> path3_point_projection(rust::Slice<const int64_t> xs,
                                           rust::Slice<const int64_t> ys, int64_t px, int64_t py) {
    Slic3r::Point proj = make_polygon(xs, ys).point_projection(Slic3r::Point(px, py));
    rust::Vec<int64_t> out;
    out.push_back(static_cast<int64_t>(proj.x()));
    out.push_back(static_cast<int64_t>(proj.y()));
    return out;
}

bool path3_polygon_is_convex(rust::Slice<const int64_t> xs, rust::Slice<const int64_t> ys) {
    Slic3r::Points pts;
    pts.reserve(xs.size());
    for (size_t i = 0; i < xs.size(); ++i)
        pts.emplace_back(xs[i], ys[i]);
    return Slic3r::polygon_is_convex(pts);
}

rust::Vec<int64_t> path3_make_circle_num_segments(double radius, uint64_t num_segments) {
    Slic3r::Polygon out = Slic3r::make_circle_num_segments(radius, static_cast<size_t>(num_segments));
    rust::Vec<int64_t> result;
    result.reserve(out.points.size() * 2);
    for (const Slic3r::Point &p : out.points) {
        result.push_back(static_cast<int64_t>(p.x()));
        result.push_back(static_cast<int64_t>(p.y()));
    }
    return result;
}

rust::Vec<int64_t> path3_remove_sticks(rust::Slice<const int64_t> xs,
                                        rust::Slice<const int64_t> ys) {
    Slic3r::Polygon poly = make_polygon(xs, ys);
    bool modified = Slic3r::remove_sticks(poly);
    rust::Vec<int64_t> out;
    out.push_back(modified ? 1 : 0);
    for (const Slic3r::Point &p : poly.points) {
        out.push_back(static_cast<int64_t>(p.x()));
        out.push_back(static_cast<int64_t>(p.y()));
    }
    return out;
}

rust::Vec<float> path3_parameter_by_length(rust::Slice<const int64_t> xs,
                                            rust::Slice<const int64_t> ys) {
    std::vector<float> lengths = make_polygon(xs, ys).parameter_by_length();
    rust::Vec<float> out;
    out.reserve(lengths.size());
    for (float f : lengths)
        out.push_back(f);
    return out;
}

rust::Vec<int64_t> path3_densify(rust::Slice<const int64_t> xs, rust::Slice<const int64_t> ys,
                                  float min_length) {
    Slic3r::Polygon poly = make_polygon(xs, ys);
    poly.densify(min_length);
    return pack_points(poly.points);
}

} // namespace polygon
} // namespace proof_units
} // namespace slic3r_ffi
