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
