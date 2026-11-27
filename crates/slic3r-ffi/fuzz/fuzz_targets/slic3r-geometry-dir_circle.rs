//! `port/manifest.tsv`'s `Geometry/Circle.cpp`/`Circle.hpp` rows are both `fuzz target: required`
//! (task `76-foundation-voronoi-curves`, plan step 20). Generates arbitrary `Point` inputs --
//! this unit's own surface takes plain points, never a `Polygon`/`ExPolygon`, so the generic
//! "Polygon and ExPolygon inputs" phrasing `docs/WORKFLOW.md`'s "A `required` fuzz target" rule
//! uses does not apply to it -- and asserts exact equality between
//! `slic3r_geometry_dir::circle`'s own functions and the same calls through `slic3r-ffi`'s
//! `proof_units::circle` shims, the identical comparison
//! `crates/slic3r-geometry-dir/src/circle.rs`'s own `// PROOF: 1`/`// PROOF: 2` tests make on
//! fixed inputs, here driven by the fuzzer's arbitrary ones.
//!
//! Three arms: `circle_center` (three arbitrary points, `Circle.hpp`'s header-only row),
//! `smallest_enclosing_circle_welzl` (an arbitrary, bounded-length point set, the same row), and
//! `circle_center_taubin_newton` (`Circle.cpp`'s own externally-linked row).
//!
//! **Every coordinate is derived from an `i32` seed**, the same technique
//! `slic3r-boundingbox_bounding_box.rs`'s own fuzz target uses, so no generated `Point` ever
//! reaches a magnitude where the port's own arithmetic (plain `f64`, no scaling conversion in
//! this unit) could diverge from the oracle only because of a build-configuration difference in
//! overflow behaviour rather than the ported logic itself.

#![no_main]

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use slic3r_ffi::proof_units::circle as oracle;
use slic3r_geometry_dir::circle;
use slic3r_point::point::Point;

#[derive(Debug, Clone, Copy, Arbitrary)]
struct ArbPoint {
    x: i32,
    y: i32,
}

impl ArbPoint {
    fn to_point(self) -> Point {
        Point::new(self.x as i64, self.y as i64)
    }
}

#[derive(Debug, Arbitrary)]
struct ArbInput {
    a: ArbPoint,
    b: ArbPoint,
    c: ArbPoint,
    /// Bounded to a handful of points: `smallest_enclosing_circle2_welzl` is cubic in the worst
    /// case, and this fuzz target runs under `cargo fuzz`'s own time budget, not a correctness
    /// bound.
    extra: Vec<ArbPoint>,
}

fuzz_target!(|input: ArbInput| {
    let a_pt = input.a.to_point();
    let b_pt = input.b.to_point();
    let c_pt = input.c.to_point();

    // Arm 1: `circle_center<Vec2d>` (`Circle.hpp:17-36`), header-only row.
    let a = slic3r_point::point::unscale2(a_pt.x(), a_pt.y());
    let b = slic3r_point::point::unscale2(b_pt.x(), b_pt.y());
    let c = slic3r_point::point::unscale2(c_pt.x(), c_pt.y());
    let epsilon = 1e-6;
    let actual = circle::circle_center(a, b, c, epsilon);
    let expected = oracle::path2_circle_center(a.x(), a.y(), b.x(), b.y(), c.x(), c.y(), epsilon);
    assert_eq!(
        (actual.x(), actual.y()),
        (expected[0], expected[1]),
        "circle_center must match the oracle exactly for a={a:?} b={b:?} c={c:?}",
    );

    // Arm 2: `smallest_enclosing_circle_welzl(const Points&)` (`Circle.hpp:151-154`), the same
    // header-only row, over a bounded point set (at most 8, to keep the cubic worst case cheap).
    let mut points: Vec<Point> = vec![a_pt, b_pt, c_pt];
    points.extend(input.extra.iter().take(5).map(|p| p.to_point()));
    let actual = circle::smallest_enclosing_circle_welzl(&points);
    let flat: Vec<i64> = points.iter().flat_map(|p| [p.x(), p.y()]).collect();
    let expected = oracle::path2_smallest_enclosing_circle_welzl(flat);
    assert_eq!(
        (actual.center.x(), actual.center.y(), actual.radius),
        (expected[0], expected[1], expected[2]),
        "smallest_enclosing_circle_welzl must match the oracle exactly for points={points:?}",
    );

    // Arm 3: `circle_center_taubin_newton(const Vec2ds&, size_t)` (`Circle.cpp`'s own row,
    // externally linked), over the same unscaled point set as arm 1/2 (at least 3 points, as
    // the Taubin-Newton fit expects -- an arbitrary `extra` list may still leave fewer than
    // three total if duplicated, so this arm always uses the fixed `a`/`b`/`c` triple plus the
    // same bounded `extra` set).
    let input_vec2d: Vec<_> = points
        .iter()
        .map(|p| slic3r_point::point::unscale2(p.x(), p.y()))
        .collect();
    let flat_f64: Vec<f64> = input_vec2d.iter().flat_map(|p| [p.x(), p.y()]).collect();
    let actual = circle::circle_center_taubin_newton_vec2d(&input_vec2d, 20);
    let expected = oracle::path1_circle_center_taubin_newton(flat_f64, 20);
    // `circle_center_taubin_newton` can return `NaN` on a divergent fit (`Circle.cpp:81-84`);
    // `NaN != NaN`, so that case is compared by its bit pattern instead of by `==`, matching
    // `ProofValue`'s own `f64`/`Vec<f64>` impl in `crates/slic3r-proof/src/tiers.rs`, which this
    // fuzz target does not depend on and so re-derives inline.
    assert_eq!(
        actual.x().to_bits(),
        expected[0].to_bits(),
        "circle_center_taubin_newton.x must match the oracle exactly (bit pattern, NaN included) \
         for points={points:?}",
    );
    assert_eq!(
        actual.y().to_bits(),
        expected[1].to_bits(),
        "circle_center_taubin_newton.y must match the oracle exactly (bit pattern, NaN included) \
         for points={points:?}",
    );
});
