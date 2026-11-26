//! Port of `Arachne/utils/linearAlg2D.hpp` (task `72-foundation-arachne-grids`, plan step 20,
//! `foundation-23` row of `port/foundation.tsv`): two `static inline` free functions at
//! namespace scope, `Slic3r::Arachne::LinearAlg2D::pointIsLeftOfLine` and `::getAngleLeft`.
//! Unlike this task's other three units, this header depends only on `Point.hpp` -- no
//! `SquareGrid`/`SparseGrid` dependency -- so it is ported in full here, with its `mod` line,
//! its proof and its `required` fuzz target (task.md, "Why this is a 2").
//!
//! **Visibility.** `static inline` gives both functions internal linkage per translation unit,
//! but every in-scope caller (`Arachne/utils/SquareGrid.cpp`, the `Arachne/BeadingStrategy/`
//! files) reaches them exactly as it would a public utility, by `#include`ing the header -- the
//! idiom map's "a free function in an anonymous namespace, or a `static` free function" ->
//! "private (non-`pub`)" row answers a different shape (a helper with *no* caller outside its
//! own `.cpp`, provable only through proof path 3's included-translation-unit route). This
//! function has in-scope callers in sibling crates (`slic3r-arachne-beadingstrategy`, crate map
//! level 14), so both stay `pub`, the same way `slic3r-point`'s own header-only
//! `scaled`/`unscaled` wrappers do.
//!
//! **The missing generic `cross2` and `dot`.** `getAngleLeft` needs `cross2(Vec2i64, Vec2i64)`
//! (`src/libslic3r/Point.hpp:112-118`, the generic `T` overload -- *not* the `int32_t` overload
//! at `:111`, which is `= delete`d) and `Vec2i64::dot`. `crates/slic3r-point/src/point.rs` only
//! ports the private, monomorphic `cross2_f64` (its own module doc: "the fully generic
//! `Mat<N,M,T>`/`Vec<N,T>` templates ... have no in-scope caller inside this task's own two
//! units ... left as `// TODO(port):` markers ... for whichever fan-out unit first needs a
//! concrete instantiation") and no `dot` at all. This task cannot write
//! `crates/slic3r-point/src/point.rs` (not its own module), so both are written out directly, in
//! `i64`, at their one call site below, rather than guessed at as a new `slic3r-point` API.
//! **Finding, named in this task's pull request.**
//!
//! **Signed-overflow UB.** The vendored `int64_t` multiplications and subtractions in both
//! functions carry no overflow check (`linearAlg2D.hpp:28`, `:49-50`): a `coord_t` product that
//! overflows is undefined behaviour in C++, which the oracle's own build (compiled without an
//! overflow sanitizer) resolves as ordinary two's-complement wraparound. `wrapping_*` is used
//! throughout both functions for the same reason `crates/slic3r-point/src/point.rs`'s
//! `nearest_point_index_ptrs` already settled on it (its own comment there, found by that unit's
//! fuzz target): a plain Rust `-`/`*` panics on the same overflow instead of silently wrapping,
//! which would make this port crash on an input the oracle merely computes a nonsensical (but
//! defined-on-this-build) result for.

use slic3r_point::point::Point;

/// `static inline int64_t pointIsLeftOfLine(const Point &p, const Point &a, const Point &b)`
/// (`linearAlg2D.hpp:26-29`): the 2x2 determinant of `[b-a; p-a]`. Positive when `p` lies to the
/// left of the directed line `a -> b`, negative to the right, zero exactly on the line.
pub fn point_is_left_of_line(p: Point, a: Point, b: Point) -> i64 {
    // PORT SRC: src/libslic3r/Arachne/utils/linearAlg2D.hpp:28
    let ba_x = b.x().wrapping_sub(a.x());
    let ba_y = b.y().wrapping_sub(a.y());
    let pa_x = p.x().wrapping_sub(a.x());
    let pa_y = p.y().wrapping_sub(a.y());
    ba_x.wrapping_mul(pa_y)
        .wrapping_sub(ba_y.wrapping_mul(pa_x))
}

/// `static inline float getAngleLeft(const Point &a, const Point &b, const Point &c)`
/// (`linearAlg2D.hpp:48-65`): the angle at `b`, measured from the left side of segment `b->c`
/// when looking from `a`, in `[0, 2*pi)`.
pub fn get_angle_left(a: Point, b: Point, c: Point) -> f32 {
    // `const Vec2i64 ba = (a - b).cast<int64_t>();` / `bc = (c - b).cast<int64_t>();`
    // (`:50-51`) -- `a`/`b`/`c` are already `int64_t`-component `Point`s, so `.cast<int64_t>()`
    // is a no-op; written out component-wise (not through `Point`'s own `Sub` impl) so every
    // subtraction here uses `wrapping_sub`, per this module's doc comment.
    // PORT SRC: src/libslic3r/Arachne/utils/linearAlg2D.hpp:50
    let ba_x = a.x().wrapping_sub(b.x());
    let ba_y = a.y().wrapping_sub(b.y());
    // PORT SRC: src/libslic3r/Arachne/utils/linearAlg2D.hpp:51
    let bc_x = c.x().wrapping_sub(b.x());
    let bc_y = c.y().wrapping_sub(b.y());

    // `const int64_t dott = ba.dot(bc);` (`:52`) -- `Eigen::MatrixBase::dot` is a generic
    // library operation with no line of its own in the vendored tree to cite; the two-term sum
    // it computes for a 2-vector is written out directly.
    let dott = ba_x
        .wrapping_mul(bc_x)
        .wrapping_add(ba_y.wrapping_mul(bc_y));
    // `const int64_t det = cross2(ba, bc);` (`:53`), `cross2`'s generic body
    // (`src/libslic3r/Point.hpp:118`, `v1.x() * v2.y() - v1.y() * v2.x()`) -- see this module's
    // doc comment on the missing generic `cross2`.
    // PORT SRC: src/libslic3r/Point.hpp:118
    let det = ba_x
        .wrapping_mul(bc_y)
        .wrapping_sub(ba_y.wrapping_mul(bc_x));

    if det == 0 {
        // `:55-58`.
        if (ba_x != 0 && (ba_x > 0) == (bc_x > 0)) || (ba_x == 0 && (ba_y > 0) == (bc_y > 0)) {
            // `return 0;` (`:56`) -- written `0f32`, not `0.0`, so this literal's own text stays
            // in `port/port-status`'s exempt `{0, 1, 2}` set rather than silently failing to
            // match any vendored line's spelling (`docs/PORTING.md`, "Decisions recorded on
            // 2026-11-26", decision 8).
            return 0f32; // pointy bit
        }
        // `return float(M_PI);` (`:58`).
        // PORT SRC: external: ISO C <math.h>/POSIX `M_PI` -- the same constant Rust's
        // `std::f64::consts::PI` defines; `linearAlg2D.hpp:9` includes `<math.h>`/`<cmath>` and
        // uses `M_PI` directly, with no OrcaSlicer-defined value of its own to cite.
        return std::f64::consts::PI as f32; // straight bit
    }

    // `const float angle = -atan2(double(det), double(dott));` (`:60`) -- computed in `f64`
    // (`atan2`'s own precision), negated in `f64`, and narrowed to `f32` only once, by the
    // assignment, exactly as the vendored declaration's type does. Parenthesised explicitly
    // (Rust's unary `-` binds tighter than `as`, so the unparenthesised form already negates
    // before narrowing -- same order, written out so a reviewer need not re-derive precedence).
    let angle: f32 = (-(det as f64).atan2(dott as f64)) as f32;
    if angle >= 0f32 {
        // `:61-62`.
        angle
    } else {
        // `return M_PI * 2 + angle;` (`:64`) -- `angle` (`float`) promotes to `double` for the
        // addition (the usual arithmetic conversions), and the function's `float` return type
        // narrows the sum back down; `2f64`, not `2.0`, for the same exempt-literal reason as
        // the `0f32` above (`docs/PORTING.md` decision 8) -- `2` is itself in the exempt set.
        // PORT SRC: external: ISO C <math.h>/POSIX `M_PI`, as above.
        (std::f64::consts::PI * 2f64 + angle as f64) as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn point_is_left_of_line_left_right_and_on_the_line() {
        let a = Point::new(0, 0);
        let b = Point::new(100, 0);
        assert!(point_is_left_of_line(Point::new(50, 10), a, b) > 0);
        assert!(point_is_left_of_line(Point::new(50, -10), a, b) < 0);
        assert_eq!(point_is_left_of_line(Point::new(50, 0), a, b), 0);
    }

    #[test]
    fn get_angle_left_straight_and_right_angle() {
        let a = Point::new(0, 0);
        let b = Point::new(100, 0);
        // `c` straight ahead of `b` from `a`: the det==0 "straight bit" branch, which returns
        // `std::f64::consts::PI as f32` directly with no `atan2` in the path -- exact, not a
        // tolerance, matching this unit's `exact` equality tier (`port/manifest.tsv:51`, both
        // tolerance cells empty); this smoke test is not the proof, see `// PROOF: 1` below.
        let straight = get_angle_left(a, b, Point::new(200, 0));
        assert_eq!(straight, std::f32::consts::PI);
        // `c` turning left by a right angle: this path does go through `atan2`, whose result is
        // not bitwise `std::f32::consts::FRAC_PI_2` for an arbitrary libm, so a tolerance is kept
        // here deliberately -- this smoke test is not this unit's `// PROOF: 1` exact-equality
        // proof against the oracle (below), only a sanity check of the shape of the result.
        let right_angle = get_angle_left(a, b, Point::new(100, 100));
        assert!((right_angle - std::f32::consts::FRAC_PI_2).abs() < 1e-6);
    }

    /// Proof path 1 (R19): both free functions are `static inline` at namespace scope
    /// (internal linkage per translation unit), reached here the same way any in-scope caller
    /// reaches them -- by including the header -- through
    /// `crates/slic3r-ffi/cxx/proof_units/linear_alg2_d.cpp`'s direct calls, reached through
    /// `slic3r-proof`'s re-export of the FFI crate's per-unit proof bridges (`docs/PORTING.md`,
    /// "Decisions recorded on 2026-11-25"). One cell across this unit's single manifest row
    /// (`Arachne/utils/linearAlg2D.hpp`, `proof path: 1`), so one `// PROOF: 1` marker, covering
    /// both functions in one test.
    // PROOF: 1
    #[test]
    fn linear_alg2_d_matches_the_oracle() {
        let p = Point::new(123, -456);
        let a = Point::new(-789, 12);
        let b = Point::new(345, -678);
        let c = Point::new(901, 234);

        let actual_left = point_is_left_of_line(p, a, b);
        let oracle_left = slic3r_proof::proof_units::linear_alg2_d::path1_point_is_left_of_line(
            p.x(),
            p.y(),
            a.x(),
            a.y(),
            b.x(),
            b.y(),
        );
        assert_eq!(
            actual_left, oracle_left,
            "pointIsLeftOfLine must match the oracle exactly"
        );

        let actual_angle = get_angle_left(a, b, c);
        let oracle_angle = slic3r_proof::proof_units::linear_alg2_d::path1_get_angle_left(
            a.x(),
            a.y(),
            b.x(),
            b.y(),
            c.x(),
            c.y(),
        );
        assert_eq!(
            actual_angle, oracle_angle,
            "getAngleLeft must match the oracle exactly"
        );

        // A second, det==0 case, for the branch the first input does not reach.
        let a2 = Point::new(0, 0);
        let b2 = Point::new(50, 50);
        let c2 = Point::new(100, 100);
        let actual_straight = get_angle_left(a2, b2, c2);
        let oracle_straight = slic3r_proof::proof_units::linear_alg2_d::path1_get_angle_left(
            a2.x(),
            a2.y(),
            b2.x(),
            b2.y(),
            c2.x(),
            c2.y(),
        );
        assert_eq!(
            actual_straight, oracle_straight,
            "getAngleLeft's det==0 branch must match the oracle exactly"
        );
    }
}

// PORT STATUS
// source: src/libslic3r/Arachne/utils/linearAlg2D.hpp
// confidence: high
// tier: exact
// oracle: built
