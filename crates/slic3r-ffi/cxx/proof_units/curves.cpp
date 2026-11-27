// Proof shim for the `Geometry/Curves.hpp` unit (R19, proof path 1, task
// `76-foundation-voronoi-curves`), compiled onto `crates/slic3r-ffi`'s shared `cxx_build::Build`
// by this crate's `build.rs`.
//
// `PiecewiseFittedCurve<Dimension, NumberType, KernelType>::get_fitted_value` is header-only;
// this translation unit instantiates it directly at `Dimension = 2`, `NumberType = float`,
// `KernelType = CubicBSplineKernel<float>`, the one instantiation any in-scope caller reaches
// (see `crates/slic3r-geometry-dir/src/curves.rs`'s module doc). The object's fields are
// assigned directly, never through `fit_curve`/`fit_cubic_bspline`, the same reason
// `bounding_box.cpp`'s module doc gives for assigning a shim's fields directly rather than
// through a constructor whose own invariants could override the state under test.
#include "curves.h"

#include "Geometry/Curves.hpp"

namespace slic3r_ffi {
namespace proof_units {
namespace curves {

rust::Vec<double> path1_get_fitted_value(rust::Vec<double> coefficients_flat, uint64_t cols,
                                          double start, double segment_size,
                                          uint64_t endpoints_level_of_freedom,
                                          double observation_point) {
    using Kernel = Slic3r::Geometry::CubicBSplineKernel<float>;
    using Curve = Slic3r::Geometry::PiecewiseFittedCurve<2, float, Kernel>;

    Curve curve{};
    curve.coefficients.resize(2, static_cast<Eigen::Index>(cols));
    for (uint64_t j = 0; j < cols; ++j) {
        curve.coefficients(0, static_cast<Eigen::Index>(j)) =
            static_cast<float>(coefficients_flat[j * 2 + 0]);
        curve.coefficients(1, static_cast<Eigen::Index>(j)) =
            static_cast<float>(coefficients_flat[j * 2 + 1]);
    }
    curve.start = static_cast<float>(start);
    curve.segment_size = static_cast<float>(segment_size);
    curve.endpoints_level_of_freedom = static_cast<size_t>(endpoints_level_of_freedom);

    auto v = curve.get_fitted_value(static_cast<float>(observation_point));

    rust::Vec<double> out;
    out.push_back(static_cast<double>(v.x()));
    out.push_back(static_cast<double>(v.y()));
    return out;
}

} // namespace curves
} // namespace proof_units
} // namespace slic3r_ffi
