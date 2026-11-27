#pragma once

#include "slic3r-ffi/src/proof_units/curves.rs.h"

namespace slic3r_ffi {
namespace proof_units {
namespace curves {

// Path 1 (task 76-foundation-voronoi-curves): `Geometry/Curves.hpp`'s own header-only
// `PiecewiseFittedCurve<2, float, CubicBSplineKernel<float>>::get_fitted_value`, the object
// constructed directly with these literal fields in this translation unit's counterpart `.cpp`
// -- bypassing `fit_curve`/`fit_cubic_bspline` on both sides (see
// `crates/slic3r-geometry-dir/src/curves.rs`'s module doc for why those two are not ported).
// `coefficients_flat` is `[x0, y0, x1, y1, ...]`, `cols` columns.
rust::Vec<double> path1_get_fitted_value(rust::Vec<double> coefficients_flat, uint64_t cols,
                                          double start, double segment_size,
                                          uint64_t endpoints_level_of_freedom,
                                          double observation_point);

} // namespace curves
} // namespace proof_units
} // namespace slic3r_ffi
