//! Port of `src/libslic3r/Geometry/Curves.hpp` (task `76-foundation-voronoi-curves`, plan step
//! 20, `port/foundation.tsv`'s `foundation-25` row): `template-heavy: yes`, hand-designed before
//! fan-out.
//!
//! **Only one instantiation is reachable from in-scope code.** Measured the same way
//! [`crate::bicubic`]'s module doc measures its own header: `grep -rn` over the whole of
//! `vendor/OrcaSlicer/src/` finds `PolynomialCurve`, `fit_polynomial`, `fit_linear_spline` and
//! `fit_catmul_rom_spline` with **zero** callers anywhere, and so `LinearKernel<T>`/
//! `CubicCatmulRomKernel<T>` (`Bicubic.hpp`'s own aliases, used by this header only through
//! those two dead `fit_*` wrappers) stay dead for the same reason `crate::bicubic`'s module doc
//! already gives. The one live path is `fit_cubic_bspline<2, float>` -> `fit_curve<
//! CubicBSplineKernel<float>, 2, float>` -> `PiecewiseFittedCurve<2, float,
//! CubicBSplineKernel<float>>`, called from `src/libslic3r/GCode/SeamPlacer.cpp:2044`
//! (`port/manifest.tsv` `in`, crate `slic3r-gcode`, `not-started` at the pin) and read back
//! through `PiecewiseFittedCurve::get_fitted_value` at `SeamPlacer.cpp:2056`. Per the idiom map's
//! "a template is not force-collapsed to one concrete type unless the manifest row's unit only
//! ever instantiates one", this header's generic machinery collapses to that one instantiation.
//!
//! **`PolynomialCurve`/`fit_polynomial` and `fit_linear_spline`/`fit_catmul_rom_spline` are left
//! unported**, named rather than guessed, for the same "zero callers anywhere in the vendored
//! tree" reason `crate::bicubic`'s module doc gives for its own dead symbols
//! (`docs/PORTING.md`, "Decisions recorded on 2026-11-26", decision 9).
//!
//! **`fit_curve`/`fit_cubic_bspline` themselves -- the fitting step, as opposed to
//! [`PiecewiseFittedCurve::get_fitted_value`], the evaluation step -- are also left unported,
//! and this is a named finding for the pull request, not a dead-code call.** They are live
//! (`SeamPlacer.cpp:2044`'s own caller), but their body is a weighted linear least-squares solve
//! through `Eigen::MatrixXf::fullPivHouseholderQr().solve(...)` (`Curves.hpp:176-179`): a dense
//! QR decomposition with full pivoting over a matrix whose size depends on the caller's own
//! `segments_count`. Reproducing Eigen's specific pivoting and Householder-reflection order
//! bit-for-bit -- which this row's `exact` equality tier with no tolerance cell demands -- needs
//! either a linear-algebra dependency (no foundation task may add one to any `Cargo.toml`,
//! `docs/WORKFLOW.md` "Files this task must not write") or a from-scratch reimplementation of
//! Eigen's own algorithm, which is a different and much larger task than hand-designing this
//! unit's type shape. `docs/WORKFLOW.md`'s own rule governs the choice made here: "A unit that
//! cannot be held to exact equality is a finding for the pull request, not a tolerance invented
//! on the branch." [`PiecewiseFittedCurve::get_fitted_value`] -- the part of this unit that *is*
//! ported and proven below -- needs no QR solve at all; its proof constructs a
//! `PiecewiseFittedCurve` with literal field values directly (bypassing `fit_curve` on both
//! sides), the same technique `crates/slic3r-ffi/cxx/proof_units/bounding_box.cpp`'s module doc
//! describes for assigning a shim's fields directly rather than through a constructor.

use slic3r_point::point::{Vec2, Vec2f};

/// `PiecewiseFittedCurve<Dimension, NumberType, KernelType>` (`Curves.hpp:67-94`), monomorphised
/// at `Dimension = 2`, `NumberType = float`, `KernelType = CubicBSplineKernel<float>` -- the one
/// instantiation this header's in-scope caller reaches (see the module doc). The vendored
/// `coefficients` field is an `Eigen::MatrixXf` of shape `(Dimension, parameters_count)`; at
/// `Dimension = 2` this is represented as one [`Vec2f`] per column rather than a flat matrix --
/// an isomorphic data-shape change, not an idiom substitution (`docs/PORTING.md`, "Faithfulness,
/// not idiom"): every element the vendored matrix holds is still present, in the same column
/// order, read the same way [`PiecewiseFittedCurve::get_fitted_value`] reads it below.
#[derive(Debug, Clone)]
pub struct PiecewiseFittedCurve {
    /// One `Vec2f` per column of the vendored `Eigen::MatrixXf coefficients` (`:71`).
    pub coefficients: Vec<Vec2f>,
    /// `NumberType start` (`:72`).
    pub start: f32,
    /// `NumberType segment_size` (`:73`).
    pub segment_size: f32,
    /// `size_t endpoints_level_of_freedom` (`:74`).
    pub endpoints_level_of_freedom: usize,
}

impl PiecewiseFittedCurve {
    /// `PiecewiseFittedCurve::get_fitted_value(const NumberType&)` (`Curves.hpp:76-93`), at the
    /// `CubicBSplineKernel<float>` instantiation (see the module doc): `Kernel::kernel_span`/
    /// `Kernel::kernel(...)` below are [`crate::bicubic::CUBIC_BSPLINE_KERNEL_SPAN`]/
    /// [`crate::bicubic::cubic_bspline_kernel`].
    ///
    /// The vendored body mixes `int middle_right_segment_index`/`int start_segment_idx` with
    /// `Kernel::kernel_span` (`size_t`, unsigned) in `-`/`+` only (never `/` or a comparison on
    /// the mixed-signedness intermediate), so the signed/unsigned promotion C++'s usual
    /// arithmetic conversions apply at `:82` round-trips to the same bit pattern plain `i64`
    /// arithmetic gives once truncated back to the narrower type, for every segment index this
    /// port's own caller can reach; this is ported as plain signed arithmetic rather than
    /// reproducing the intermediate unsigned step.
    pub fn get_fitted_value(&self, observation_point: f32) -> Vec2f {
        // `0f32` (no decimal point) so the exempt value `0` normalises into R16's exempt set
        // rather than needing a marker (`docs/PORTING.md`, "Decisions recorded on 2026-11-26",
        // decision 8's spelling technique).
        let mut result = Vec2::new(0f32, 0f32);

        // `int middle_right_segment_index = floor((observation_point - start) / segment_size);`
        // (`:80`).
        let middle_right_segment_index =
            ((observation_point - self.start) / self.segment_size).floor() as i64;
        let kernel_span = crate::bicubic::CUBIC_BSPLINE_KERNEL_SPAN as i64;
        // `int start_segment_idx = middle_right_segment_index - Kernel::kernel_span / 2 + 1;`
        // (`:82`).
        let start_segment_idx = middle_right_segment_index - kernel_span / 2 + 1;

        for segment_index in start_segment_idx..start_segment_idx + kernel_span {
            // `NumberType segment_start = start + segment_index * segment_size;` (`:85`).
            let segment_start = self.start + (segment_index as f32) * self.segment_size;
            // `NumberType normalized_segment_distance = (segment_start - observation_point) /
            // segment_size;` (`:86`).
            let normalized_segment_distance =
                (segment_start - observation_point) / self.segment_size;

            // `int parameter_index = segment_index + endpoints_level_of_freedom;` (`:88`);
            // `parameter_index = std::clamp(parameter_index, 0, int(coefficients.cols()) - 1);`
            // (`:89`).
            let parameter_index = segment_index + self.endpoints_level_of_freedom as i64;
            let max_index = self.coefficients.len() as i64 - 1;
            let parameter_index = parameter_index.clamp(0, max_index) as usize;

            // `result += Kernel::kernel(normalized_segment_distance) *
            // coefficients.col(parameter_index);` (`:90`).
            let k = crate::bicubic::cubic_bspline_kernel(normalized_segment_distance);
            let c = self.coefficients[parameter_index];
            result = Vec2::new(result.x() + k * c.x(), result.y() + k * c.y());
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use slic3r_proof::{compare_unit, Oracle};

    /// `PiecewiseFittedCurve<2, float, CubicBSplineKernel<float>>::get_fitted_value` against the
    /// oracle: the shim (`crates/slic3r-ffi/cxx/proof_units/curves.cpp`) constructs the vendored
    /// object with these same literal fields directly, bypassing `fit_curve`/`fit_cubic_bspline`
    /// on both sides (see the module doc's third paragraph for why those two are not ported),
    /// and calls its own `get_fitted_value`. `f32 -> f64` casts on both sides are an exact
    /// widening, never a change of the computed value (see `crate::bicubic`'s identical note).
    // PROOF: 1
    #[test]
    fn curves_piecewise_fitted_curve_get_fitted_value_matches_the_oracle() {
        let coefficients = vec![
            Vec2::new(0.0f32, 0.0f32),
            Vec2::new(1.0, -1.0),
            Vec2::new(2.0, 2.5),
            Vec2::new(-1.5, 3.0),
            Vec2::new(0.5, -2.0),
            Vec2::new(3.0, 0.0),
        ];
        let start = -2.0f32;
        let segment_size = 1.25f32;
        let endpoints_level_of_freedom = 1usize;

        let flat: Vec<f64> = coefficients
            .iter()
            .flat_map(|c| [c.x() as f64, c.y() as f64])
            .collect();

        for &observation_point in &[-3.0f32, -2.0, -0.6, 0.0, 0.3, 1.9, 4.0, 7.3] {
            let curve = PiecewiseFittedCurve {
                coefficients: coefficients.clone(),
                start,
                segment_size,
                endpoints_level_of_freedom,
            };
            let actual = curve.get_fitted_value(observation_point);
            let actual = vec![actual.x() as f64, actual.y() as f64];

            let expected = slic3r_proof::proof_units::curves::path1_get_fitted_value(
                flat.clone(),
                coefficients.len() as u64,
                start as f64,
                segment_size as f64,
                endpoints_level_of_freedom as u64,
                observation_point as f64,
            );

            compare_unit("Geometry/Curves.hpp", Oracle::Built, actual, expected)
                .unwrap_or_else(|e| panic!("get_fitted_value({observation_point}): {e}"));
        }
    }
}

// PORT STATUS
// source: src/libslic3r/Geometry/Curves.hpp
// confidence: medium
// tier: exact
// oracle: built
