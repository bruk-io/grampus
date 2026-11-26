#pragma once

// Declarations only (see cxx/clipper2_shim.h's comment for why this crate always splits the
// cxx-generated header from the hand-written declarations file). `cxx_build::bridges` generates
// this header's counterpart at <crate-name>/src/proof_units/utils.rs.h on the include path it
// sets up, and `cxx/proof_units/utils.cpp` is the one translation unit that defines the
// function below.
#include "slic3r-ffi/src/proof_units/utils.rs.h"

#include <cstdint>

namespace slic3r_ffi {
namespace proof_units {
namespace utils {

std::uint64_t path1_get_utf8_sequence_length(rust::Vec<std::uint8_t> seq);

} // namespace utils
} // namespace proof_units
} // namespace slic3r_ffi
