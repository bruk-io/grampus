// Proof shim for the `MaterialType.cpp`/`MaterialType.hpp` unit (R19), the fixed-point
// (near-miss) leg of task `35-trial-run`. Compiled onto `crates/slic3r-ffi`'s shared
// `cxx_build::Build` by this crate's `build.rs` -- the one sanctioned route a ported unit
// reaches the oracle by (`docs/PORTING.md`, "Decisions recorded on 2026-11-25").
// `crates/slic3r-materialtype`'s proof calls this through
// `slic3r_proof::proof_units::material_type`.
//
// **Mechanism finding, recorded here and in this task's pull request body.** This row's seeded
// `proof path` cell is `3` -- the "include the vendored `.cpp` whole and forward to its
// internal-linkage helper" mechanism decision 1 states for a `static`/anonymous-namespace
// function. `Slic3r::MaterialType`'s five methods are `public` class statics, not internal
// linkage: they are already externally linked symbols in the archive the derived link line
// carries (`liblibslic3r.a`'s `MaterialType.cpp.o`). Including `MaterialType.cpp` whole here
// would therefore *redefine* those same symbols inside this crate's own shared `Build`, which
// would fail to link with a duplicate-symbol error against the archive -- unlike
// `clipper_z_utils.cpp`'s own path-3 shim, whose `to_paths64` really is a `static` helper with
// no competing external definition. So this shim calls the already-externally-linked symbols
// directly instead (the path-1 mechanism), by just including the header. `// PROOF: 3` is still
// the marker this unit's Rust test carries, because that is the row's own manifest cell
// verbatim -- `port/port-status`'s `proof_present` matches a test's marker against the cell, not
// against a chosen mechanism -- so the mismatch between the seeded mechanism and the one that
// actually works is a finding for whoever regenerates the proof-path seeding rule, the same
// shape `crates/slic3r-commondefs/src/common_defs.rs`'s own `// PROOF: 2` doc comment already
// recorded for a different wrong seeding (a `proof path` cell seeded from a row's pipeline
// stage rather than from whether its functions are internally or externally linked).
#include "material_type.h"

#include "MaterialType.hpp"

#include <string>

namespace slic3r_ffi {
namespace proof_units {
namespace material_type {

rust::Vec<int64_t> path_temperature_range(rust::Str name) {
    int min_temp = 0;
    int max_temp = 0;
    const bool found = Slic3r::MaterialType::get_temperature_range(std::string(name), min_temp, max_temp);
    rust::Vec<int64_t> out;
    out.push_back(static_cast<int64_t>(min_temp));
    out.push_back(static_cast<int64_t>(max_temp));
    out.push_back(found ? 1 : 0);
    return out;
}

rust::Vec<int64_t> path_chamber_temperature_range(rust::Str name) {
    int chamber_min_temp = 0;
    int chamber_max_temp = 0;
    const bool found = Slic3r::MaterialType::get_chamber_temperature_range(std::string(name), chamber_min_temp,
                                                                            chamber_max_temp);
    rust::Vec<int64_t> out;
    out.push_back(static_cast<int64_t>(chamber_min_temp));
    out.push_back(static_cast<int64_t>(chamber_max_temp));
    out.push_back(found ? 1 : 0);
    return out;
}

rust::Vec<double> path_adhesion_coefficient(rust::Str name) {
    double adhesion_coefficient = 0.0;
    const bool found = Slic3r::MaterialType::get_adhesion_coefficient(std::string(name), adhesion_coefficient);
    rust::Vec<double> out;
    out.push_back(adhesion_coefficient);
    out.push_back(found ? 1.0 : 0.0);
    return out;
}

rust::Vec<double> path_yield_strength(rust::Str name) {
    double yield_strength = 0.0;
    const bool found = Slic3r::MaterialType::get_yield_strength(std::string(name), yield_strength);
    rust::Vec<double> out;
    out.push_back(yield_strength);
    out.push_back(found ? 1.0 : 0.0);
    return out;
}

rust::Vec<double> path_thermal_length(rust::Str name) {
    double thermal_length = 0.0;
    const bool found = Slic3r::MaterialType::get_thermal_length(std::string(name), thermal_length);
    rust::Vec<double> out;
    out.push_back(thermal_length);
    out.push_back(found ? 1.0 : 0.0);
    return out;
}

} // namespace material_type
} // namespace proof_units
} // namespace slic3r_ffi
