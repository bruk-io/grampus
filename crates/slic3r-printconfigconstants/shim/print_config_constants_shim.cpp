// Proof-path-1 shim for `PrintConfigConstants.hpp` (task 54-foundation-objectid-clonable, R19),
// compiled by this crate's own `build.rs` and linked into `crates/slic3r-printconfigconstants`'s
// `#[cfg(test)]` proof -- `proof_present`'s "a `#[cfg(test)]` module inside the module file
// itself" branch.
//
// `#include`s the vendored header verbatim and unmodified, exactly as
// `crates/slic3r-ffi/cxx/include_vendored.cpp` does for a proof-path-3 unit elsewhere in this
// port. `PrintConfigConstants.hpp` includes nothing at all (three `#define`s and a `#pragma
// once`), so this translation unit needs no include path beyond `src/libslic3r` itself.
//
// Each macro is returned at the C++ type its own `PrintConfig.cpp` call site pins it to --
// `ConfigOptionFloat` wraps `double`, `ConfigOptionInt` wraps `int`, `ConfigOptionBool` wraps
// `bool` (`Config.hpp:824,1018,1907`) -- so the value the Rust side compares against is the one
// the oracle itself would store, not a value widened or narrowed in transit.
#include "PrintConfigConstants.hpp"

#include <cstdint>

extern "C" {

double slic3r_printconfigconstants_shim_initial_layer_height(void) {
    const double value = INITIAL_LAYER_HEIGHT;
    return value;
}

int32_t slic3r_printconfigconstants_shim_initial_raft_layers(void) {
    const int value = INITIAL_RAFT_LAYERS;
    return static_cast<int32_t>(value);
}

int32_t slic3r_printconfigconstants_shim_initial_reduce_crossing_wall(void) {
    const bool value = INITIAL_REDUCE_CROSSING_WALL;
    return value ? 1 : 0;
}

} // extern "C"
