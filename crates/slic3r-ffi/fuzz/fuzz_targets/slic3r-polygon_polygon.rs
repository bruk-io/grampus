//! `port/manifest.tsv`'s `Polygon.cpp`/`Polygon.hpp` row is `fuzz target: required` on both
//! cells (task `62-foundation-polygon`, plan step 20). Generates an arbitrary polygon and probe
//! point and asserts exact equality of integer-coordinate output between `slic3r-polygon`'s own
//! module and the same calls through `slic3r-ffi`'s `proof_units::polygon` shims -- the
//! identical comparisons that module's `// PROOF: 3` test makes on fixed inputs, here driven by
//! the fuzzer's arbitrary ones.
//!
//! `just fuzz slic3r-polygon_polygon` runs it.

#![no_main]

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use slic3r_ffi::proof_units::polygon as oracle;
use slic3r_point::point::Point;
use slic3r_polygon::polygon::{self, Polygon};

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
    d: ArbPoint,
    e: ArbPoint,
    probe: ArbPoint,
}

fuzz_target!(|input: ArbInput| {
    let pts: Vec<Point> = [input.a, input.b, input.c, input.d, input.e]
        .iter()
        .map(|p| Point::new(p.x as i64, p.y as i64))
        .collect();
    let poly = Polygon::from_points(pts.clone());
    let xs: Vec<i64> = pts.iter().map(|p| p.x()).collect();
    let ys: Vec<i64> = pts.iter().map(|p| p.y()).collect();
    let probe_x = input.probe.x as i64;
    let probe_y = input.probe.y as i64;

    assert_eq!(poly.area(), oracle::path3_area(&xs, &ys), "Polygon::area diverged for {:?}", input);
    assert_eq!(
        poly.is_counter_clockwise(),
        oracle::path3_is_counter_clockwise(&xs, &ys),
        "Polygon::is_counter_clockwise diverged for {:?}",
        input
    );

    let c = poly.centroid();
    let oracle_c = oracle::path3_centroid(&xs, &ys);
    // `centroid()` divides by the (possibly zero, for a degenerate input) area sum; skip the
    // comparison on a degenerate polygon rather than asserting a NaN/Inf shape neither side
    // defines meaningfully.
    if poly.area() != 0.0 {
        assert_eq!(vec![c.x(), c.y()], oracle_c, "Polygon::centroid diverged for {:?}", input);
    }

    assert_eq!(
        poly.contains(Point::new(probe_x, probe_y)),
        oracle::path3_contains(&xs, &ys, probe_x, probe_y),
        "Polygon::contains diverged for {:?}",
        input
    );

    let proj = poly.point_projection(Point::new(probe_x, probe_y));
    let oracle_proj = oracle::path3_point_projection(&xs, &ys, probe_x, probe_y);
    assert_eq!(
        vec![proj.x(), proj.y()],
        oracle_proj,
        "Polygon::point_projection diverged for {:?}",
        input
    );

    assert_eq!(
        polygon::polygon_is_convex(&poly.points),
        oracle::path3_polygon_is_convex(&xs, &ys),
        "polygon_is_convex diverged for {:?}",
        input
    );

    let actual_param = poly.parameter_by_length();
    let oracle_param = oracle::path3_parameter_by_length(&xs, &ys);
    assert_eq!(
        actual_param, oracle_param,
        "Polygon::parameter_by_length diverged for {:?}",
        input
    );
});
