// Proof-path-2 shim for the `MTUtils.hpp` unit (R19, task `56-foundation-point`), compiled onto
// `crates/slic3r-ffi`'s shared `cxx_build::Build` by this crate's `build.rs` -- the one
// sanctioned route a ported unit reaches the oracle by (`docs/PORTING.md`, "Decisions recorded
// on 2026-11-25"). This unit's manifest row carries `proof path: 2`, but no field under
// `port/dumps/*/posPrepareInfill.dump` corresponds to `MTUtils.hpp`'s own output -- it is a
// generic, data-independent numeric-sequence header, not a pipeline stage
// (`crates/slic3r-mtutils/src/mt_utils.rs`'s own module doc has the full finding). This shim
// follows the same generalised route `execution.cpp`/`object_id.cpp`'s
// `path2_header_observations` already use: it calls the *real* vendored `Slic3r::grid` function
// directly, over `double` -- the one in-scope floating width this header's own callers use.
#include "mt_utils.h"

#include "MTUtils.hpp"

namespace slic3r_ffi {
namespace proof_units {
namespace mt_utils {

rust::Vec<double> path2_grid(double start, double stop, double stride) {
    // `Slic3r::grid<double>` (`MTUtils.hpp:126-139`), the real vendored function this unit ports.
    const std::vector<double> values = Slic3r::grid<double>(start, stop, stride);
    rust::Vec<double> out;
    for (const double v : values) {
        out.push_back(v);
    }
    return out;
}

} // namespace mt_utils
} // namespace proof_units
} // namespace slic3r_ffi
