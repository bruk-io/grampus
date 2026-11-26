#pragma once

// Declarations only (see cxx/clipper2_shim.h's comment for why this crate always splits the
// cxx-generated header from the hand-written declarations file). `cxx_build::bridges` generates
// this header's counterpart -- `rust::Vec` and the bridge's own signature -- at
// <crate-name>/src/proof_units/filament_mixer_model.rs.h on the include path it sets up, and
// `cxx/proof_units/filament_mixer_model.cpp` is the one translation unit that defines the
// function below.
#include "slic3r-ffi/src/proof_units/filament_mixer_model.rs.h"

namespace slic3r_ffi {
namespace proof_units {
namespace filament_mixer_model {

rust::Vec<uint8_t> path1_filament_mixer_lerp(uint8_t r1, uint8_t g1, uint8_t b1, uint8_t r2,
                                              uint8_t g2, uint8_t b2, float t);

} // namespace filament_mixer_model
} // namespace proof_units
} // namespace slic3r_ffi
