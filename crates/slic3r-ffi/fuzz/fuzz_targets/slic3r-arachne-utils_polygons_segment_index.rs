//! Differential fuzz target for `Arachne/utils/PolygonsSegmentIndex.hpp` (task
//! `73-foundation-arachne-indices`, plan step 20): `port/manifest.tsv`'s
//! `Arachne/utils/PolygonsSegmentIndex.hpp` row is `fuzz target: required`. Generates an
//! arbitrary one-polygon `Polygons` and a `point_idx` within it, and asserts exact equality
//! between `slic3r_arachne_utils::polygons_segment_index::PolygonsSegmentIndex::{from, to}` and
//! the same calls through `slic3r-ffi`'s own `proof_units::polygons_segment_index` shim -- the
//! identical comparison `crates/slic3r-arachne-utils/src/polygons_segment_index.rs`'s own
//! `// PROOF: 1` test makes on fixed inputs, here driven by the fuzzer's arbitrary ones.
//!
//! `just fuzz slic3r-arachne-utils_polygons_segment_index` runs it.

#![no_main]

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use slic3r_arachne_utils::polygons_segment_index::PolygonsSegmentIndex;
use slic3r_ffi::proof_units::polygons_segment_index::path1_from_to;
use slic3r_point::point::Point;
use slic3r_polygon::polygon::{Polygon, Polygons};

/// A single point over 64-bit integer coordinates, the same shape
/// `slic3r-arachne-utils_linear_alg2_d.rs`'s own `ArbPoint` uses.
#[derive(Debug, Clone, Copy, Arbitrary)]
struct ArbPoint {
    x: i64,
    y: i64,
}

/// `points` is kept non-empty by construction -- see
/// `slic3r-arachne-utils_polygons_point_index.rs`'s identical `ArbInput` doc for why.
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

    let seg = PolygonsSegmentIndex::new(&polys, 0, point_idx);
    let actual_from = seg.from();
    let actual_to = seg.to();

    let oracle = path1_from_to(&xs, &ys, point_idx as u32);
    assert_eq!(
        (actual_from.x(), actual_from.y(), actual_to.x(), actual_to.y()),
        (oracle[0], oracle[1], oracle[2], oracle[3]),
        "PolygonsSegmentIndex::from/to must match the oracle exactly for xs={:?} ys={:?} point_idx={}",
        xs,
        ys,
        point_idx,
    );
});
