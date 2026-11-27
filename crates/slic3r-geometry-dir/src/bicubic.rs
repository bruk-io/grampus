//! Port of `src/libslic3r/Geometry/Bicubic.hpp` (task `76-foundation-voronoi-curves`, plan step
//! 20, part of `port/foundation.tsv`'s `foundation-25` row): `template-heavy: yes`,
//! hand-designed before fan-out, as `docs/WORKFLOW.md` "Unit of work" states.
//!
//! **Only one instantiation of this header's machinery is reachable from in-scope code at all.**
//! Measured with `grep -rn` over the whole of `vendor/OrcaSlicer/` -- `src/` (not just
//! `libslic3r/`), `tests/`, `sandboxes/` and `tools/`, the only directories that could plausibly
//! name a libslic3r symbol:
//! `BicubicInternal::LinearKernel<T>` and `BicubicInternal::CubicCatmulRomKernel<T>` (and the
//! `Geometry::LinearKernel`/`Geometry::CubicCatmulRomKernel` aliases over them), `CubicKernelWrapper::interpolate`,
//! and the two free functions `cubic_interpolate`/`bicubic_interpolate` (`:241-287`) have **zero**
//! callers anywhere in the vendored tree -- not even in this unit's own sibling
//! `Geometry/Curves.hpp`, which calls only `Kernel::kernel`/`Kernel::kernel_span` (see that
//! module's own doc comment). The one path that is reachable:
//! `Geometry::CubicBSplineKernel<float>::kernel()`, through `CubicKernelWrapper<BicubicInternal::
//! CubicBSplineKernel<float>>::kernel(float)`, reached transitively via `Curves.hpp`'s
//! `fit_cubic_bspline`/`PiecewiseFittedCurve::get_fitted_value`, itself called at
//! `src/libslic3r/GCode/SeamPlacer.cpp:2044` (`port/manifest.tsv` marks that file `in`, crate
//! `slic3r-gcode`, `not-started` at the pin -- `observations`/`weights` there are
//! `std::vector<Vec2f>`, i.e. `Dimension = 2`, `NumberType = float`).
//!
//! `docs/PORTING.md`'s idiom-map "A template" row: "a template is not force-collapsed to one
//! concrete type unless the manifest row's unit only ever instantiates one." This unit's
//! `CubicKernelWrapper` instantiates, across the whole in-scope tree, at exactly one concrete
//! type -- `BicubicInternal::CubicBSplineKernel<float>` -- so this module ports that one concrete
//! instantiation directly rather than a Rust generic nobody ever calls at a second type.
//!
//! **Left unported, named here rather than guessed**, following the precedent
//! `docs/PORTING.md`'s "Decisions recorded on 2026-11-26", decision 9, states for
//! `intersect_triangle_epsilon` ("dead code that cannot be instantiated has no behaviour to
//! preserve"): `BicubicInternal::LinearKernel`/`CubicCatmulRomKernel` and the `CubicKernelWrapper`
//! instantiations over them, `CubicKernelWrapper::interpolate` (used only by the two functions
//! below), and `cubic_interpolate`/`bicubic_interpolate` themselves. Unlike
//! `intersect_triangle_epsilon` these compile cleanly if instantiated -- the read here is "has no
//! behaviour that is ever exercised", since zero in-scope code ever instantiates them, which is
//! the same practical conclusion decision 9 reaches by a different route. This is a finding named
//! in this task's pull request, not a tolerance or a stand-in.

/// `BicubicInternal::CubicBSplineKernel<float>::a00()..a33()` (`Bicubic.hpp:135-182`), the
/// B-spline kernel's 4x4 coefficient table. The vendored type keeps each coefficient as its own
/// zero-argument static accessor (dispatched generically alongside two sibling kernels that are
/// never instantiated, per the module doc above); `docs/PORTING.md`'s "Faithfulness, not idiom"
/// licenses folding that table directly into the one formula that reads it,
/// [`cubic_bspline_kernel`] below, rather than carrying the C++ dispatch shape into Rust for a
/// kernel that resolves to one concrete type. `a11`, `a30`, `a31` and `a32` are the literal `0`
/// R16 exempts, so they carry no `// PORT SRC:` marker.
// PORT SRC: src/libslic3r/Geometry/Bicubic.hpp:136
const A00: f32 = 1. / 6.;
// PORT SRC: src/libslic3r/Geometry/Bicubic.hpp:139
const A01: f32 = -3. / 6.;
// PORT SRC: src/libslic3r/Geometry/Bicubic.hpp:142
const A02: f32 = 3. / 6.;
// PORT SRC: src/libslic3r/Geometry/Bicubic.hpp:145
const A03: f32 = -1. / 6.;
// PORT SRC: src/libslic3r/Geometry/Bicubic.hpp:148
const A10: f32 = 4. / 6.;
// `a11` is the literal `0` R16 exempts, spelled `0f32` (no decimal point) rather than `0.` so
// its normalised text lands in the exempt set (`docs/PORTING.md`, "Decisions recorded on
// 2026-11-26", decision 8's spelling technique, applied here to an exempt value rather than a
// transcribed one).
const A11: f32 = 0f32;
// PORT SRC: src/libslic3r/Geometry/Bicubic.hpp:154
const A12: f32 = -6. / 6.;
// PORT SRC: src/libslic3r/Geometry/Bicubic.hpp:157
const A13: f32 = 3. / 6.;

/// `CubicKernelWrapper<Kernel>::kernel_span` (`Bicubic.hpp:198`), `size_t kernel_span = 4`.
// PORT SRC: src/libslic3r/Geometry/Bicubic.hpp:198
pub const CUBIC_BSPLINE_KERNEL_SPAN: usize = 4;

/// `CubicKernelWrapper<BicubicInternal::CubicBSplineKernel<float>>::kernel(float)`
/// (`Bicubic.hpp:200-215`), at the one instantiation reachable from in-scope code (see the module
/// doc). Every operation is a plain `float` op, never widened to `f64` mid-formula
/// (`docs/PORTING.md`, "Float widths").
pub fn cubic_bspline_kernel(x: f32) -> f32 {
    let x = x.abs();
    // Every literal below is transcribed from a vendored line that spells it with a decimal
    // point (`2.`, `0.0f`, `1.`), so each keeps that decimal spelling plus its own
    // `// PORT SRC:` marker -- `docs/PORTING.md`, "Decisions recorded on 2026-11-26", decision 8
    // forbids relying on the `0`/`1`/`2` exempt spelling (`2f32`, `1f32`, ...) "as a way to skip
    // writing a marker for a value that *is* transcribed from a vendored line"; that licence is
    // only for a value the vendored line itself spells as a bare integer (see
    // `CircleSq::make_invalid` in `circle.rs` for that other case).
    // PORT SRC: src/libslic3r/Geometry/Bicubic.hpp:203
    if x >= 2. {
        // PORT SRC: src/libslic3r/Geometry/Bicubic.hpp:204
        return 0.0;
    }
    // PORT SRC: src/libslic3r/Geometry/Bicubic.hpp:205
    if x <= 1. {
        let x2 = x * x;
        let x3 = x2 * x;
        return A10 + A11 * x + A12 * x2 + A13 * x3;
    }
    // `assert(x > (FloatType)1. && x < (FloatType)2.);` (`:210`) -- compiled out of the oracle's
    // release build, ported as `debug_assert!`, never a live `panic!` (`docs/PORTING.md`, "Error
    // sets").
    // PORT SRC: src/libslic3r/Geometry/Bicubic.hpp:210
    debug_assert!(x > 1. && x < 2.);
    // PORT SRC: src/libslic3r/Geometry/Bicubic.hpp:211
    let x = x - 1.;
    let x2 = x * x;
    let x3 = x2 * x;
    A00 + A01 * x + A02 * x2 + A03 * x3
}

#[cfg(test)]
mod tests {
    use super::*;
    use slic3r_proof::{compare_unit, Oracle};

    /// `CubicKernelWrapper<BicubicInternal::CubicBSplineKernel<float>>::kernel(float)` against
    /// the oracle, instantiated directly in the shim
    /// (`crates/slic3r-ffi/cxx/proof_units/bicubic.cpp`) the same way
    /// `crates/slic3r-ffi/cxx/proof_units/bounding_box.cpp`'s `path2_*` methods instantiate
    /// `BoundingBoxBase<Point, Points>` directly -- `docs/PORTING.md`, "Decisions recorded on
    /// 2026-11-26", decision 2: a `proof path: 1` row's oracle route is the per-unit bridge
    /// regardless of whether the symbol is externally linked or a header-only template
    /// instantiated at the call site. Every sample is cast `f32 -> f64` on both sides before
    /// comparing, an exact widening that loses no bit of the `f32` result
    /// (`docs/PORTING.md`, "Float widths": the *stored*/computed type stays `f32`; the cast is
    /// only this test's wire representation across `compare_unit`, which has no `f32` impl).
    // PROOF: 1
    #[test]
    fn bicubic_kernel_matches_the_oracle() {
        for &x in &[
            -3.0f32, -2.0, -1.5, -1.0, -0.5, -0.000001, 0.0, 0.000001, 0.5, 0.999999, 1.0,
            1.0000001, 1.5, 1.9999999, 2.0, 2.5, 3.0,
        ] {
            let actual = cubic_bspline_kernel(x) as f64;
            let expected = slic3r_proof::proof_units::bicubic::path1_cubic_bspline_kernel(x as f64);
            compare_unit("Geometry/Bicubic.hpp", Oracle::Built, actual, expected)
                .unwrap_or_else(|e| panic!("cubic_bspline_kernel({x}): {e}"));
        }
    }
}

// PORT STATUS
// source: src/libslic3r/Geometry/Bicubic.hpp
// confidence: medium
// tier: exact
// oracle: built
