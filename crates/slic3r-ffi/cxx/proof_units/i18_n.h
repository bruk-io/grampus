#pragma once

// Declarations only (see cxx/clipper2_shim.h's comment for why this crate always splits the
// cxx-generated header from the hand-written declarations file). `cxx_build::bridges` generates
// this header's counterpart -- `rust::String` and the bridge's own signature -- at
// <crate-name>/src/proof_units/i18_n.rs.h on the include path it sets up, and
// `cxx/proof_units/i18_n.cpp` is the one translation unit that defines the function below.
#include "slic3r-ffi/src/proof_units/i18_n.rs.h"

namespace slic3r_ffi {
namespace proof_units {
namespace i18_n {

rust::String path2_translate(rust::Str s);
rust::String path2_translate_with_callback(rust::Str s);

} // namespace i18_n
} // namespace proof_units
} // namespace slic3r_ffi
