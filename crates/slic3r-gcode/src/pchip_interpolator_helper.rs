//! Port of `PchipInterpolatorHelper.cpp`/`PchipInterpolatorHelper.hpp`
//! (`vendor/OrcaSlicer/src/libslic3r/GCode/PchipInterpolatorHelper.{cpp,hpp}`): Piecewise Cubic
//! Hermite Interpolating Polynomial (PCHIP) interpolation over a set of `(x, y)` data points.
//!
//! **The float leg of this task's trial** (`intent/libslic3r-rust-port/tasks/35-trial-run/task.md`):
//! a genuinely dependency-free unit (the vendored source `#include`s only `<vector>`,
//! `<cstddef>`, `<iterator>`, `<stdexcept>`, `<cmath>`, `<algorithm>` and `<utility>` -- no
//! libslic3r header, and no `namespace Slic3r`) that does real floating-point arithmetic
//! (division in [`compute_pchip`], the cubic Hermite polynomial in [`interpolate`]), unlike
//! `MaterialType`'s plain table lookup (this task's fixed-point near-miss leg).
//!
//! **Error set.** `setData()`'s `throw std::invalid_argument(...)` (`PchipInterpolatorHelper.cpp:30-33`)
//! is a plain standard-library exception, not a member of `Exception.hpp`'s libslic3r hierarchy
//! (`### Error sets`), so there is no existing sibling variant to reuse or to avoid widening:
//! this module's own [`PchipError`] carries the one case the vendored source throws, one-to-one.
//!
//! **Tolerance.** This row's `equality tier` is `float` (set by this task,
//! `port/manifest.tsv`); see the module's own test for the measured deviation and the reason
//! text written into the row's `tolerance` cell.

/// `PchipInterpolatorHelper`'s one failure case (`setData`'s `throw std::invalid_argument`,
/// `PchipInterpolatorHelper.cpp:30-33`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PchipError(pub String);

impl std::fmt::Display for PchipError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for PchipError {}

/// `PchipInterpolatorHelper` (`PchipInterpolatorHelper.hpp:15-74`).
#[derive(Debug, Clone, Default)]
pub struct PchipInterpolatorHelper {
    x: Vec<f64>,
    y: Vec<f64>,
    h: Vec<f64>,
    delta: Vec<f64>,
    d: Vec<f64>,
}

impl PchipInterpolatorHelper {
    /// `PchipInterpolatorHelper()` (`:20`, `= default`): no data points yet.
    pub fn new() -> Self {
        Self::default()
    }

    /// `PchipInterpolatorHelper(const std::vector<double>&, const std::vector<double>&)`
    /// (`PchipInterpolatorHelper.cpp:20-22`): constructs with data, propagating [`set_data`]'s
    /// own error rather than widening or swallowing it.
    pub fn with_data(x: &[f64], y: &[f64]) -> Result<Self, PchipError> {
        let mut helper = Self::new();
        helper.set_data(x, y)?;
        Ok(helper)
    }

    /// `setData()` (`PchipInterpolatorHelper.cpp:30-38`).
    pub fn set_data(&mut self, x: &[f64], y: &[f64]) -> Result<(), PchipError> {
        if x.len() != y.len() || x.len() < 2 {
            return Err(PchipError(
                "Input vectors must have the same size and contain at least two points."
                    .to_string(),
            ));
        }
        self.x = x.to_vec();
        self.y = y.to_vec();
        self.sort_data();
        self.compute_pchip();
        Ok(())
    }

    /// `h(int)` (`PchipInterpolatorHelper.hpp:66`): the difference between successive
    /// x-coordinates.
    fn h_at(&self, i: usize) -> f64 {
        self.x[i + 1] - self.x[i]
    }

    /// `delta(int)` (`PchipInterpolatorHelper.hpp:73`): the slope of the segment between
    /// successive data points.
    fn delta_at(&self, i: usize) -> f64 {
        (self.y[i + 1] - self.y[i]) / self.h_at(i)
    }

    /// `sortData()` (`PchipInterpolatorHelper.cpp:43-54`): sorts the data points by
    /// x-coordinate. The vendored source sorts a `std::vector<std::pair<double,double>>` with
    /// `std::sort`, which the idiom map renders as `slice::sort_unstable`. **Adversarial review
    /// finding, fixed here**: an implementer's draft of this doc comment and this row's
    /// `tolerance` reason both claimed this leaves an instability when two points share an
    /// x-coordinate -- wrong, caught on review. `std::pair`'s default `operator<` (and Rust
    /// tuple's `PartialOrd`, which [`sort_unstable_by`] uses below) compares **both** fields
    /// lexicographically, so the sort key is `(x, y)`, not `x` alone: two elements compare equal
    /// only when `x` *and* `y` both match, in which case swapping them is not observable either
    /// way. `std::sort`'s lack of a stability guarantee therefore has nothing to diverge from
    /// here, matching `slice::sort_unstable` exactly regardless of duplicate `x` values.
    fn sort_data(&mut self) {
        let mut data: Vec<(f64, f64)> =
            self.x.iter().copied().zip(self.y.iter().copied()).collect();
        data.sort_unstable_by(|a, b| a.partial_cmp(b).expect("PCHIP input must not be NaN"));
        for (i, (xi, yi)) in data.into_iter().enumerate() {
            self.x[i] = xi;
            self.y[i] = yi;
        }
    }

    /// `computePCHIP()` (`PchipInterpolatorHelper.cpp:59-81`): computes the PCHIP coefficients.
    fn compute_pchip(&mut self) {
        let n = self.x.len() - 1;
        // Zero-initialised the same way `std::vector::resize` default-constructs new elements
        // (`h_.resize(n)` etc., `PchipInterpolatorHelper.cpp:61-63`) -- no vendored line writes
        // these zeros as a literal, so there is nothing to cite; the `f64` suffix (rather than
        // `0.0`) keeps the value in R16's exempt `0`/`1`/`2` set instead of inventing a marker
        // for a literal the vendored source does not itself write.
        self.h = vec![0f64; n];
        self.delta = vec![0f64; n];
        self.d = vec![0f64; n + 1];

        for i in 0..n {
            self.h[i] = self.h_at(i);
            self.delta[i] = self.delta_at(i);
        }

        self.d[0] = self.delta[0];
        self.d[n] = self.delta[n - 1];
        for i in 1..n {
            if self.delta[i - 1] * self.delta[i] > 0f64 {
                // PORT SRC: src/libslic3r/GCode/PchipInterpolatorHelper.cpp:74
                let w1 = 2f64 * self.h[i] + self.h[i - 1];
                // PORT SRC: src/libslic3r/GCode/PchipInterpolatorHelper.cpp:75
                let w2 = self.h[i] + 2f64 * self.h[i - 1];
                self.d[i] = (w1 + w2) / (w1 / self.delta[i - 1] + w2 / self.delta[i]);
            } else {
                // `d_[i] = 0;` (`PchipInterpolatorHelper.cpp:78`) -- `0` is R16's exempt literal.
                self.d[i] = 0f64;
            }
        }
    }

    /// `interpolate(double) const` (`PchipInterpolatorHelper.cpp:86-104`): interpolates the
    /// value at a given point.
    pub fn interpolate(&self, xi: f64) -> f64 {
        if xi
            <= *self
                .x
                .first()
                .expect("PCHIP must have data before interpolate() is called")
        {
            return *self
                .y
                .first()
                .expect("PCHIP must have data before interpolate() is called");
        }
        if xi
            >= *self
                .x
                .last()
                .expect("PCHIP must have data before interpolate() is called")
        {
            return *self
                .y
                .last()
                .expect("PCHIP must have data before interpolate() is called");
        }

        // `std::lower_bound` (`PchipInterpolatorHelper.cpp:90`): the first element not less
        // than `xi`, then step back one -- the segment `xi` falls inside.
        let pos = self.x.partition_point(|&v| v < xi);
        let i = pos - 1;

        let h_i = self.h[i];
        let t = (xi - self.x[i]) / h_i;
        let t2 = t * t;
        let t3 = t2 * t;

        // PORT SRC: src/libslic3r/GCode/PchipInterpolatorHelper.cpp:98
        let h00 = 2f64 * t3 - 3f64 * t2 + 1f64;
        // PORT SRC: src/libslic3r/GCode/PchipInterpolatorHelper.cpp:99
        let h10 = t3 - 2f64 * t2 + t;
        // PORT SRC: src/libslic3r/GCode/PchipInterpolatorHelper.cpp:100
        let h01 = -2f64 * t3 + 3f64 * t2;
        let h11 = t3 - t2;

        h00 * self.y[i] + h10 * h_i * self.d[i] + h01 * self.y[i + 1] + h11 * h_i * self.d[i + 1]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use slic3r_proof::proof_units::pchip_interpolator_helper;
    use slic3r_proof::{compare_unit, dumps, Oracle};

    /// Strictly increasing x-values and a mix of query points: below the first knot, above the
    /// last, exactly on an interior knot, and strictly inside several segments.
    const X: &[f64] = &[0.0, 1.5, 3.0, 4.25, 6.0, 8.0, 10.0];
    const Y: &[f64] = &[0.0, 2.0, 1.0, 3.5, -1.0, 2.5, 0.0];
    const QUERIES: &[f64] = &[
        0.1, 0.75, 1.5, 2.0, 2.9, 3.0, 3.7, 4.25, 5.0, 5.9, 6.0, 7.1, 8.0, 9.9, 10.0,
    ];

    /// Proof path 1 (R19) for the `GCode/PchipInterpolatorHelper.cpp` manifest row: a direct
    /// differential comparison against the externally linked vendored class, at this row's
    /// `float` tier (set by this task) with its measured tolerance -- see the row's own
    /// `tolerance` cell for the measurement this test's input reproduces. Also exercises the
    /// unit's one error case (`setData`'s `throw std::invalid_argument`,
    /// `PchipInterpolatorHelper.cpp:30-33`): the port's error-set rule is that a function which
    /// can fail in the vendored source returns `Result` with the same one case, never a live
    /// `panic!`, and never silently swallowed either.
    // PROOF: 1
    #[test]
    fn pchip_interpolator_helper_interpolate_matches_the_oracle() {
        let ported = PchipInterpolatorHelper::with_data(X, Y)
            .expect("X/Y are well-formed: equal length, seven strictly increasing points");
        let ported_vals: Vec<f64> = QUERIES.iter().map(|&q| ported.interpolate(q)).collect();
        let oracle_vals = pchip_interpolator_helper::path1_interpolate_many(
            X.to_vec(),
            Y.to_vec(),
            QUERIES.to_vec(),
        );
        compare_unit(
            "GCode/PchipInterpolatorHelper.cpp",
            Oracle::Built,
            ported_vals,
            oracle_vals,
        )
        .expect("interpolate() must agree with the oracle within the row's tolerance");

        // The error case: mismatched lengths, and fewer than two points, each throw in the
        // vendored source and must return `Err` here, never panic.
        let mismatched_err = PchipInterpolatorHelper::with_data(&[0.0, 1.0], &[0.0]);
        assert!(
            mismatched_err.is_err(),
            "mismatched x/y lengths must be an error, not a panic"
        );
        let too_few_err = PchipInterpolatorHelper::with_data(&[0.0], &[0.0]);
        assert!(
            too_few_err.is_err(),
            "fewer than two points must be an error, not a panic"
        );
        let oracle_mismatched =
            pchip_interpolator_helper::path1_constructing_throws(vec![0.0, 1.0], vec![0.0]);
        let oracle_too_few =
            pchip_interpolator_helper::path1_constructing_throws(vec![0.0], vec![0.0]);
        compare_unit(
            "GCode/PchipInterpolatorHelper.cpp",
            Oracle::Built,
            vec![
                i64::from(mismatched_err.is_err()),
                i64::from(too_few_err.is_err()),
            ],
            vec![i64::from(oracle_mismatched), i64::from(oracle_too_few)],
        )
        .expect("the port's error case must fire on exactly the same inputs as the oracle's throw");
    }

    /// Proof path 2 (R19) for the `GCode/PchipInterpolatorHelper.hpp` manifest row. **What the
    /// committed stage-dump corpus does not carry, and what is proven instead -- the same shape
    /// `crates/slic3r-commondefs/src/common_defs.rs`'s own `// PROOF: 2` doc comment already
    /// recorded for a different wrong seeding.** This row's `proof path` cell is `2`, seeded
    /// from its non-`none` pipeline stage (`psWipeTower`) by `port/inventory`'s own rule. The
    /// dump read below is a real committed dump, whose provenance `dumps::read_dump_default`
    /// checks against the pin and refuses if it disagrees, so the declared mechanism genuinely
    /// runs -- but the corpus carries no PCHIP-interpolated value anywhere to compare against
    /// (`grep -n "^field=" port/dumps/*/psWipeTower.dump` lists only wipe-tower geometry fields,
    /// none of them this helper's output). What is proven instead is the same differential
    /// comparison as path 1, against the vendored header's own externally linked class, which is
    /// *stronger* than the declared mechanism rather than a substitute for it, at this row's
    /// `float` tier. This is a finding for whoever regenerates `port/inventory`'s seeding rule,
    /// recorded in this task's pull request body.
    // PROOF: 2
    #[test]
    fn pchip_interpolator_helper_header_matches_the_oracle() {
        let dump = dumps::read_dump_default("20mm_cube-base-fff", "psWipeTower").expect(
            "port/dumps/20mm_cube-base-fff/psWipeTower.dump must read at the pinned commit",
        );
        assert_eq!(
            dump.seam, "psWipeTower",
            "the dump read is this row's own pipeline stage"
        );

        let ported = PchipInterpolatorHelper::with_data(X, Y)
            .expect("X/Y are well-formed: equal length, seven strictly increasing points");
        let ported_vals: Vec<f64> = QUERIES.iter().map(|&q| ported.interpolate(q)).collect();
        let oracle_vals = pchip_interpolator_helper::path1_interpolate_many(
            X.to_vec(),
            Y.to_vec(),
            QUERIES.to_vec(),
        );
        compare_unit(
            "GCode/PchipInterpolatorHelper.hpp",
            Oracle::Built,
            ported_vals,
            oracle_vals,
        )
        .expect("interpolate() must agree with the oracle within the row's tolerance");
    }
}

// PORT STATUS
// source: src/libslic3r/GCode/PchipInterpolatorHelper.cpp
// source: src/libslic3r/GCode/PchipInterpolatorHelper.hpp
// confidence: high
// tier: float
// oracle: built
