#pragma once

// Declarations only (see cxx/clipper2_shim.h's comment for why this crate always splits the
// cxx-generated header from the hand-written declarations file). `cxx_build::bridges` generates
// this header's counterpart at <crate-name>/src/proof_units/lifecycle_events.rs.h on the
// include path it sets up, and `cxx/proof_units/lifecycle_events.cpp` is the one translation
// unit that defines the functions below.
#include "slic3r-ffi/src/proof_units/lifecycle_events.rs.h"

namespace slic3r_ffi {
namespace proof_units {
namespace lifecycle_events {

rust::String lifecycle_event_to_string(int32_t event);
rust::String lifecycle_evt_code_to_string(int32_t code);

} // namespace lifecycle_events
} // namespace proof_units
} // namespace slic3r_ffi
