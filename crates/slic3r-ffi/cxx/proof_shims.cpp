// Proof path 1 (R19, plan step 18, task 31-proof-harness): a direct call to an externally linked
// symbol in the archive `just oracle-build` produces, through `crates/slic3r-ffi/build.rs`'s
// link line (derived from the CMake build tree, not enumerated).
//
// `Slic3r::wipe_tower_object_id()` (src/libslic3r/ObjectID.cpp) is deterministic here: it
// constructs the first `Slic3r::ObjectBase` this whole process ever makes --
// `ObjectID.cpp`'s file-static `s_last_id` starts at 0, and nothing else linked into this
// crate's test binary touches `Slic3r::ObjectBase` -- so it always returns id 1.
// `crates/slic3r-proof/tests/harness_paths.rs` asserts exactly that, through `tiers::compare_unit`
// at `port/manifest.tsv`'s `ObjectID.cpp` row (equality tier `exact`, proof path `1`).

#include "proof_shims.h"

#include "ObjectID.hpp"

namespace slic3r_ffi {
namespace proof {

uint64_t path1_wipe_tower_object_id() {
    return static_cast<uint64_t>(Slic3r::wipe_tower_object_id().id);
}

} // namespace proof
} // namespace slic3r_ffi
