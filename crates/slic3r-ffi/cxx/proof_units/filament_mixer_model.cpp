// Proof shim for the `FilamentMixerModel.hpp` unit (R19, proof path 1, task
// 52-foundation-common-defs, `port/foundation.tsv` row `foundation-01`), compiled onto
// `crates/slic3r-ffi`'s shared `cxx_build::Build` by this crate's `build.rs` -- the one
// sanctioned route a ported unit reaches the oracle by (`docs/PORTING.md`, "Decisions recorded
// on 2026-11-25"). `crates/slic3r-filamentmixermodel`'s proof calls this through
// `slic3r_proof::proof_units::filament_mixer_model`.
//
// `Slic3r::filament_mixer_lerp` (`src/libslic3r/FilamentMixer.cpp:58-64`) is an externally
// linked symbol in the archive `just oracle-build` produces: a pure forwarder into
// `::filament_mixer::lerp` with no transformation of its own, verified by reading
// `FilamentMixer.cpp:58-64` directly. It takes its three output channels by pointer
// (`FilamentMixer.hpp:67-70`); a `cxx` bridge function may not take an out-parameter, so this
// shim supplies three local `unsigned char`s for them and returns the three results as a
// 3-element `rust::Vec<uint8_t>` (r, g, b, in that order).
#include "filament_mixer_model.h"

#include "FilamentMixer.hpp"

namespace slic3r_ffi {
namespace proof_units {
namespace filament_mixer_model {

rust::Vec<uint8_t> path1_filament_mixer_lerp(uint8_t r1, uint8_t g1, uint8_t b1, uint8_t r2,
                                              uint8_t g2, uint8_t b2, float t) {
    unsigned char out_r = 0;
    unsigned char out_g = 0;
    unsigned char out_b = 0;
    Slic3r::filament_mixer_lerp(r1, g1, b1, r2, g2, b2, t, &out_r, &out_g, &out_b);
    rust::Vec<uint8_t> out;
    out.push_back(out_r);
    out.push_back(out_g);
    out.push_back(out_b);
    return out;
}

} // namespace filament_mixer_model
} // namespace proof_units
} // namespace slic3r_ffi
