//! Differential fuzz target for `Arachne/utils/PolygonsPointIndex.hpp` (task
//! `73-foundation-arachne-indices`, plan step 20): `port/manifest.tsv`'s
//! `Arachne/utils/PolygonsPointIndex.hpp` row is `fuzz target: required`. Generates an
//! arbitrary one-polygon `Polygons` and a `point_idx` within it, and asserts exact equality
//! between `slic3r_arachne_utils::polygons_point_index::PathsPointIndex::{p, next, prev}` and
//! the same calls through `slic3r-ffi`'s own `proof_units::polygons_point_index` shim -- the
//! identical comparison `crates/slic3r-arachne-utils/src/polygons_point_index.rs`'s own
//! `// PROOF: 1` test makes on fixed inputs, here driven by the fuzzer's arbitrary ones.
//!
//! `just fuzz slic3r-arachne-utils_polygons_point_index` runs it.

#![no_main]

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use slic3r_arachne_utils::polygons_point_index::PathsPointIndex;
use slic3r_ffi::proof_units::polygons_point_index::path1_p_next_prev;
use slic3r_point::point::Point;
use slic3r_polygon::polygon::{Polygon, Polygons};

/// A single point over 64-bit integer coordinates, the same shape
/// `slic3r-arachne-utils_linear_alg2_d.rs`'s own `ArbPoint` uses.
#[derive(Debug, Clone, Copy, Arbitrary)]
struct ArbPoint {
    x: i64,
    y: i64,
}

/// `points` is kept non-empty by construction (`first` plus zero or more `rest`), so
/// `point_idx_raw % points.len()` below never divides by zero -- an empty `Vec<ArbPoint>` would
/// make `(*polygons)[poly_idx]`'s own `size()` zero, which the vendored `operator++`/`--`'s
/// modulo-by-size would divide by (UB in C++, a panic in Rust): a false finding this shape
/// avoids rather than one this unit's own translation needs to guard against.
#[derive(Debug, Arbitrary)]
struct ArbInput {
    first: ArbPoint,
    rest: Vec<ArbPoint>,
    point_idx_raw: u32,
}

fuzz_target!(|input: ArbInput| {
    let mut points: Vec<Point> = Vec::with_capacity(1 + input.rest.len());
    points.push(Point::new(input.first.x, input.first.y));
    for p in &input.rest {
        points.push(Point::new(p.x, p.y));
    }
    let xs: Vec<i64> = points.iter().map(|p| p.x()).collect();
    let ys: Vec<i64> = points.iter().map(|p| p.y()).collect();

    let polys: Polygons = vec![Polygon::from_points(points)];
    let point_idx = (input.point_idx_raw as usize) % xs.len();

    let idx = PathsPointIndex::new(&polys, 0, point_idx);
    let actual_p = idx.p();
    let actual_next = idx.next().p();
    let actual_prev = idx.prev().p();

    let oracle = path1_p_next_prev(&xs, &ys, point_idx as u32);
    assert_eq!(
        (
            actual_p.x(),
            actual_p.y(),
            actual_next.x(),
            actual_next.y(),
            actual_prev.x(),
            actual_prev.y(),
        ),
        (oracle[0], oracle[1], oracle[2], oracle[3], oracle[4], oracle[5]),
        "PolygonsPointIndex::p/next/prev must match the oracle exactly for xs={:?} ys={:?} point_idx={}",
        xs,
        ys,
        point_idx,
    );
});
