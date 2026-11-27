#pragma once

// Declarations only (see cxx/clipper2_shim.h's comment for why this crate always splits the
// cxx-generated header from the hand-written declarations file).
#include "slic3r-ffi/src/proof_units/bicubic.rs.h"

namespace slic3r_ffi {
namespace proof_units {
namespace bicubic {

// Path 1 (task 76-foundation-voronoi-curves): `Geometry/Bicubic.hpp`'s own header-only
// `CubicKernelWrapper<BicubicInternal::CubicBSplineKernel<float>>::kernel(float)`, instantiated
// directly in this translation unit's counterpart `.cpp` -- the manifest row's `proof path` cell
// is `1`, and the per-unit bridge is that row's oracle route regardless of whether the symbol is
// externally linked or a header-only template (`docs/PORTING.md`, "Decisions recorded on
// 2026-11-26", decision 2).
double path1_cubic_bspline_kernel(double x);

} // namespace bicubic
} // namespace proof_units
} // namespace slic3r_ffi
