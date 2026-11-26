//! The first truly differential fuzz target in the repository (task `56-foundation-point`, plan
//! step 20): `port/manifest.tsv`'s `Point.cpp`/`Point.hpp` row is the first whose `fuzz target`
//! cell reads `required`. Generates an arbitrary [`ArbPoint`] and an arbitrary rotation (a center
//! point and an angle), reuses the `ArbPoint` newtype `32-proof-dumps-fuzz`'s own committed
//! example target already defined (`crates/slic3r-ffi/fuzz/fuzz_targets/slic3r-ffi_roundtrip.rs`
//! -- not imported from there, since a `#![no_main]` fuzz binary is its own crate and cannot
//! import another `[[bin]]`'s items, but the identical shape: `x: i64, y: i64` via
//! `#[derive(Arbitrary)]`), converts each into *this unit's own ported type*,
//! `slic3r_point::point::Point`, at this target's entry point, and asserts exact equality of
//! integer-coordinate output between [`slic3r_point::point::Point::rotate`] and the same
//! `Slic3r::Point::rotate` call through `slic3r-ffi`'s `proof_units::point::path2_point_rotate`
//! shim -- the identical comparison `crates/slic3r-point/src/point.rs`'s own `// PROOF: 2` test
//! makes on fixed inputs, here driven by the fuzzer's arbitrary ones.
//!
//! **Three arms, one per run, so one corpus entry exercises the unit's two moving parts the
//! task's own text names.** The task says this target "generates arbitrary `Point` and `Points`
//! inputs"; the rotation arm below is the `Point` one, and the `nearest_point_index` arm is the
//! `Points` (`Vec<Point>`) one, driven against `Point.cpp`'s own `Points`-consuming entry point
//! through `proof_units::point::path1_nearest_point_index`. The third arm drives `Point.hpp`'s
//! own header-only `scaled<coord_t>()`/`unscaled<double>()` templates, the same two functions
//! `crates/slic3r-point/src/point.rs`'s own second `// PROOF: 2` test exercises on fixed inputs.
//!
//! `just fuzz slic3r-point_point` runs it.

#![no_main]

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use slic3r_ffi::proof_units::point::{
    path1_nearest_point_index, path2_point_rotate, path2_scaled_coord_f64, path2_unscaled_f64,
};
use slic3r_point::point::{scaled_coord, unscaled_f64, Point};

/// A single point over 64-bit integer coordinates (`coord_t` is `int64_t` at the pin,
/// `vendor/OrcaSlicer/src/libslic3r/libslic3r.h:45-48`) -- the same shape
/// `slic3r-ffi_roundtrip.rs`'s own `ArbPoint` uses, redefined here rather than imported because a
/// `#![no_main]` fuzz binary is its own crate (see the module doc).
#[derive(Debug, Clone, Copy, Arbitrary)]
struct ArbPoint {
    x: i64,
    y: i64,
}

/// The fuzz input: a point to rotate, a center to rotate it about, and an angle (the rotation
/// arm); an arbitrary list of points and a query point (the `Points`-consuming
/// `nearest_point_index` arm); and a scaling value (the `scaled`/`unscaled` arm). The angle is
/// derived from an `i32` rather than taken as a raw `f64` so the fuzzer's arbitrary bytes always
/// produce a finite, bounded value (never NaN or an astronomically large magnitude that would
/// make the two sides' floating-point rounding diverge for reasons unrelated to the rotation
/// logic under test) -- scaled to the full `[-2*PI, 2*PI]` range, wider than one full turn, so
/// the fuzzer can still reach angles past a `2*PI` wrap. The point list is capped (`arbitrary`'s
/// own length-prefixed `Vec<T>` impl, bounded here with `arbitrary_take_rest`-free `len() % 64`)
/// so one corpus entry cannot blow up the per-run cost unboundedly.
#[derive(Debug, Arbitrary)]
struct ArbInput {
    point: ArbPoint,
    center: ArbPoint,
    angle_seed: i32,
    points: Vec<ArbPoint>,
    query: ArbPoint,
    scale_seed: f64,
}

fuzz_target!(|input: ArbInput| {
    // Arm 1: `Point::rotate(angle, center)` (`Point.cpp:57-65`).
    let point = Point::new(input.point.x, input.point.y);
    let center = Point::new(input.center.x, input.center.y);
    let angle = (input.angle_seed as f64 / i32::MAX as f64) * 2.0 * std::f64::consts::PI;
    let actual = point.rotated_about(angle, center);
    let oracle = path2_point_rotate(point.x(), point.y(), center.x(), center.y(), angle);
    assert_eq!(
        (actual.x(), actual.y()),
        (oracle[0], oracle[1]),
        "Point::rotate(angle, center) must match the oracle exactly for point={:?} center={:?} \
         angle={angle}",
        input.point,
        input.center,
    );

    // Arm 2: `Point::nearest_point_index(const Points &points)` (`Point.cpp:197-204`), the
    // `Points`-consuming entry point the task's own text names.
    let points: Vec<Point> = input
        .points
        .iter()
        .take(64)
        .map(|p| Point::new(p.x, p.y))
        .collect();
    let query = Point::new(input.query.x, input.query.y);
    let xs: Vec<i64> = points.iter().map(Point::x).collect();
    let ys: Vec<i64> = points.iter().map(Point::y).collect();
    let actual_idx = query.nearest_point_index(&points);
    let oracle_idx = path1_nearest_point_index(&xs, &ys, query.x(), query.y());
    assert_eq!(
        actual_idx, oracle_idx,
        "Point::nearest_point_index(Points) must match the oracle exactly for query={:?} \
         points={:?}",
        input.query, points,
    );

    // Arm 3: `scaled<coord_t>()`/`unscaled<double>()` (`Point.hpp:669-700`), finite inputs only
    // for the same reason the angle above is derived rather than taken raw.
    if input.scale_seed.is_finite() {
        let actual_scaled = scaled_coord(input.scale_seed);
        let oracle_scaled = path2_scaled_coord_f64(input.scale_seed);
        assert_eq!(
            actual_scaled, oracle_scaled,
            "scaled::<coord_t>() must match the oracle exactly for v={}",
            input.scale_seed,
        );
    }
    let actual_unscaled = unscaled_f64(input.point.x);
    let oracle_unscaled = path2_unscaled_f64(input.point.x);
    assert_eq!(
        actual_unscaled, oracle_unscaled,
        "unscaled::<double>() must match the oracle exactly for v={}",
        input.point.x,
    );
});
