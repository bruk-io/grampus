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
//! `just fuzz slic3r-point_point` runs it.

#![no_main]

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use slic3r_ffi::proof_units::point::path2_point_rotate;
use slic3r_point::point::Point;

/// A single point over 64-bit integer coordinates (`coord_t` is `int64_t` at the pin,
/// `vendor/OrcaSlicer/src/libslic3r/libslic3r.h:45-48`) -- the same shape
/// `slic3r-ffi_roundtrip.rs`'s own `ArbPoint` uses, redefined here rather than imported because a
/// `#![no_main]` fuzz binary is its own crate (see the module doc).
#[derive(Debug, Clone, Copy, Arbitrary)]
struct ArbPoint {
    x: i64,
    y: i64,
}

/// The fuzz input: a point to rotate, a center to rotate it about, and an angle. The angle is
/// derived from an `i32` rather than taken as a raw `f64` so the fuzzer's arbitrary bytes always
/// produce a finite, bounded value (never NaN or an astronomically large magnitude that would
/// make the two sides' floating-point rounding diverge for reasons unrelated to the rotation
/// logic under test) -- scaled to the full `[-2*PI, 2*PI]` range, wider than one full turn, so
/// the fuzzer can still reach angles past a `2*PI` wrap.
#[derive(Debug, Arbitrary)]
struct ArbRotation {
    point: ArbPoint,
    center: ArbPoint,
    angle_seed: i32,
}

fuzz_target!(|input: ArbRotation| {
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
});
