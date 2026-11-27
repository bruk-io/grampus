//! `port/manifest.tsv`'s `Geometry.cpp`/`Geometry.hpp` rows are `fuzz target: required` on both
//! cells (task `66-foundation-geometry`, plan step 20). Generates arbitrary `Point` inputs and
//! asserts exact equality of integer-coordinate (and boolean-predicate) output between
//! `slic3r-geometry`'s own ported functions and the same calls through `slic3r-ffi`'s
//! `proof_units::geometry` shims.
//!
//! Coordinates are generated as `i16` and widened to `i64` -- tighter than
//! `slic3r-multipoint_multi_point.rs`'s own `i32` bound. `orient`'s and `segments_intersect`'s
//! internal 64-bit cross terms multiply together *differences* of two coordinates (up to twice
//! a single coordinate's magnitude), so an `i32`-wide coordinate (as this target's first version
//! used) lets a difference-of-differences product exceed `i64::MAX` -- this target's own second
//! run found that overflow panic directly (`geometry.rs:247`, `segments_intersect`'s
//! `could_intersect`), on top of the `is_point_inside_polygon_corner`/`is_ccw` divergences its
//! first run found. `i16` keeps every product this unit's own functions compute comfortably
//! inside `i64` range (`(2^17)^2 = 2^34`, far below `2^63`); see
//! `crates/slic3r-geometry/src/geometry.rs`'s module doc, "A documented arithmetic divergence,
//! not worked around", for the magnitude this port does not add defensive wrapping for.
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
    x: i16,
    y: i16,
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
    //
    // `is_ccw`'s own vendored comment states its precondition: "The input polygon must not
    // contain duplicate points (or at least the left bottom corner point must not have
    // duplicates)", and its body carries `assert(o != ORIENTATION_COLINEAR)` (`:88`) for the
    // left-bottom corner. That `assert` is compiled out in the oracle's `-DNDEBUG` release
    // build, but this unit's own proof shim (`cxx/proof_units/geometry.cpp`) recompiles the
    // header fresh at this crate's own flags, which carry no `-DNDEBUG` either way (the same
    // point `cxx/proof_units/clipper_z_utils.cpp`'s header comment makes about every file on
    // this shared `Build`) -- so a degenerate polygon aborts the shim rather than returning an
    // answer to compare. Found by this fuzz target's own first extended run; guarded here by
    // skipping exactly the inputs the vendored precondition excludes, not by weakening the
    // comparison.
    let poly = [a, b, c, q];
    let all_distinct = poly[0] != poly[1]
        && poly[0] != poly[2]
        && poly[0] != poly[3]
        && poly[1] != poly[2]
        && poly[1] != poly[3]
        && poly[2] != poly[3];
    if all_distinct {
        let mut imin = 0usize;
        for i in 1..poly.len() {
            let pmin = poly[imin];
            let p = poly[i];
            if p.x() < pmin.x() || (p.x() == pmin.x() && p.y() < pmin.y()) {
                imin = i;
            }
        }
        let iprev = (if imin == 0 { poly.len() } else { imin }) - 1;
        let inext = if imin + 1 == poly.len() { 0 } else { imin + 1 };
        let corner_orientation = geometry::orient(poly[iprev], poly[imin], poly[inext]);
        if corner_orientation != geometry::Orientation::Colinear {
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
    }
});
