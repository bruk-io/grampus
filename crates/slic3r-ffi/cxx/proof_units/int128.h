#pragma once

// Declarations only (see cxx/clipper2_shim.h's comment for why this crate always splits the
// cxx-generated header from the hand-written declarations file). `cxx_build::bridges` generates
// this header's counterpart at <crate-name>/src/proof_units/int128.rs.h on the include path it
// sets up, and `cxx/proof_units/int128.cpp` is the one translation unit that defines the
// functions below.
#include "slic3r-ffi/src/proof_units/int128.rs.h"

namespace slic3r_ffi {
namespace proof_units {
namespace int128 {

int64_t multiply_hi(int64_t lhs, int64_t rhs);
uint64_t multiply_lo(int64_t lhs, int64_t rhs);
double multiply_as_double(int64_t lhs, int64_t rhs);
int32_t multiply_sign(int64_t lhs, int64_t rhs);
int32_t sign_determinant_2x2(int64_t a11, int64_t a12, int64_t a21, int64_t a22);
int32_t compare_rationals(int64_t p1, int64_t q1, int64_t p2, int64_t q2);
int32_t sign_determinant_2x2_filtered(int64_t a11, int64_t a12, int64_t a21, int64_t a22);
int32_t compare_rationals_filtered(int64_t p1, int64_t q1, int64_t p2, int64_t q2);
int64_t add_hi(int64_t lhs, int64_t rhs);
uint64_t add_lo(int64_t lhs, int64_t rhs);
int64_t sub_hi(int64_t lhs, int64_t rhs);
uint64_t sub_lo(int64_t lhs, int64_t rhs);
int64_t neg_hi(int64_t lhs);
uint64_t neg_lo(int64_t lhs);
int64_t add_assign_hi(int64_t lhs, int64_t rhs);
uint64_t add_assign_lo(int64_t lhs, int64_t rhs);
int64_t sub_assign_hi(int64_t lhs, int64_t rhs);
uint64_t sub_assign_lo(int64_t lhs, int64_t rhs);

} // namespace int128
} // namespace proof_units
} // namespace slic3r_ffi
