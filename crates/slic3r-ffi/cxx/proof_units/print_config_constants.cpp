// Proof-path-1 shim for the `PrintConfigConstants.hpp` unit (R19), compiled onto
// `crates/slic3r-ffi`'s shared `cxx_build::Build` by this crate's `build.rs` -- the one
// sanctioned route a ported unit reaches the oracle by (`docs/PORTING.md`, "Decisions recorded
// on 2026-11-25"). `crates/slic3r-printconfigconstants`'s proof calls these through
// `slic3r_proof::proof_units::print_config_constants`.
//
// `#include`s the vendored header verbatim and unmodified, exactly as
// `cxx/proof_units/clipper_z_utils.cpp` does for a proof-path-3 unit elsewhere in this port.
// `PrintConfigConstants.hpp` includes nothing at all (three `#define`s and a `#pragma once`), so
// this translation unit needs no include path beyond `src/libslic3r` itself.
//
// Each macro is returned at the C++ type its own `PrintConfig.cpp` call site pins it to --
// `ConfigOptionFloat` wraps `double`, `ConfigOptionInt` wraps `int`, `ConfigOptionBool` wraps
// `bool` (`Config.hpp:824,1018,1907`) -- so the value the Rust side compares against is the one
// the oracle itself would store, not a value widened or narrowed in transit.
#include "print_config_constants.h"

#include "PrintConfigConstants.hpp"

namespace slic3r_ffi {
namespace proof_units {
namespace print_config_constants {

double path1_initial_layer_height() {
    const double value = INITIAL_LAYER_HEIGHT;
    return value;
}

int32_t path1_initial_raft_layers() {
    const int value = INITIAL_RAFT_LAYERS;
    return static_cast<int32_t>(value);
}

bool path1_initial_reduce_crossing_wall() {
    const bool value = INITIAL_REDUCE_CROSSING_WALL;
    return value;
}

} // namespace print_config_constants
} // namespace proof_units
} // namespace slic3r_ffi
