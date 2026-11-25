#pragma once

// Declarations only: never includes a Clipper2 header (see src/clipper2.rs's module doc for why
// the two namespaces cannot share a translation unit). cxx_build::bridge("src/clipper2.rs")
// generates this header's counterpart, the shared struct definitions (CPoint64, CPath64,
// CPointZ, CPathZ) and rust::Vec, at <crate-name>/src/clipper2.rs.h on the include path it sets
// up; cxx/clipper2_plain.cc and cxx/clipper2_z.cc are the two translation units that implement
// the functions below, each including exactly one of Clipper2's two namespace headers.
#include "slic3r-ffi/src/clipper2.rs.h"

namespace slic3r_ffi {
namespace clipper2 {

rust::Vec<CPath64> clipper2_union(const rust::Vec<CPath64> &subject);
rust::Vec<CPath64> clipper2_offset(const rust::Vec<CPath64> &subject, double delta);
rust::Vec<CPathZ> clipper2_z_union(const rust::Vec<CPathZ> &subject);

} // namespace clipper2
} // namespace slic3r_ffi
