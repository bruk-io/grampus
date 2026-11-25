// The plain Clipper2Lib surface: this translation unit includes only <clipper2/clipper.h>, never
// <clipper2/clipper2_z.hpp> -- see src/clipper2.rs's module doc.
#include "cxx/clipper2_shim.h"

#include <clipper2/clipper.h>

namespace slic3r_ffi {
namespace clipper2 {

namespace {

Clipper2Lib::Paths64 to_paths64(const rust::Vec<CPath64> &paths) {
    Clipper2Lib::Paths64 out;
    out.reserve(paths.size());
    for (const CPath64 &path : paths) {
        Clipper2Lib::Path64 p;
        p.reserve(path.points.size());
        for (const CPoint64 &pt : path.points)
            p.emplace_back(pt.x, pt.y);
        out.push_back(std::move(p));
    }
    return out;
}

rust::Vec<CPath64> from_paths64(const Clipper2Lib::Paths64 &paths) {
    rust::Vec<CPath64> out;
    for (const Clipper2Lib::Path64 &path : paths) {
        CPath64 p;
        for (const Clipper2Lib::Point64 &pt : path)
            p.points.push_back(CPoint64{pt.x, pt.y});
        out.push_back(std::move(p));
    }
    return out;
}

} // namespace

// Clipper2Lib::Clipper64 over the subject paths with no clip paths, the way
// src/libslic3r/ClipperUtils.cpp:808 unions a single path set: `c2_clip(C2::ClipType::Union,
// to_paths64(...), {}, to_c2(fillType), out)`.
rust::Vec<CPath64> clipper2_union(const rust::Vec<CPath64> &subject) {
    Clipper2Lib::Clipper64 clipper;
    clipper.AddSubject(to_paths64(subject));
    Clipper2Lib::Paths64 solution;
    clipper.Execute(Clipper2Lib::ClipType::Union, Clipper2Lib::FillRule::NonZero, solution);
    return from_paths64(solution);
}

// Clipper2Lib::ClipperOffset with the library's own defaults
// (src/libslic3r/ClipperUtils.hpp:31,33: DefaultJoinType = jtMiter, DefaultEndType = etOpenButt
// for open paths, but offsetting a closed polygon on both sides is EndType::Polygon, as
// ClipperUtils.cpp's offset() overloads use for a closed subject).
rust::Vec<CPath64> clipper2_offset(const rust::Vec<CPath64> &subject, double delta) {
    Clipper2Lib::ClipperOffset co;
    co.AddPaths(to_paths64(subject), Clipper2Lib::JoinType::Miter, Clipper2Lib::EndType::Polygon);
    Clipper2Lib::Paths64 solution;
    co.Execute(delta, solution);
    return from_paths64(solution);
}

} // namespace clipper2
} // namespace slic3r_ffi
