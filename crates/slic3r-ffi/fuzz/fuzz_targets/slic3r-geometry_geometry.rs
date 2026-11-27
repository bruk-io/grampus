//! `port/manifest.tsv`'s `Geometry.cpp`/`Geometry.hpp` rows are `fuzz target: required` on both
//! cells (task `66-foundation-geometry`, plan step 20). Generates arbitrary `Point` inputs and
//! asserts exact equality of integer-coordinate (and boolean-predicate) output between
//! `slic3r-geometry`'s own ported functions and the same calls through `slic3r-ffi`'s
//! `proof_units::geometry` shims.
//!
//! Coordinates are generated as `i32` and widened to `i64`, the same bound
//! `slic3r-multipoint_multi_point.rs` uses, so `orient`'s and `segments_intersect`'s internal
//! 64-bit cross products stay within `i64` range -- see `crates/slic3r-geometry/src/geometry.rs`'s
//! module doc, "A documented arithmetic divergence, not worked around".
//!
//! `just fuzz slic3r-geometry_geometry` runs it.

#![no_main]

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use slic3r_ffi::proof_units::geometry as oracle;
use slic3r_geometry::geometry;
use slic3r_point::point::Point;

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
    q: ArbPoint,
    jp1: ArbPoint,
    jp2: ArbPoint,
}

fuzz_target!(|input: ArbInput| {
    let a = Point::new(input.a.x as i64, input.a.y as i64);
    let b = Point::new(input.b.x as i64, input.b.y as i64);
    let c = Point::new(input.c.x as i64, input.c.y as i64);
    let q = Point::new(input.q.x as i64, input.q.y as i64);
    let jp1 = Point::new(input.jp1.x as i64, input.jp1.y as i64);
    let jp2 = Point::new(input.jp2.x as i64, input.jp2.y as i64);

    // `Geometry::orient` (`Geometry.hpp:44-61`).
    let actual_orient = geometry::orient(a, b, c) as i32;
    let oracle_orient = oracle::path_orient(a.x(), a.y(), b.x(), b.y(), c.x(), c.y());
    assert_eq!(
        actual_orient, oracle_orient,
        "Geometry::orient diverged for {:?}",
        input
    );

    // `Geometry::segments_intersect` (`:134-184`).
    let actual_seg = geometry::segments_intersect(a, b, jp1, jp2);
    let oracle_seg = oracle::path_segments_intersect(
        a.x(),
        a.y(),
        b.x(),
        b.y(),
        jp1.x(),
        jp1.y(),
        jp2.x(),
        jp2.y(),
    );
    assert_eq!(
        actual_seg, oracle_seg,
        "Geometry::segments_intersect diverged for {:?}",
        input
    );

    // `Geometry::is_point_inside_polygon_corner` (`Geometry.cpp:864-908`). Skipped when `a == b`
    // or `c == b`, the vendored precondition the un-ported `assert(ba != bc)` states (a
    // zero-length `ba`/`bc` normalizes to NaN on both sides alike, which is not an interesting
    // divergence to report).
    if a != b && c != b {
        let actual_corner = geometry::is_point_inside_polygon_corner(a, b, c, q);
        let oracle_corner = oracle::path_is_point_inside_polygon_corner(
            a.x(),
            a.y(),
            b.x(),
            b.y(),
            c.x(),
            c.y(),
            q.x(),
            q.y(),
        );
        assert_eq!(
            actual_corner, oracle_corner,
            "Geometry::is_point_inside_polygon_corner diverged for {:?}",
            input
        );
    }

    // `Geometry::is_ccw` (`Geometry.hpp:66-90`), over the same four points as a ring.
    if a != b && b != c && a != c {
        let poly = [a, b, c, q];
        let xs: Vec<i64> = poly.iter().map(|p| p.x()).collect();
        let ys: Vec<i64> = poly.iter().map(|p| p.y()).collect();
        let actual_ccw = geometry::is_ccw(&poly);
        let oracle_ccw = oracle::path_is_ccw(&xs, &ys);
        assert_eq!(
            actual_ccw, oracle_ccw,
            "Geometry::is_ccw diverged for {:?}",
            input
        );
    }
});
