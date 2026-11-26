// Proof shim for the `InstanceLock.cpp`/`InstanceLock.hpp` unit (R19), the
// throw/non-const-ref-out-param leg of task `35-trial-run`. Compiled onto
// `crates/slic3r-ffi`'s shared `cxx_build::Build` by this crate's `build.rs` -- the one
// sanctioned route a ported unit reaches the oracle by (`docs/PORTING.md`, "Decisions recorded
// on 2026-11-25"). `crates/slic3r-instancelock`'s proof calls this through
// `slic3r_proof::proof_units::instance_lock`.
//
// **What this shim can and cannot reach.** `InstanceLock::open_lock_file`/`defer`
// (`InstanceLock.hpp:52-53`) are `private:` statics: their symbols are exported from the
// archive (confirmed with `nm -gjU vendor/OrcaSlicer/build/arm64/src/libslic3r/Release/
// liblibslic3r.a | grep -i instancelock`), but C++ access control still blocks a shim outside
// the class from calling them, and redeclaring the mangled symbol to dodge that would be
// exactly the workaround the rejection rule forbids (`docs/WORKFLOW.md`). This shim therefore
// drives the unit through its public surface only -- the constructor, the destructor and
// `locked()` -- and exercises the out-params and the caught `std::system_error` throw
// transitively: `open_lock_file` is reached on every guard construction that attempts the
// file, and `NativeFileLock`'s own throw is the error that path 1's `try_lock`/open sequence
// could surface if the file could not be opened or locked, caught inside `open_lock_file`
// before it ever reaches here.
#include "instance_lock.h"

#include "InstanceLock.hpp"

namespace slic3r_ffi {
namespace proof_units {
namespace instance_lock {

rust::Vec<int64_t> path1_observations(rust::Str path) {
    const std::string p(path);
    rust::Vec<int64_t> obs;

    {
        Slic3r::InstanceLock empty_guard("");
        obs.push_back(empty_guard.locked() ? 1 : 0);
    }

    {
        Slic3r::InstanceLock outer(p);
        obs.push_back(outer.locked() ? 1 : 0);
        {
            Slic3r::InstanceLock inner(p);
            obs.push_back(inner.locked() ? 1 : 0);
        }
        obs.push_back(outer.locked() ? 1 : 0);
    }

    {
        Slic3r::InstanceLock again(p);
        obs.push_back(again.locked() ? 1 : 0);
    }

    return obs;
}

} // namespace instance_lock
} // namespace proof_units
} // namespace slic3r_ffi
