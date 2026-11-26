//! Differential fuzz target for `Arachne/utils/linearAlg2D.hpp`'s `pointIsLeftOfLine` and
//! `getAngleLeft` (task `72-foundation-arachne-grids`, plan step 20): `port/manifest.tsv`'s
//! `Arachne/utils/linearAlg2D.hpp` row is `fuzz target: required`. Generates four arbitrary
//! points and asserts exact equality between
//! `slic3r_arachne_utils::linear_alg2_d::{point_is_left_of_line, get_angle_left}` and the same
//! calls through `slic3r-ffi`'s `proof_units::linear_alg2_d` shim -- the identical comparison
//! `crates/slic3r-arachne-utils/src/linear_alg2_d.rs`'s own `// PROOF: 1` test makes on fixed
//! inputs, here driven by the fuzzer's arbitrary ones.
//!
//! `just fuzz slic3r-arachne-utils_linear_alg2_d` runs it.

#![no_main]

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use slic3r_arachne_utils::linear_alg2_d::{get_angle_left, point_is_left_of_line};
use slic3r_ffi::proof_units::linear_alg2_d::{path1_get_angle_left, path1_point_is_left_of_line};
use slic3r_point::point::Point;

/// A single point over 64-bit integer coordinates (`coord_t` is `int64_t` at the pin,
/// `vendor/OrcaSlicer/src/libslic3r/libslic3r.h:45-48`) -- the same shape
/// `slic3r-ffi_roundtrip.rs`'s own `ArbPoint` uses, redefined here rather than imported because
/// a `#![no_main]` fuzz binary is its own crate (`slic3r-point_point.rs`'s own module doc
/// states the identical reason).
#[derive(Debug, Clone, Copy, Arbitrary)]
struct ArbPoint {
    x: i64,
    y: i64,
}

/// The fuzz input: `p`/`a`/`b` for the `pointIsLeftOfLine` arm, `a`/`b`/`c` for the
/// `getAngleLeft` arm -- `a` and `b` are shared between the two arms, driving both from one
/// corpus entry.
#[derive(Debug, Arbitrary)]
struct ArbInput {
    p: ArbPoint,
    a: ArbPoint,
    b: ArbPoint,
    c: ArbPoint,
}

fuzz_target!(|input: ArbInput| {
    let p = Point::new(input.p.x, input.p.y);
    let a = Point::new(input.a.x, input.a.y);
    let b = Point::new(input.b.x, input.b.y);
    let c = Point::new(input.c.x, input.c.y);

    // Arm 1: `pointIsLeftOfLine(p, a, b)` (`linearAlg2D.hpp:27-30`).
    let actual_left = point_is_left_of_line(p, a, b);
    let oracle_left = path1_point_is_left_of_line(p.x(), p.y(), a.x(), a.y(), b.x(), b.y());
    assert_eq!(
        actual_left, oracle_left,
        "pointIsLeftOfLine must match the oracle exactly for p={:?} a={:?} b={:?}",
        input.p, input.a, input.b,
    );

    // Arm 2: `getAngleLeft(a, b, c)` (`linearAlg2D.hpp:46-70`).
    let actual_angle = get_angle_left(a, b, c);
    let oracle_angle = path1_get_angle_left(a.x(), a.y(), b.x(), b.y(), c.x(), c.y());
    assert_eq!(
        actual_angle, oracle_angle,
        "getAngleLeft must match the oracle exactly for a={:?} b={:?} c={:?}",
        input.a, input.b, input.c,
    );
});
