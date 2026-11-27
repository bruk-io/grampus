// Proof shim for the `Geometry/Bicubic.hpp` unit (R19, proof path 1, task
// `76-foundation-voronoi-curves`), compiled onto `crates/slic3r-ffi`'s shared `cxx_build::Build`
// by this crate's `build.rs` -- the one sanctioned route a ported unit reaches the oracle by
// (`docs/PORTING.md`, "Decisions recorded on 2026-11-25").
//
// `CubicKernelWrapper<Kernel>::kernel(float)` is header-only; this translation unit instantiates
// it directly at `Kernel = BicubicInternal::CubicBSplineKernel<float>`, the one instantiation any
// in-scope caller reaches (see `crates/slic3r-geometry-dir/src/bicubic.rs`'s module doc).
#include "bicubic.h"

#include "Geometry/Bicubic.hpp"

namespace slic3r_ffi {
namespace proof_units {
namespace bicubic {

double path1_cubic_bspline_kernel(double x) {
    using Kernel = Slic3r::Geometry::CubicBSplineKernel<float>;
    return static_cast<double>(Kernel::kernel(static_cast<float>(x)));
}

} // namespace bicubic
} // namespace proof_units
} // namespace slic3r_ffi
