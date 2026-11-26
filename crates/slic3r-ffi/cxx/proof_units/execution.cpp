// Proof-path-2 shim for the `Execution/Execution.hpp` unit (R19), compiled onto
// `crates/slic3r-ffi`'s shared `cxx_build::Build` by this crate's `build.rs` -- the one
// sanctioned route a ported unit reaches the oracle by (`docs/PORTING.md`, "Decisions recorded
// on 2026-11-25"). This unit's manifest row carries `proof path: 2`, but `Execution.hpp` is a
// pure dispatch layer with no pipeline stage of its own: it forwards to `Traits<EP>`, which only
// a concrete policy (`ExecutionSeq.hpp`/`ExecutionTBB.hpp`, a sibling foundation task's own
// units) specialises, so no in-scope instantiation's output reaches any committed stage dump
// (see `crates/slic3r-execution/src/execution.rs`'s module doc). Decision 1's shim mechanism is
// not limited to proof paths 1 and 3 -- `cxx/proof_units/object_id.cpp`'s
// `path2_header_observations` already proves a `proof path: 2` row this same way -- so this shim
// does the same for `Execution.hpp`: it specialises `Slic3r::IsExecutionPolicy_` and
// `Slic3r::execution::Traits` for a minimal, real, serial proof-local policy
// (`ProofSerialEP`, standing in for a real concrete policy the same way `cxx/proof_units/
// enum_bitmask.cpp`'s proof-local enum stands in for a real one -- the template under test is
// generic in `EP`, not in which policy parameterises it), and calls the vendored
// `Slic3r::execution::reduce` free function through it -- the actual dispatch code this unit
// ports, not a reimplementation of it.
#include "execution.h"

#include "libslic3r/Execution/Execution.hpp"

#include <vector>

namespace Slic3r {

// A minimal, real execution policy (not mocked away): `IsExecutionPolicy_<ProofSerialEP>`
// specialised to `true_type` is exactly how a concrete policy opts in
// (`Execution.hpp:14`), and `execution::Traits<ProofSerialEP>` below gives the two operations
// `Execution.hpp`'s free functions forward to, run serially and single-threaded -- a real
// property of this proof policy, not a substitute for `Execution.hpp`'s own logic, which is pure
// dispatch and holds none of a loop's behaviour itself (the same point
// `crates/slic3r-execution/src/execution.rs`'s own `OneThread` test policy makes on the Rust
// side).
struct ProofSerialEP {};

template <>
struct IsExecutionPolicy_<ProofSerialEP> : public std::true_type {};

namespace execution {

template <>
struct Traits<ProofSerialEP> {
    using SpinningMutex = int;
    using BlockingMutex = int;

    static size_t max_concurrency(const ProofSerialEP &) { return 1; }

    template <class I, class MergeFn, class T, class AccessFn>
    static T reduce(const ProofSerialEP &, I from, I to, const T &init, MergeFn &&mergefn,
                     AccessFn &&accessfn, size_t /*granularity*/) {
        T acc = init;
        for (; from != to; ++from) {
            acc = mergefn(acc, accessfn(*from));
        }
        return acc;
    }
};

} // namespace execution
} // namespace Slic3r

namespace slic3r_ffi {
namespace proof_units {
namespace execution {

int64_t path2_reduce_doubles_and_sums(const rust::Vec<int64_t> &values) {
    const Slic3r::ProofSerialEP ep;
    const std::vector<int64_t> v(values.begin(), values.end());
    // `Slic3r::execution::reduce` (`Execution.hpp:55-73`), the real vendored dispatch function
    // this unit ports, forwarding to `Traits<ProofSerialEP>::reduce` above.
    return Slic3r::execution::reduce(
        ep, v.begin(), v.end(), int64_t(0),
        [](const int64_t &a, const int64_t &b) { return a + b; },
        [](const int64_t &x) { return x * 2; });
}

} // namespace execution
} // namespace proof_units
} // namespace slic3r_ffi
