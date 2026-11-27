//! `port/manifest.tsv`'s `Polyline.cpp`/`Polyline.hpp` rows are `fuzz target: required` on both
//! cells (task `63-foundation-polyline`, plan step 20). Generates an arbitrary open point chain
//! and asserts exact equality of output between `slic3r-polyline`'s own methods and the same
//! calls through `slic3r-ffi`'s `proof_units::polyline` shims -- the identical comparisons that
//! module's `// PROOF: 1`/`// PROOF: 2` tests make on fixed inputs, here driven by the fuzzer's
//! arbitrary ones.
//!
//! `just fuzz slic3r-polyline_polyline` runs it.

#![no_main]

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use slic3r_ffi::proof_units::polyline as oracle;
use slic3r_point::point::{Point, Vec2d};
use slic3r_polyline::polyline::{self, Polyline, Polylines};

/// A single point over 64-bit integer coordinates, the same bounded shape
/// `slic3r-point_point.rs`'s own `ArbPoint` uses (redefined here, since a `#![no_main]` fuzz
/// binary is its own crate).
#[derive(Debug, Clone, Copy, Arbitrary)]
struct ArbPoint {
    x: i32,
    y: i32,
}

#[derive(Debug, Arbitrary)]
struct ArbInput {
    a: ArbPoint,
    b: ArbPoint,
    c: ArbPoint,
    distance: f64,
    scale_x: f64,
    scale_y: f64,
}

fuzz_target!(|input: ArbInput| {
    if !input.distance.is_finite() || !input.scale_x.is_finite() || !input.scale_y.is_finite() {
        return;
    }
    let pts = vec![
        Point::new(input.a.x as i64, input.a.y as i64),
        Point::new(input.b.x as i64, input.b.y as i64),
        Point::new(input.c.x as i64, input.c.y as i64),
    ];
    let xs: Vec<i64> = pts.iter().map(|p| p.x()).collect();
    let ys: Vec<i64> = pts.iter().map(|p| p.y()).collect();

    // Arm 1: `Polyline::clip_end` (`Polyline.cpp:61-101`), proof path 1. A non-positive distance
    // is a no-op loop guard (`while (distance > 0)`) on both sides, so every finite `distance`
    // is a legitimate input.
    let mut pl = Polyline::from_points(pts.clone());
    pl.clip_end(input.distance);
    let actual: Vec<i64> = pl.points.iter().flat_map(|p| [p.x(), p.y()]).collect();
    let expected = oracle::path1_clip_end(&xs, &ys, input.distance);
    assert_eq!(actual, expected, "Polyline::clip_end diverged for {:?}", input);

    // Arm 2: the free `total_length(Polylines)` (`Polyline.hpp:168-173`) and
    // `Polyline::is_closed` (`:132`), proof path 2.
    let pl2 = Polyline::from_points(pts.clone());
    let mut pls = Polylines::new();
    pls.push(pl2.clone());
    assert_eq!(
        polyline::total_length(&pls),
        oracle::path2_total_length(&xs, &ys),
        "total_length(Polylines) diverged for {:?}",
        input
    );
    assert_eq!(
        pl2.is_closed(),
        oracle::path2_is_closed(&xs, &ys),
        "Polyline::is_closed diverged for {:?}",
        input
    );

    // Arm 3: `Polyline::new_scale` (`Polyline.hpp:55-63`), proof path 2.
    let scale_pts = vec![Vec2d::new(input.scale_x, input.scale_y)];
    let sxs: Vec<f64> = scale_pts.iter().map(|p| p.x()).collect();
    let sys: Vec<f64> = scale_pts.iter().map(|p| p.y()).collect();
    let actual_scale: Vec<i64> = Polyline::new_scale(&scale_pts)
        .points
        .iter()
        .flat_map(|p| [p.x(), p.y()])
        .collect();
    let expected_scale = oracle::path2_new_scale(&sxs, &sys);
    assert_eq!(
        actual_scale, expected_scale,
        "Polyline::new_scale diverged for {:?}",
        input
    );
});
