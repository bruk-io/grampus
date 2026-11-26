//! `port/manifest.tsv`'s `MultiPoint.cpp`/`MultiPoint.hpp` rows are `fuzz target: required` on
//! both cells (task `62-foundation-polygon`, plan step 20). Generates an arbitrary closed ring
//! of points and asserts exact equality of integer-coordinate output between
//! `slic3r-multipoint`'s own trait default methods (driven through a minimal, faithful
//! `MultiPoint` implementor whose `lines()` matches `Polygon`'s own closed-ring shape -- see
//! `crates/slic3r-multipoint/src/multi_point.rs`'s own test module for why this crate cannot use
//! the real `Polygon` type here) and the same calls through `slic3r-ffi`'s own
//! `proof_units::multi_point` shims -- the identical comparisons that module's `// PROOF:
//! 1`/`// PROOF: 2` tests make on fixed inputs, here driven by the fuzzer's arbitrary ones.
//!
//! `just fuzz slic3r-multipoint_multi_point` runs it.

#![no_main]

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use slic3r_ffi::proof_units::multi_point as oracle;
use slic3r_line::line::Line;
use slic3r_multipoint::multi_point::{self, MultiPoint};
use slic3r_point::point::{Point, Points};

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
    d: ArbPoint,
    scale_factor: f64,
    tx: i32,
    ty: i32,
    cos_angle: f64,
    sin_angle: f64,
    line_a: ArbPoint,
    line_b: ArbPoint,
}

/// The same `MultiPoint` implementor `crates/slic3r-multipoint/src/multi_point.rs`'s own tests
/// use: a closed ring whose `lines()` matches `Polygon`'s.
struct ClosedRing {
    points: Points,
}

impl MultiPoint for ClosedRing {
    fn points(&self) -> &Points {
        &self.points
    }
    fn points_mut(&mut self) -> &mut Points {
        &mut self.points
    }
    fn last_point(&self) -> Point {
        self.points[0]
    }
    fn lines(&self) -> slic3r_line::line::Lines {
        let pts = &self.points;
        let mut lines = slic3r_line::line::Lines::new();
        if pts.len() > 2 {
            for i in 0..pts.len() - 1 {
                lines.push(Line::new(pts[i], pts[i + 1]));
            }
            lines.push(Line::new(pts[pts.len() - 1], pts[0]));
        }
        lines
    }
}

fuzz_target!(|input: ArbInput| {
    if !input.scale_factor.is_finite()
        || !input.cos_angle.is_finite()
        || !input.sin_angle.is_finite()
    {
        return;
    }
    let pts = vec![
        Point::new(input.a.x as i64, input.a.y as i64),
        Point::new(input.b.x as i64, input.b.y as i64),
        Point::new(input.c.x as i64, input.c.y as i64),
        Point::new(input.d.x as i64, input.d.y as i64),
    ];
    let xs: Vec<i64> = pts.iter().map(|p| p.x()).collect();
    let ys: Vec<i64> = pts.iter().map(|p| p.y()).collect();

    // Arm 1: `MultiPoint::scale` (`MultiPoint.cpp:17-21`), proof path 1.
    let mut r = ClosedRing { points: pts.clone() };
    r.scale(input.scale_factor);
    let actual: Vec<i64> = r.points.iter().flat_map(|p| [p.x(), p.y()]).collect();
    let expected = oracle::path1_scale(&xs, &ys, input.scale_factor);
    assert_eq!(actual, expected, "MultiPoint::scale diverged for {:?}", input);

    // Arm 2: `MultiPoint::translate` (`:32-36`), proof path 1.
    let mut r = ClosedRing { points: pts.clone() };
    r.translate(Point::new(input.tx as i64, input.ty as i64));
    let actual: Vec<i64> = r.points.iter().flat_map(|p| [p.x(), p.y()]).collect();
    let expected = oracle::path1_translate(&xs, &ys, input.tx as i64, input.ty as i64);
    assert_eq!(actual, expected, "MultiPoint::translate diverged for {:?}", input);

    // Arm 3: `MultiPoint::rotate(cos, sin)` (`:38-46`), proof path 1.
    let mut r = ClosedRing { points: pts.clone() };
    r.rotate_cos_sin(input.cos_angle, input.sin_angle);
    let actual: Vec<i64> = r.points.iter().flat_map(|p| [p.x(), p.y()]).collect();
    let expected = oracle::path1_rotate_cos_sin(&xs, &ys, input.cos_angle, input.sin_angle);
    assert_eq!(actual, expected, "MultiPoint::rotate diverged for {:?}", input);

    // Arm 4: `MultiPoint::length()` (`:59-67`) and `MultiPoint::bounding_box()` (`:100-103`),
    // proof path 1; `MultiPoint::intersection` (`:132-139`), proof path 1.
    let r = ClosedRing { points: pts.clone() };
    assert_eq!(
        r.length(),
        oracle::path1_length(&xs, &ys),
        "MultiPoint::length diverged for {:?}",
        input
    );
    let bb = r.bounding_box();
    assert_eq!(
        vec![bb.min.x(), bb.min.y(), bb.max.x(), bb.max.y()],
        oracle::path1_bounding_box(&xs, &ys),
        "MultiPoint::bounding_box diverged for {:?}",
        input
    );
    let line_a = Point::new(input.line_a.x as i64, input.line_a.y as i64);
    let line_b = Point::new(input.line_b.x as i64, input.line_b.y as i64);
    let line = Line::new(line_a, line_b);
    let actual_intersection = r.intersection(&line);
    let actual_vec = match actual_intersection {
        Some(p) => vec![1i64, p.x(), p.y()],
        None => vec![0, 0, 0],
    };
    let oracle_intersection =
        oracle::path1_intersection(&xs, &ys, line_a.x(), line_a.y(), line_b.x(), line_b.y());
    assert_eq!(
        actual_vec, oracle_intersection,
        "MultiPoint::intersection diverged for {:?}",
        input
    );

    // Arm 5: `MultiPoint::closest_point`/`distance_to` (`MultiPoint.hpp:77-83`, inline) and the
    // free `length`/`area` (`:181-196`), proof path 2.
    let probe = Point::new(input.line_a.x as i64, input.line_a.y as i64);
    let actual_cp = r.closest_point(probe);
    let actual_cp_vec = match actual_cp {
        Some(p) => vec![1i64, p.x(), p.y()],
        None => vec![0, 0, 0],
    };
    let oracle_cp = oracle::path2_closest_point(&xs, &ys, probe.x(), probe.y());
    assert_eq!(
        actual_cp_vec, oracle_cp,
        "MultiPoint::closest_point diverged for {:?}",
        input
    );
    assert_eq!(
        r.distance_to(probe),
        oracle::path2_distance_to(&xs, &ys, probe.x(), probe.y()),
        "MultiPoint::distance_to diverged for {:?}",
        input
    );
    assert_eq!(
        multi_point::length(r.points()),
        oracle::path2_length_free(&xs, &ys),
        "the free length(Points) diverged for {:?}",
        input
    );
    assert_eq!(
        multi_point::area(r.points()),
        oracle::path2_area_free(&xs, &ys),
        "the free area(Points) diverged for {:?}",
        input
    );
});
