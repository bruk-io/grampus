// The Clipper2Lib_Z surface: this translation unit includes only <clipper2/clipper2_z.hpp>, never
// <clipper2/clipper.h> -- see src/clipper2.rs's module doc.
#include "cxx/clipper2_shim.h"

#include <clipper2/clipper2_z.hpp>

namespace slic3r_ffi {
namespace clipper2 {

namespace {

Clipper2Lib_Z::Paths64 to_paths64_z(const rust::Vec<CPathZ> &paths) {
    Clipper2Lib_Z::Paths64 out;
    out.reserve(paths.size());
    for (const CPathZ &path : paths) {
        Clipper2Lib_Z::Path64 p;
        p.reserve(path.points.size());
        for (const CPointZ &pt : path.points)
            p.emplace_back(pt.x, pt.y, pt.z);
        out.push_back(std::move(p));
    }
    return out;
}

rust::Vec<CPathZ> from_paths64_z(const Clipper2Lib_Z::Paths64 &paths) {
    rust::Vec<CPathZ> out;
    for (const Clipper2Lib_Z::Path64 &path : paths) {
        CPathZ p;
        for (const Clipper2Lib_Z::Point64 &pt : path)
            p.points.push_back(CPointZ{pt.x, pt.y, pt.z});
        out.push_back(std::move(p));
    }
    return out;
}

} // namespace

// Clipper2Lib_Z::Clipper64 over the subject paths with no clip paths and no Z-fill callback --
// the surface src/libslic3r/ClipperZUtils.cpp's clip_zpaths() wraps (ClipperZUtils.cpp:48-77),
// minus the callback this bridge does not wire yet.
rust::Vec<CPathZ> clipper2_z_union(const rust::Vec<CPathZ> &subject) {
    Clipper2Lib_Z::Clipper64 clipper;
    clipper.AddSubject(to_paths64_z(subject));
    Clipper2Lib_Z::Paths64 solution;
    clipper.Execute(Clipper2Lib_Z::ClipType::Union, Clipper2Lib_Z::FillRule::NonZero, solution);
    return from_paths64_z(solution);
}

} // namespace clipper2
} // namespace slic3r_ffi
