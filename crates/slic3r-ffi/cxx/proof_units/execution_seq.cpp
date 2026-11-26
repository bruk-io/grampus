// Proof-path-2 shim for the `Execution/ExecutionSeq.hpp` unit (R19, task `56-foundation-point`),
// compiled onto `crates/slic3r-ffi`'s shared `cxx_build::Build` by this crate's `build.rs` -- the
// one sanctioned route a ported unit reaches the oracle by (`docs/PORTING.md`, "Decisions
// recorded on 2026-11-25"). This unit's manifest row carries `proof path: 2`, but no committed
// stage dump carries a `posContouring` seam at all (`port/dumps/*/` holds `posSlice` through
// `posSimplifyPath`, `psSkirtBrim` and `psWipeTower` only), so this shim follows the same
// generalised route `cxx/proof_units/execution.cpp` and `object_id.cpp`'s
// `path2_header_observations` already use for a `proof path: 2` header-only row: it drives the
// *real* vendored dispatch directly, through the real `Slic3r::ex_seq` global this header defines
// (`ExecutionSeq.hpp:20`) -- not a proof-local stand-in policy, because a real one exists right
// here, unlike `Execution.hpp`'s own proof, which had none.
#include "execution_seq.h"

#include "libslic3r/Execution/ExecutionSeq.hpp"

#include <vector>

namespace slic3r_ffi {
namespace proof_units {
namespace execution_seq {

int64_t path2_reduce_triples_and_sums(const rust::Vec<int64_t> &values) {
    const std::vector<int64_t> v(values.begin(), values.end());
    // `Slic3r::execution::reduce` (`Execution.hpp:55-73`) dispatching through
    // `Traits<ExecutionSeq>::reduce` (`ExecutionSeq.hpp:67-80`), the real vendored specialisation
    // this unit ports.
    return Slic3r::execution::reduce(
        Slic3r::ex_seq, v.begin(), v.end(), int64_t(0),
        [](const int64_t &a, const int64_t &b) { return a + b; },
        [](const int64_t &x) { return x * 3; });
}

} // namespace execution_seq
} // namespace proof_units
} // namespace slic3r_ffi
