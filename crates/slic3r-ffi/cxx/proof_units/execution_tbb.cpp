// Proof-path-2 shim for the `Execution/ExecutionTBB.hpp` unit (R19, task `56-foundation-point`),
// compiled onto `crates/slic3r-ffi`'s shared `cxx_build::Build` by this crate's `build.rs` -- the
// one sanctioned route a ported unit reaches the oracle by (`docs/PORTING.md`, "Decisions
// recorded on 2026-11-25"). Same finding as `execution_seq.cpp`'s own comment: no committed stage
// dump carries a `posContouring` seam, so this shim drives the *real* vendored dispatch directly,
// through the real `Slic3r::ex_tbb` global this header defines (`ExecutionTBB.hpp:22`) -- the
// genuine TBB-backed specialisation, which really does run `tbb::parallel_reduce` on this side of
// the comparison (see `crates/slic3r-execution/src/execution_tbb.rs`'s own module doc for why the
// Rust port does not).
#include "execution_tbb.h"

#include "libslic3r/Execution/ExecutionTBB.hpp"

#include <vector>

namespace slic3r_ffi {
namespace proof_units {
namespace execution_tbb {

int64_t path2_reduce_increments_and_sums(const rust::Vec<int64_t> &values) {
    const std::vector<int64_t> v(values.begin(), values.end());
    // `Slic3r::execution::reduce` (`Execution.hpp:55-73`) dispatching through
    // `Traits<ExecutionTBB>::reduce` (`ExecutionTBB.hpp:53-71`), which really runs
    // `tbb::parallel_reduce` -- the real vendored specialisation this unit ports.
    return Slic3r::execution::reduce(
        Slic3r::ex_tbb, v.begin(), v.end(), int64_t(0),
        [](const int64_t &a, const int64_t &b) { return a + b; },
        [](const int64_t &x) { return x + 1; }, 2);
}

} // namespace execution_tbb
} // namespace proof_units
} // namespace slic3r_ffi
