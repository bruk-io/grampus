//! Port of `Int128.hpp` (task 53-foundation-int128-queue, plan step 20). A 128-bit signed integer
//! type for 64x64->128 multiply, add, subtract and compare, used by Clipper2-derived exact
//! geometric predicates -- `sign_determinant_2x2` and `compare_rationals` (and their "filtered"
//! fast paths) are the two load-bearing entry points callers reach for.
//!
//! The vendored header has two branches: an intrinsic-`__int128` path
//! (`HAS_INTRINSIC_128_TYPE`, active whenever the compiler is not MSVC and defines
//! `__SIZEOF_INT128__`) and a portable path that splits the value into a `hi`/`lo` pair of 64-bit
//! words. The pin's oracle is built on arm64 with Apple clang, which always defines
//! `__SIZEOF_INT128__`, so the intrinsic path is the only one this build ever reaches; the
//! portable path is dead code on this target and is not ported (`// PORT NOTE:` below). Rust's
//! native `i128` has the same 128-bit two's-complement semantics as the intrinsic path's `__int128`
//! (wraps the same way on overflow, and `as i128`/`as f64` casts match C++'s own
//! implicit/`static_cast` integer<->integer and integer->double conversions bit for bit for every
//! value this type's own callers construct), so this module ports that branch's methods directly
//! against `i128` rather than reproducing the hi/lo split. Four methods -- `multiply`,
//! `Add::add`, `Sub::sub` and `Neg::neg` -- are a measured exception to "matches bit for bit":
//! each funnels its true 128-bit result through the class's only viable constructor,
//! `Int128(int64_t lo = 0) : value(lo) {}`, a narrowing, sign-extending `__int128` -> `int64_t`
//! -> `__int128` round trip that keeps only the low 64 bits. See each method's own doc comment
//! for the measurement. `AddAssign`/`SubAssign` (`operator+=`/`operator-=`) assign to `value`
//! directly and do **not** share this truncation -- they are the non-truncating counterparts of
//! the same two expressions.

/// A 128-bit signed integer, ported from `Int128.hpp`'s `HAS_INTRINSIC_128_TYPE` branch (the only
/// branch the pinned oracle's arm64 build reaches -- see the module doc).
///
/// `#[derive(PartialOrd, Ord)]` is this unit's order choice for `operator<=`
/// (`src/libslic3r/Int128.hpp:96`, the intrinsic branch's own comparator, `return value <=
/// rhs.value`): `port/reports/order-comparators.md`'s row for that site records `diverges: yes`,
/// shape `non_strict` ("equal elements compare true by definition"), port's choice "the strict
/// form of the same relation" -- i.e. derive `<=` from the strict `<` rather than transcribe the
/// non-strict body directly, which is exactly what `#[derive(..., PartialOrd, Ord)]` does over a
/// single `i128` field (`Ord::le` is `!matches!(self.cmp(other), Greater)`, built from `cmp`, not
/// a separately transcribed non-strict expression). The same report's rows for
/// `Int128.hpp:143`/`:144`/`:146` (`operator>`/`operator<`/`operator<=` in the portable, non-
/// intrinsic branch) need no decision here: that whole branch is dead code on this build and is
/// not ported at all (see the `// PORT NOTE:` near the end of this file).
///
/// // PORT SRC: src/libslic3r/Int128.hpp:80
// FIDELITY(port): src/libslic3r/Int128.hpp:96 -- order-comparators.md row 53/56: the non-strict
// `operator<=` is rendered as the strict total order's derived form, per that report's resolved
// shape for this site.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Int128 {
    value: i128,
}

impl Int128 {
    /// `Int128(int64_t lo = 0) : value(lo) {}` -- sign-extends, matching the C++ constructor's
    /// implicit `int64_t` -> `__int128` conversion.
    ///
    /// // PORT SRC: src/libslic3r/Int128.hpp:82
    pub const fn from_i64(lo: i64) -> Self {
        Int128 { value: lo as i128 }
    }

    /// `uint64_t lo() const { return uint64_t(value); }`
    ///
    /// // PORT SRC: src/libslic3r/Int128.hpp:87
    pub fn lo(self) -> u64 {
        self.value as u64
    }

    /// `int64_t hi() const { return int64_t(value >> 64); }`
    ///
    /// // PORT SRC: src/libslic3r/Int128.hpp:88
    pub fn hi(self) -> i64 {
        // PORT SRC: src/libslic3r/Int128.hpp:88
        (self.value >> 64) as i64
    }

    /// `int sign() const { return (value > 0) - (value < 0); }`
    ///
    /// // PORT SRC: src/libslic3r/Int128.hpp:89
    pub fn sign(self) -> i32 {
        i32::from(self.value > 0) - i32::from(self.value < 0)
    }

    /// `static inline Int128 multiply(int64_t lhs, int64_t rhs) { return
    /// Int128(__int128(lhs) * __int128(rhs)); }`
    ///
    /// **Measured, surprising divergence from the mathematically "obvious" reading of this
    /// line, reproduced here rather than fixed.** The class's only viable constructor for this
    /// call is `Int128(int64_t lo = 0) : value(lo) {}` -- there is no constructor taking
    /// `__int128` -- so `__int128(lhs) * __int128(rhs)`, the exact 128-bit product, is passed
    /// through an implicit, narrowing `__int128` -> `int64_t` conversion before `value` (itself
    /// `__int128`) is sign-extended back up from it. The net effect: **this standalone function**
    /// keeps only the low 64 bits of the true product, sign-extended -- not the true 128-bit
    /// product -- on this (`HAS_INTRINSIC_128_TYPE`) branch, for every call whose true product
    /// does not fit in 64 bits. Confirmed directly against the real oracle, not reasoned from the
    /// source alone: `Int128::multiply(INT64_MAX, INT64_MAX)` (whose true product is
    /// `85070591730234615847396907784232501249`, requiring 126 bits) returns `hi() == 0`,
    /// `lo() == 1` -- exactly the low 64 bits of that product (`1`), not the product itself (see
    /// this task's pull request body for the standalone C++ measurement). **This bug is not
    /// local to this one function** (an earlier version of this module doc said it was; that was
    /// wrong, caught in adversarial review): `Add::add`, `Sub::sub` and `Neg::neg` below pass
    /// through the exact same narrowing constructor and truncate the same way -- see their own
    /// doc comments. `sign_determinant_2x2` and `compare_rationals` below do not call `multiply`
    /// on this branch -- each has its own inline `__int128` arithmetic that never passes through
    /// `Int128`'s narrowing constructor -- so they, and only they among this module's arithmetic,
    /// are unaffected and are ported below with the true, untruncated 128-bit product.
    ///
    /// // PORT SRC: src/libslic3r/Int128.hpp:106
    pub fn multiply(lhs: i64, rhs: i64) -> Self {
        let full_product: i128 = i128::from(lhs) * i128::from(rhs);
        Int128 {
            // `as i64` truncates to the low 64 bits the same way the C++ narrowing conversion
            // does, and the surrounding cast back to `i128` sign-extends it the same way the
            // `Int128(int64_t lo)` constructor does.
            value: i128::from(full_product as i64),
        }
    }

    /// ```c++
    /// static int sign_determinant_2x2(int64_t a11, int64_t a12, int64_t a21, int64_t a22)
    /// {
    ///     __int128 det = __int128(a11) * __int128(a22) - __int128(a12) * __int128(a21);
    ///     return (det > 0) - (det < 0);
    /// }
    /// ```
    /// Direct `__int128` arithmetic, not `Int128::multiply()` -- see that method's own doc for
    /// why the distinction matters on this branch.
    ///
    /// // PORT SRC: src/libslic3r/Int128.hpp:109
    pub fn sign_determinant_2x2(a11: i64, a12: i64, a21: i64, a22: i64) -> i32 {
        let det: i128 = i128::from(a11) * i128::from(a22) - i128::from(a12) * i128::from(a21);
        i32::from(det > 0) - i32::from(det < 0)
    }

    /// ```c++
    /// static int compare_rationals(int64_t p1, int64_t q1, int64_t p2, int64_t q2)
    /// {
    ///     int invert = ((q1 < 0) == (q2 < 0)) ? 1 : -1;
    ///     __int128 det = __int128(p1) * __int128(q2) - __int128(p2) * __int128(q1);
    ///     return ((det > 0) - (det < 0)) * invert;
    /// }
    /// ```
    /// Direct `__int128` arithmetic, not `Int128::multiply()` -- same distinction as
    /// `sign_determinant_2x2` above.
    ///
    /// // PORT SRC: src/libslic3r/Int128.hpp:116
    pub fn compare_rationals(p1: i64, q1: i64, p2: i64, q2: i64) -> i32 {
        let invert: i32 = if (q1 < 0) == (q2 < 0) { 1 } else { -1 };
        let det: i128 = i128::from(p1) * i128::from(q2) - i128::from(p2) * i128::from(q1);
        (i32::from(det > 0) - i32::from(det < 0)) * invert
    }

    /// `static int sign_determinant_2x2_filtered(...)` -- a numeric filter over the top 31 bits
    /// of each input that avoids the 128-bit multiply above when the filtered determinant's
    /// magnitude provably exceeds the maximum possible rounding error, falling back to the exact
    /// (unfiltered) computation otherwise.
    ///
    /// `(a11 + (1 << 31)) >> 32`: in the vendored C++, `1 << 31` is an `int` (32-bit) literal
    /// shift that sets the sign bit, so its *value*, sign-extended to `int64_t` by the following
    /// addition, is `-2147483648` -- not `+2147483648`. Measured directly against the real oracle
    /// (`crates/slic3r-ffi/cxx/proof_units/int128.cpp`'s `sign_determinant_2x2_filtered`, called
    /// from this module's own `#[cfg(test)]` proof below) rather than assumed: the two hypotheses
    /// disagree at inputs that straddle the filter's `abs(det) > err` branch, and the committed
    /// test below is what the oracle actually returns. `ROUND_BIAS` is `i32::MIN as i64`, the
    /// same bit pattern `1 << 31` produces as a 32-bit `int`.
    ///
    /// `wrapping_add`, not `+`: at `a11 == i64::MIN` (and a handful of other inputs this task's
    /// own test commits), `a11 + ROUND_BIAS` is a C++ signed-integer overflow -- formally
    /// undefined behaviour, but measured (with and without `-fwrapv`) to produce the same
    /// two's-complement wraparound result on this compiler and target either way, since the
    /// generated `add` instruction simply wraps. Reproduced here as a `wrapping_add` rather than
    /// left to panic, since a panic would itself be a divergence from what the oracle's compiled
    /// binary actually returns for this input.
    ///
    /// // PORT SRC: src/libslic3r/Int128.hpp:260
    pub fn sign_determinant_2x2_filtered(a11: i64, a12: i64, a21: i64, a22: i64) -> i32 {
        const ROUND_BIAS: i64 = i32::MIN as i64;
        // PORT SRC: src/libslic3r/Int128.hpp:264
        let a11s = a11.wrapping_add(ROUND_BIAS) >> 32;
        // PORT SRC: src/libslic3r/Int128.hpp:265
        let a12s = a12.wrapping_add(ROUND_BIAS) >> 32;
        // PORT SRC: src/libslic3r/Int128.hpp:266
        let a21s = a21.wrapping_add(ROUND_BIAS) >> 32;
        // PORT SRC: src/libslic3r/Int128.hpp:267
        let a22s = a22.wrapping_add(ROUND_BIAS) >> 32;
        let det = a11s * a22s - a12s * a21s;
        // `err` is always `>= 1` (the `+ 1` below is unconditional), which is the unstated
        // invariant `i32::from(det > 0) - i32::from(det < 0)` below relies on: the vendored
        // branch is `(det > 0) ? 1 : -1` (Int128.hpp:274), which is `-1` at `det == 0` and
        // therefore diverges from `i32::from(det > 0) - i32::from(det < 0)` (`0` at `det == 0`)
        // -- but this branch is only ever taken when `det.abs() > err >= 1`, so `det == 0` never
        // reaches it on either side. Verified equivalent over 300,000 inputs (adversarial
        // review, this task), not merely argued; written this way rather than transcribing the
        // ternary literally because the two differ (wrongly, for a `det` that could be zero) at
        // this one value, and the invariant that makes them agree here is worth a comment rather
        // than left to be rediscovered.
        let err = ((a11s.abs() + a12s.abs() + a21s.abs() + a22s.abs()) << 1) + 1;
        debug_assert!(
            det.abs() <= err
                || (i32::from(det > 0) - i32::from(det < 0))
                    == Self::sign_determinant_2x2(a11, a12, a21, a22)
        );
        if det.abs() > err {
            i32::from(det > 0) - i32::from(det < 0)
        } else {
            Self::sign_determinant_2x2(a11, a12, a21, a22)
        }
    }

    /// `static int compare_rationals_filtered(...)` -- the same numeric-filter idiom as
    /// `sign_determinant_2x2_filtered` above, applied to `compare_rationals`.
    ///
    /// // PORT SRC: src/libslic3r/Int128.hpp:279
    pub fn compare_rationals_filtered(p1: i64, q1: i64, p2: i64, q2: i64) -> i32 {
        const ROUND_BIAS: i64 = i32::MIN as i64;
        let invert: i32 = if (q1 < 0) == (q2 < 0) { 1 } else { -1 };
        // PORT SRC: src/libslic3r/Int128.hpp:284
        let q1s = q1.wrapping_add(ROUND_BIAS) >> 32;
        // PORT SRC: src/libslic3r/Int128.hpp:285
        let q2s = q2.wrapping_add(ROUND_BIAS) >> 32;
        if q1s != 0 && q2s != 0 {
            // PORT SRC: src/libslic3r/Int128.hpp:287
            let p1s = p1.wrapping_add(ROUND_BIAS) >> 32;
            // PORT SRC: src/libslic3r/Int128.hpp:288
            let p2s = p2.wrapping_add(ROUND_BIAS) >> 32;
            let det = p1s * q2s - p2s * q1s;
            // Same unstated invariant as `sign_determinant_2x2_filtered` above: `err >= 1`
            // always, so `i32::from(det > 0) - i32::from(det < 0)` below only runs where
            // `det != 0`, which is where it agrees with the vendored `(det > 0) ? 1 : -1`.
            let err = ((p1s.abs() + q1s.abs() + p2s.abs() + q2s.abs()) << 1) + 1;
            debug_assert!(
                det.abs() <= err
                    || (i32::from(det > 0) - i32::from(det < 0)) * invert
                        == Self::compare_rationals(p1, q1, p2, q2)
            );
            if det.abs() > err {
                return (i32::from(det > 0) - i32::from(det < 0)) * invert;
            }
        }
        Self::sign_determinant_2x2(p1, q1, p2, q2) * invert
    }

    /// `operator double() const { return double(value); }` -- Rust's `as` cast from `i128` to
    /// `f64` rounds to nearest, same as C++'s `static_cast<double>(__int128)`.
    ///
    /// // PORT SRC: src/libslic3r/Int128.hpp:104
    pub fn as_f64(self) -> f64 {
        self.value as f64
    }
}

impl std::ops::AddAssign for Int128 {
    /// `Int128& operator+=(const Int128 &rhs) { value += rhs.value; return *this; }` --
    /// `value` is `__int128` and this assigns to it directly, so unlike `operator+` below it
    /// does **not** pass through the narrowing `Int128(int64_t lo)` constructor. Measured
    /// directly (this task's pull request body, "standalone C++ measurement"): this is the one
    /// operator of the four whose output never truncates.
    ///
    /// // PORT SRC: src/libslic3r/Int128.hpp:98
    fn add_assign(&mut self, rhs: Int128) {
        self.value += rhs.value;
    }
}

impl std::ops::SubAssign for Int128 {
    /// `Int128& operator-=(const Int128 &rhs) { value -= rhs.value; return *this; }` -- same
    /// non-narrowing shape as `operator+=` above.
    ///
    /// // PORT SRC: src/libslic3r/Int128.hpp:100
    fn sub_assign(&mut self, rhs: Int128) {
        self.value -= rhs.value;
    }
}

impl std::ops::Add for Int128 {
    type Output = Int128;
    /// `Int128 operator+ (const Int128 &rhs) const { return Int128(value + rhs.value); }` --
    /// **measured, truncating divergence, reproduced here rather than fixed** (the same
    /// narrowing-constructor quirk `multiply()` above documents, and *not* local to that one
    /// function as this module previously claimed: see the corrected module doc at the top of
    /// this file). `value + rhs.value` is the true 128-bit sum, but the class's only viable
    /// constructor for the implicit `Int128(...)` wrapping the result is
    /// `Int128(int64_t lo = 0) : value(lo) {}` -- there is no `Int128(__int128)` overload -- so
    /// the sum is narrowed to its low 64 bits and sign-extended back before `value` is set.
    /// Confirmed directly against the real oracle, not reasoned from the source alone:
    /// `Int128((int64_t)INT64_MAX) + Int128((int64_t)INT64_MAX)` returns `hi() == -1`,
    /// `lo() == 18446744073709551614` (value `-2`) -- the low 64 bits of the true sum
    /// `18446744073709551614`, sign-extended, not the true (untruncated) sum itself (see this
    /// task's pull request body for the standalone C++ measurement). `operator+=` above is the
    /// non-truncating counterpart of this same expression.
    ///
    /// // PORT SRC: src/libslic3r/Int128.hpp:99
    fn add(self, rhs: Int128) -> Int128 {
        let full_sum: i128 = self.value + rhs.value;
        Int128 {
            // `as i64` truncates to the low 64 bits the same way the implicit narrowing
            // `__int128` -> `int64_t` conversion does, and the surrounding cast back to `i128`
            // sign-extends it the same way the `Int128(int64_t lo)` constructor does.
            value: i128::from(full_sum as i64),
        }
    }
}

impl std::ops::Sub for Int128 {
    type Output = Int128;
    /// `Int128 operator -(const Int128 &rhs) const { return Int128(value - rhs.value); }` --
    /// same narrowing-constructor quirk as `operator+` above; see that method's own doc.
    /// Measured: `Int128((int64_t)INT64_MAX) - Int128((int64_t)INT64_MIN)` returns `hi() == -1`,
    /// `lo() == 18446744073709551615` (value `-1`), the low 64 bits of the true difference,
    /// sign-extended, not the true difference itself. `operator-=` above is the non-truncating
    /// counterpart of this same expression.
    ///
    /// // PORT SRC: src/libslic3r/Int128.hpp:101
    fn sub(self, rhs: Int128) -> Int128 {
        let full_diff: i128 = self.value - rhs.value;
        // See `operator+`'s own doc for why this narrows the same way the vendored constructor
        // does.
        Int128 {
            value: i128::from(full_diff as i64),
        }
    }
}

impl std::ops::Neg for Int128 {
    type Output = Int128;
    /// `Int128 operator -() const { return Int128(- value); }` -- same narrowing-constructor
    /// quirk as `operator+`/`operator-` above (there is no non-narrowing counterpart for unary
    /// negation: the class has no `operator-=` taking no argument). Measured:
    /// `-Int128((int64_t)INT64_MIN)` returns `hi() == -1`, `lo() == 9223372036854775808`
    /// (value `-9223372036854775808`, i.e. itself) -- the low 64 bits of the true negation
    /// `9223372036854775808`, sign-extended, not the true (positive, 64-bit-overflowing)
    /// negation.
    ///
    /// // PORT SRC: src/libslic3r/Int128.hpp:102
    fn neg(self) -> Int128 {
        let full_neg: i128 = -self.value;
        // See `operator+`'s own doc for why this narrows the same way the vendored constructor
        // does.
        Int128 {
            value: i128::from(full_neg as i64),
        }
    }
}

// PORT NOTE: Int128.hpp:123-252's portable (non-intrinsic) hi/lo-split branch is never reached by
// the pinned oracle's arm64 build (Apple clang always defines __SIZEOF_INT128__, so
// HAS_INTRINSIC_128_TYPE is always set at this pin -- vendor/OrcaSlicer/src/libslic3r/Int128.hpp:56-58)
// and is therefore not ported: Rust's i128 already gives the intrinsic branch's own semantics
// directly (see the module doc), so there is no second code path to keep in sync with.

#[cfg(test)]
mod tests {
    use super::Int128;
    use slic3r_proof::proof_units::int128::{
        add_assign_hi, add_assign_lo, add_hi, add_lo, compare_rationals,
        compare_rationals_filtered, multiply_as_double, multiply_hi, multiply_lo, neg_hi, neg_lo,
        sign_determinant_2x2, sign_determinant_2x2_filtered, sub_assign_hi, sub_assign_lo, sub_hi,
        sub_lo,
    };

    const PAIRS: &[(i64, i64)] = &[
        (0, 0),
        (1, 1),
        (-1, 1),
        (1, -1),
        (-1, -1),
        (i64::MAX, i64::MAX),
        (i64::MIN, i64::MAX),
        (i64::MIN, i64::MIN),
        (i64::MIN, 1),
        (i64::MAX, -1),
        (1_000_000, -1_000_000),
        (3, 7),
        (2_147_483_648, 2_147_483_648),
        (-2_147_483_648, 2_147_483_647),
    ];

    // PROOF: 1
    #[test]
    fn int128_multiply_matches_oracle() {
        for &(lhs, rhs) in PAIRS {
            let expected_hi = multiply_hi(lhs, rhs);
            let expected_lo = multiply_lo(lhs, rhs);
            let expected_f64 = multiply_as_double(lhs, rhs);
            let actual = Int128::multiply(lhs, rhs);
            assert_eq!(actual.hi(), expected_hi, "hi() for multiply({lhs}, {rhs})");
            assert_eq!(actual.lo(), expected_lo, "lo() for multiply({lhs}, {rhs})");
            assert_eq!(
                actual.as_f64().to_bits(),
                expected_f64.to_bits(),
                "as_f64() for multiply({lhs}, {rhs})"
            );
        }
    }

    // PROOF: 1
    #[test]
    fn int128_add_sub_neg_match_oracle() {
        for &(lhs, rhs) in PAIRS {
            let a = Int128::from_i64(lhs);
            let b = Int128::from_i64(rhs);

            let sum = a + b;
            let expected_add_hi = add_hi(lhs, rhs);
            let expected_add_lo = add_lo(lhs, rhs);
            assert_eq!(sum.hi(), expected_add_hi, "hi() for {lhs} + {rhs}");
            assert_eq!(sum.lo(), expected_add_lo, "lo() for {lhs} + {rhs}");

            let diff = a - b;
            let expected_sub_hi = sub_hi(lhs, rhs);
            let expected_sub_lo = sub_lo(lhs, rhs);
            assert_eq!(diff.hi(), expected_sub_hi, "hi() for {lhs} - {rhs}");
            assert_eq!(diff.lo(), expected_sub_lo, "lo() for {lhs} - {rhs}");

            let neg = -a;
            let expected_neg_hi = neg_hi(lhs);
            let expected_neg_lo = neg_lo(lhs);
            assert_eq!(neg.hi(), expected_neg_hi, "hi() for -{lhs}");
            assert_eq!(neg.lo(), expected_neg_lo, "lo() for -{lhs}");

            // `+=`/`-=` do not narrow (unlike `+`/`-` above) -- this is the proof that the two
            // pairs genuinely diverge, not just an assertion in a doc comment.
            let mut add_assigned = a;
            add_assigned += b;
            let expected_add_assign_hi = add_assign_hi(lhs, rhs);
            let expected_add_assign_lo = add_assign_lo(lhs, rhs);
            assert_eq!(
                add_assigned.hi(),
                expected_add_assign_hi,
                "hi() for {lhs} += {rhs}"
            );
            assert_eq!(
                add_assigned.lo(),
                expected_add_assign_lo,
                "lo() for {lhs} += {rhs}"
            );

            let mut sub_assigned = a;
            sub_assigned -= b;
            let expected_sub_assign_hi = sub_assign_hi(lhs, rhs);
            let expected_sub_assign_lo = sub_assign_lo(lhs, rhs);
            assert_eq!(
                sub_assigned.hi(),
                expected_sub_assign_hi,
                "hi() for {lhs} -= {rhs}"
            );
            assert_eq!(
                sub_assigned.lo(),
                expected_sub_assign_lo,
                "lo() for {lhs} -= {rhs}"
            );
        }
    }

    const QUADS: &[(i64, i64, i64, i64)] = &[
        (0, 0, 0, 0),
        (1, 2, 3, 4),
        (-1, 2, 3, -4),
        (i64::MAX, 1, 1, i64::MAX),
        (i64::MIN, 1, 1, i64::MIN),
        (1_000_000, 1_000_000, 1_000_000, 1_000_000),
        (2, 3, 6, 9),
        (5, -3, -5, 3),
        // Values that straddle sign_determinant_2x2_filtered's numeric-filter boundary: each
        // operand near a multiple of 2^32 so the rounded-to-31-bits approximation sits close to
        // its own maximum possible error.
        (1i64 << 32, 1, 1, (1i64 << 32) - 1),
        (-(1i64 << 32), 1, 1, (1i64 << 32) - 1),
        (
            (1i64 << 31) - 1,
            (1i64 << 31) - 1,
            (1i64 << 31) - 1,
            (1i64 << 31) - 1,
        ),
        (
            -(1i64 << 31),
            (1i64 << 31) - 1,
            (1i64 << 31) - 1,
            -(1i64 << 31),
        ),
        (i64::MAX, i64::MAX, i64::MAX, i64::MAX - 1),
    ];

    // PROOF: 1
    #[test]
    fn int128_sign_determinant_2x2_matches_oracle() {
        for &(a11, a12, a21, a22) in QUADS {
            let expected = sign_determinant_2x2(a11, a12, a21, a22);
            assert_eq!(
                Int128::sign_determinant_2x2(a11, a12, a21, a22),
                expected,
                "sign_determinant_2x2({a11}, {a12}, {a21}, {a22})"
            );
        }
    }

    // PROOF: 1
    #[test]
    fn int128_compare_rationals_matches_oracle() {
        for &(p1, q1, p2, q2) in QUADS {
            let expected = compare_rationals(p1, q1, p2, q2);
            assert_eq!(
                Int128::compare_rationals(p1, q1, p2, q2),
                expected,
                "compare_rationals({p1}, {q1}, {p2}, {q2})"
            );
        }
    }

    // PROOF: 1
    #[test]
    fn int128_sign_determinant_2x2_filtered_matches_oracle() {
        for &(a11, a12, a21, a22) in QUADS {
            let expected = sign_determinant_2x2_filtered(a11, a12, a21, a22);
            assert_eq!(
                Int128::sign_determinant_2x2_filtered(a11, a12, a21, a22),
                expected,
                "sign_determinant_2x2_filtered({a11}, {a12}, {a21}, {a22})"
            );
        }
    }

    // PROOF: 1
    #[test]
    fn int128_compare_rationals_filtered_matches_oracle() {
        for &(p1, q1, p2, q2) in QUADS {
            let expected = compare_rationals_filtered(p1, q1, p2, q2);
            assert_eq!(
                Int128::compare_rationals_filtered(p1, q1, p2, q2),
                expected,
                "compare_rationals_filtered({p1}, {q1}, {p2}, {q2})"
            );
        }
    }
}

// PORT STATUS
// source: src/libslic3r/Int128.hpp
// confidence: high
// tier: exact
// oracle: built
