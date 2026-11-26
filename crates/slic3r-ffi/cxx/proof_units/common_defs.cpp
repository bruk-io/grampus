// Proof shim for the `CommonDefs.hpp` unit (R19, proof path 2, task 52-foundation-common-defs,
// `port/foundation.tsv` row `foundation-01`), compiled onto `crates/slic3r-ffi`'s shared
// `cxx_build::Build` by this crate's `build.rs` -- the one sanctioned route a ported unit
// reaches the oracle by (`docs/PORTING.md`, "Decisions recorded on 2026-11-25").
// `crates/slic3r-commondefs`'s proof calls this through
// `slic3r_proof::proof_units::common_defs`.
//
// `CommonDefs.hpp` holds one plain (unscoped) C++ `enum`, `Slic3r::NozzleType`, with no
// behaviour of its own beyond its enumerators' discriminant values. There is no dump-corpus
// field for it (`port/manifest.tsv`'s `proof path` cell was seeded from this row's pipeline
// stage, not a verified dump field -- see this unit's module doc comment for the grep that
// checked every committed dump), so this shim drives the vendored header's own compiled
// enumerators directly -- the same answer the `ObjectID.hpp` row's `path2_header_observations`
// gives to "a header row the dump corpus doesn't carry" (`docs/PORTING.md`, decision 4).
#include "common_defs.h"

#include "CommonDefs.hpp"

namespace slic3r_ffi {
namespace proof_units {
namespace common_defs {

rust::Vec<int64_t> path2_nozzle_type_discriminants() {
    const int64_t values[] = {
        static_cast<int64_t>(Slic3r::ntUndefine),
        static_cast<int64_t>(Slic3r::ntHardenedSteel),
        static_cast<int64_t>(Slic3r::ntStainlessSteel),
        static_cast<int64_t>(Slic3r::ntTungstenCarbide),
        static_cast<int64_t>(Slic3r::ntBrass),
        static_cast<int64_t>(Slic3r::ntE3D),
        static_cast<int64_t>(Slic3r::ntCount),
    };
    rust::Vec<int64_t> out;
    for (const int64_t value : values) {
        out.push_back(value);
    }
    return out;
}

} // namespace common_defs
} // namespace proof_units
} // namespace slic3r_ffi
