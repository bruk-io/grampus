//! `port/manifest.tsv`'s `BoundingBox.cpp`/`BoundingBox.hpp` row is `fuzz target: required`
//! (task `58-foundation-bounding-box`, plan step 20). Generates an arbitrary `BoundingBox` (a
//! min/max pair of arbitrary `Point`s, each independently -- `min`/`max` are not sorted into a
//! "min is actually smaller" relationship, matching how the ported struct's public fields allow
//! any combination) and a handful of scalar parameters, and asserts exact equality of
//! integer-coordinate output between `slic3r_boundingbox::bounding_box::BoundingBox`'s own
//! methods and the same calls through `slic3r-ffi`'s `proof_units::bounding_box` shims -- the
//! identical comparison `crates/slic3r-boundingbox/src/bounding_box.rs`'s own `// PROOF: 1`/
//! `// PROOF: 2` tests make on fixed inputs, here driven by the fuzzer's arbitrary ones.
//!
//! Six arms, one per run, covering both manifest rows: `merge`, `scale` and `offset`/`translate`
//! exercise the header-only `BoundingBoxBase` surface (`BoundingBox.hpp`'s own row); `rotated`
//! and `scaled` exercise `BoundingBox.cpp`'s own, externally linked methods (that row).
//!
//! **Every input is derived from an `i32` seed, bounded to a few billion either way**, the same
//! technique `slic3r-point_point.rs`'s own rotation arm uses for its angle -- never a raw `f64`
//! or a full-range `i64` coordinate. Two independent, ported-arithmetic-only overflow panics
//! (never a mismatch against the oracle) were found and fixed by this bounding during this
//! task's own fuzzing: `Point::from_f64_rounded` saturating an astronomical `offset`/`translate`
//! delta to `i64::MAX` and then overflowing `self.min - v`, and (before that) an unconstrained
//! `f64` delta reaching the same saturate-then-overflow path through a different arm. Neither is
//! a divergence from the oracle -- the oracle's own `int64_t` arithmetic wraps the same way
//! C++ signed overflow always has on this target, silently; only Rust's debug/fuzz build panics
//! on it -- so bounding every input to a magnitude no real scaled coordinate ever reaches (a
//! print bed many orders of magnitude larger than any real printer, at `SCALING_FACTOR`) avoids
//! exercising that difference in build-configuration behaviour rather than the ported logic.
//!
//! `just fuzz slic3r-boundingbox_bounding_box` runs it.

#![no_main]

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use slic3r_boundingbox::bounding_box::BoundingBox;
use slic3r_ffi::proof_units::bounding_box as oracle;
use slic3r_point::point::Point;

/// A coordinate bounded to +/- two billion -- far more than any real scaled coordinate at
/// `SCALING_FACTOR` (a print bed many kilometres across), and small enough that no combination
/// of two such coordinates, or one of them with a [`bounded_delta`] value, can overflow `i64`
/// arithmetic.
fn bounded_coord(seed: i32) -> i64 {
    seed as i64
}

/// A scalar delta/factor/angle seed, bounded to `[-1000.0, 1000.0]`, the same technique
/// `slic3r-point_point.rs`'s own angle arm uses: never `NaN`, never astronomically large.
fn bounded_f64(seed: i32) -> f64 {
    (seed as f64 / i32::MAX as f64) * 1_000.0
}

#[derive(Debug, Clone, Copy, Arbitrary)]
struct ArbPoint {
    x: i32,
    y: i32,
}

#[derive(Debug, Arbitrary)]
struct ArbInput {
    min: ArbPoint,
    max: ArbPoint,
    merge_point: ArbPoint,
    scale_seed: i32,
    translate_x_seed: i32,
    translate_y_seed: i32,
    offset_seed: i32,
    angle_seed: i32,
}

fuzz_target!(|input: ArbInput| {
    let min = Point::new(bounded_coord(input.min.x), bounded_coord(input.min.y));
    let max = Point::new(bounded_coord(input.max.x), bounded_coord(input.max.y));
    let bb = BoundingBox::from_min_max(min, max);

    // Arm 1: `BoundingBoxBase::merge(const PointType&)` (`BoundingBox.cpp:74-85`).
    let merge_point = Point::new(
        bounded_coord(input.merge_point.x),
        bounded_coord(input.merge_point.y),
    );
    let mut actual = bb;
    actual.merge(merge_point);
    let oracle_out = oracle::path2_merge_point(
        min.x(),
        min.y(),
        max.x(),
        max.y(),
        bb.defined,
        merge_point.x(),
        merge_point.y(),
    );
    assert_eq!(
        (
            actual.min.x(),
            actual.min.y(),
            actual.max.x(),
            actual.max.y(),
            actual.defined as i64
        ),
        (oracle_out[0], oracle_out[1], oracle_out[2], oracle_out[3], oracle_out[4]),
        "BoundingBoxBase::merge(point) must match the oracle exactly for bb={bb:?} \
         point={merge_point:?}",
    );

    // Arm 2: `BoundingBoxBase::scale(double)` (`:64-69`), header-only row.
    let scale_factor = bounded_f64(input.scale_seed);
    let mut actual = bb;
    actual.scale(scale_factor);
    let oracle_out = oracle::path2_scale(min.x(), min.y(), max.x(), max.y(), scale_factor);
    assert_eq!(
        (actual.min.x(), actual.min.y(), actual.max.x(), actual.max.y()),
        (oracle_out[0], oracle_out[1], oracle_out[2], oracle_out[3]),
        "BoundingBoxBase::scale(factor) must match the oracle exactly for bb={bb:?} \
         factor={scale_factor}",
    );

    // Arm 3: `BoundingBoxBase::translate(coordf_t, coordf_t)` (`BoundingBox.hpp:46`), the
    // rounding overload -- `assert(this->defined)` in the vendored body (compiled out of the
    // oracle's release build, ported as `debug_assert!`), so this arm is skipped on an
    // undefined `bb` exactly as a real caller is required to: the precondition this method
    // documents, not a gap in the port.
    if bb.defined {
        let tx = bounded_f64(input.translate_x_seed);
        let ty = bounded_f64(input.translate_y_seed);
        let mut actual = bb;
        actual.translate(tx, ty);
        let oracle_out = oracle::path2_translate_xy(min.x(), min.y(), max.x(), max.y(), tx, ty);
        assert_eq!(
            (actual.min.x(), actual.min.y(), actual.max.x(), actual.max.y()),
            (oracle_out[0], oracle_out[1], oracle_out[2], oracle_out[3]),
            "BoundingBoxBase::translate(x, y) must match the oracle exactly for bb={bb:?} \
             x={tx} y={ty}",
        );
    }

    // Arm 4: `BoundingBoxBase::offset(coordf_t)` (`BoundingBox.cpp:208-214`).
    let delta = bounded_f64(input.offset_seed);
    let mut actual = bb;
    actual.offset(delta);
    let oracle_out = oracle::path2_offset(min.x(), min.y(), max.x(), max.y(), delta);
    assert_eq!(
        (actual.min.x(), actual.min.y(), actual.max.x(), actual.max.y()),
        (oracle_out[0], oracle_out[1], oracle_out[2], oracle_out[3]),
        "BoundingBoxBase::offset(delta) must match the oracle exactly for bb={bb:?} delta={delta}",
    );

    // Arm 5: `BoundingBox::rotated(angle)` (`BoundingBox.cpp:37-45`), the `.cpp` row's own
    // externally linked method.
    let angle = bounded_f64(input.angle_seed);
    let actual = bb.rotated(angle);
    let oracle_out = oracle::path1_rotated(min.x(), min.y(), max.x(), max.y(), angle);
    assert_eq!(
        (actual.min.x(), actual.min.y(), actual.max.x(), actual.max.y()),
        (oracle_out[0], oracle_out[1], oracle_out[2], oracle_out[3]),
        "BoundingBox::rotated(angle) must match the oracle exactly for bb={bb:?} angle={angle}",
    );

    // Arm 6: `BoundingBox::scaled(factor)` (`:57-62`), the `.cpp` row's own wrapper around
    // `scale`.
    let actual = bb.scaled(scale_factor);
    let oracle_out = oracle::path1_scaled(
        min.x(),
        min.y(),
        max.x(),
        max.y(),
        bb.defined,
        scale_factor,
    );
    assert_eq!(
        (
            actual.min.x(),
            actual.min.y(),
            actual.max.x(),
            actual.max.y(),
            actual.defined as i64
        ),
        (oracle_out[0], oracle_out[1], oracle_out[2], oracle_out[3], oracle_out[4]),
        "BoundingBox::scaled(factor) must match the oracle exactly for bb={bb:?} \
         factor={scale_factor}",
    );
});
