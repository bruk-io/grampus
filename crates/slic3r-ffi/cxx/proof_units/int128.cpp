// Proof path 1 (R19) for the `Int128.hpp` unit: `Int128.hpp` is header-only with no externally
// linked symbol of its own (every method is `inline`/template), so there is nothing to call
// through `liblibslic3r.a` -- this translation unit `#include`s the vendored header directly,
// unmodified, and the functions below call its methods fresh, compiled onto
// `crates/slic3r-ffi`'s shared `cxx_build::Build`, which carries the same `-ffp-contract=off`
// flag the normative oracle's own build uses (R17; `crates/slic3r-ffi/build.rs`'s own comment).
// `crates/slic3r-int128`'s proof calls these through `slic3r_proof::proof_units::int128` -- the
// one sanctioned route a ported unit reaches the oracle by (`docs/PORTING.md`, "Decisions
// recorded on 2026-11-25"). `Int128.hpp` includes nothing beyond `<cassert> <cstdint> <cmath>`,
// so this translation unit needs no other vendored or superbuild include path.
#include "int128.h"

#include "Int128.hpp"

#include <cstdint>

namespace slic3r_ffi {
namespace proof_units {
namespace int128 {

int64_t multiply_hi(int64_t lhs, int64_t rhs) {
    return Int128::multiply(lhs, rhs).hi();
}

uint64_t multiply_lo(int64_t lhs, int64_t rhs) {
    return Int128::multiply(lhs, rhs).lo();
}

double multiply_as_double(int64_t lhs, int64_t rhs) {
    return static_cast<double>(Int128::multiply(lhs, rhs));
}

// `int sign() const { return (value > 0) - (value < 0); }` (Int128.hpp:89) -- an adversarial-
// review finding (this task) that `sign()` itself had no oracle-verified test. **Not a genuinely
// 128-bit value**: `multiply()` narrows every result through `Int128(int64_t lo = 0)` (see
// Int128.hpp:106's own doc on the Rust side), so this is still effectively a bare `int64_t`
// construction for `sign()`'s purposes -- `add_assign_sign` below, driven through the
// non-narrowing `operator+=`, is this unit's real coverage of `sign()` outside the
// sign-extended-`i64` range.
int32_t multiply_sign(int64_t lhs, int64_t rhs) {
    return static_cast<int32_t>(Int128::multiply(lhs, rhs).sign());
}

int32_t sign_determinant_2x2(int64_t a11, int64_t a12, int64_t a21, int64_t a22) {
    return static_cast<int32_t>(Int128::sign_determinant_2x2(a11, a12, a21, a22));
}

int32_t compare_rationals(int64_t p1, int64_t q1, int64_t p2, int64_t q2) {
    return static_cast<int32_t>(Int128::compare_rationals(p1, q1, p2, q2));
}

int32_t sign_determinant_2x2_filtered(int64_t a11, int64_t a12, int64_t a21, int64_t a22) {
    return static_cast<int32_t>(Int128::sign_determinant_2x2_filtered(a11, a12, a21, a22));
}

int32_t compare_rationals_filtered(int64_t p1, int64_t q1, int64_t p2, int64_t q2) {
    return static_cast<int32_t>(Int128::compare_rationals_filtered(p1, q1, p2, q2));
}

// `operator+`/`operator-`/unary `operator-`/`operator+=` (adversarial review, task
// 53-foundation-int128-queue): these four return/mutate `hi()`/`lo()` through the real vendored
// operators, so the proof can tell the narrowing three (`+`, `-`, unary `-`) apart from the one
// that does not narrow (`+=`).
int64_t add_hi(int64_t lhs, int64_t rhs) {
    return (Int128(lhs) + Int128(rhs)).hi();
}

uint64_t add_lo(int64_t lhs, int64_t rhs) {
    return (Int128(lhs) + Int128(rhs)).lo();
}

int64_t sub_hi(int64_t lhs, int64_t rhs) {
    return (Int128(lhs) - Int128(rhs)).hi();
}

uint64_t sub_lo(int64_t lhs, int64_t rhs) {
    return (Int128(lhs) - Int128(rhs)).lo();
}

int64_t neg_hi(int64_t lhs) {
    return (-Int128(lhs)).hi();
}

uint64_t neg_lo(int64_t lhs) {
    return (-Int128(lhs)).lo();
}

int64_t add_assign_hi(int64_t lhs, int64_t rhs) {
    Int128 v(lhs);
    v += Int128(rhs);
    return v.hi();
}

uint64_t add_assign_lo(int64_t lhs, int64_t rhs) {
    Int128 v(lhs);
    v += Int128(rhs);
    return v.lo();
}

// Adversarial review (task 53-foundation-int128-queue, third pass): `multiply_sign` above drives
// `sign()`'s oracle comparison through `multiply()`, which (see Int128.hpp:106's own doc on the
// Rust side) narrows every result through the class's only viable constructor,
// `Int128(int64_t lo = 0)`, so every value it can return is, bit for bit, a sign-extended i64 --
// "a genuinely 128-bit value, not a bare int64_t construction" was therefore not actually true of
// that test. `operator+=` does not narrow (see its own doc), and reaches a value outside the i64
// range at, e.g., `lhs = rhs = INT64_MAX` (true sum `2^64 - 2`, far past `INT64_MAX`). These two
// functions close the gap for real, driving `sign()`/`operator double()` through that same
// non-narrowed value.
int32_t add_assign_sign(int64_t lhs, int64_t rhs) {
    Int128 v(lhs);
    v += Int128(rhs);
    return static_cast<int32_t>(v.sign());
}

double add_assign_as_double(int64_t lhs, int64_t rhs) {
    Int128 v(lhs);
    v += Int128(rhs);
    return static_cast<double>(v);
}

int64_t sub_assign_hi(int64_t lhs, int64_t rhs) {
    Int128 v(lhs);
    v -= Int128(rhs);
    return v.hi();
}

uint64_t sub_assign_lo(int64_t lhs, int64_t rhs) {
    Int128 v(lhs);
    v -= Int128(rhs);
    return v.lo();
}

} // namespace int128
} // namespace proof_units
} // namespace slic3r_ffi
