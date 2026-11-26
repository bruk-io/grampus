//! `port/manifest.tsv`'s `Line.cpp`/`Line.hpp` row is `fuzz target: required` on both cells
//! (task `61-foundation-line-slicing`, plan step 20). Generates arbitrary `Point`/`Line` inputs
//! and asserts exact equality of integer-coordinate output between `slic3r-line`'s own module
//! and the same calls through `slic3r-ffi`'s `proof_units::line` shims -- the identical
//! comparisons `crates/slic3r-line/src/line.rs`'s own `// PROOF: 1`/`// PROOF: 2` tests make on
//! fixed inputs, here driven by the fuzzer's arbitrary ones.
//!
//! Two arms, one per run: `intersection_infinite` (`Line.cpp`, proof path 1) and
//! `length()`/`midpoint()`/`normal()` (`Line.hpp`'s own header-only surface, proof path 2).
//!
//! `just fuzz slic3r-line_line` runs it.

#![no_main]

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use slic3r_ffi::proof_units::line::{
    path1_intersection_infinite, path2_line_length, path2_line_midpoint_normal,
};
use slic3r_line::line::Line;
use slic3r_point::point::Point;

/// A single point over 64-bit integer coordinates (`coord_t` is `int64_t` at the pin) -- the
/// same shape `slic3r-point_point.rs`'s own `ArbPoint` uses, redefined here rather than imported
/// because a `#![no_main]` fuzz binary is its own crate (see that target's module doc).
#[derive(Debug, Clone, Copy, Arbitrary)]
struct ArbPoint {
    x: i64,
    y: i64,
}

/// The fuzz input: two arbitrary lines (`l1`'s endpoints feed both arms; `l2` is `l1`'s partner
/// for `intersection_infinite`).
#[derive(Debug, Arbitrary)]
struct ArbInput {
    l1a: ArbPoint,
    l1b: ArbPoint,
    l2a: ArbPoint,
    l2b: ArbPoint,
}

fuzz_target!(|input: ArbInput| {
    let l1 = Line::new(
        Point::new(input.l1a.x, input.l1a.y),
        Point::new(input.l1b.x, input.l1b.y),
    );
    let l2 = Line::new(
        Point::new(input.l2a.x, input.l2a.y),
        Point::new(input.l2b.x, input.l2b.y),
    );

    // Arm 1: `Line::intersection_infinite` (`Line.cpp:27-46`), proof path 1.
    let actual = l1.intersection_infinite(&l2);
    let oracle = path1_intersection_infinite(
        l1.a.x(),
        l1.a.y(),
        l1.b.x(),
        l1.b.y(),
        l2.a.x(),
        l2.a.y(),
        l2.b.x(),
        l2.b.y(),
    );
    let actual_vec = match actual {
        Some(p) => vec![1i64, p.x(), p.y()],
        None => vec![0i64, 0, 0],
    };
    assert_eq!(
        actual_vec, oracle,
        "Line::intersection_infinite must match the oracle exactly for l1={:?} l2={:?}",
        l1, l2,
    );

    // Arm 2: `length()`/`midpoint()`/`normal()` (`Line.hpp:166-167,183`), proof path 2.
    let actual_length = l1.length();
    let oracle_length = path2_line_length(l1.a.x(), l1.a.y(), l1.b.x(), l1.b.y());
    assert_eq!(
        actual_length, oracle_length,
        "Line::length() must match the oracle exactly for l1={:?}",
        l1,
    );
    let mid = l1.midpoint();
    let normal = l1.normal();
    let oracle_mid_normal = path2_line_midpoint_normal(l1.a.x(), l1.a.y(), l1.b.x(), l1.b.y());
    assert_eq!(
        vec![mid.x(), mid.y(), normal.x(), normal.y()],
        oracle_mid_normal,
        "Line::midpoint()/normal() must match the oracle exactly for l1={:?}",
        l1,
    );
});
