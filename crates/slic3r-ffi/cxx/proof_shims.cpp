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

// Task 54-foundation-objectid-clonable, R19, proof path 1: the `PrintConfigConstants.hpp` and
// `Technologies.hpp` manifest rows. Appended at the end, changing no line above, so a sibling
// foundation task's append conflicts textually rather than reordering silently (task.md's
// "Conflict discipline").
//
// `ObjectID.cpp`, this task's third `proof path: 1` row, needs no append: its shim is
// `path1_wipe_tower_object_id()` above, which task 31-proof-harness already wrote and which is
// exactly that row's oracle.
//
// **These two are not what that task's own proofs call, and the reason is structural.**
// `proof_present` (`port/port-status`) looks for the `// PROOF:` marker in
// `crates/<target crate>/tests/` or in a `#[cfg(test)]` module inside the target module file --
// i.e. inside `slic3r-printconfigconstants` and `slic3r-technologies`. Reaching a bridge in this
// crate from there needs `crates/slic3r-ffi` to be a direct dependency of those crates (Rust does
// not expose an indirect dependency's items), and no foundation task may add a dependency to any
// `crates/<crate>/Cargo.toml`. So each of those crates compiles its own small shim over the same
// vendored header through its own `build.rs`, which needs no manifest key -- the same answer task
// 53-foundation-int128-queue reached for `Int128.hpp`. These entries are appended because
// task.md's Done-when asks for one shim per proven path-1 unit here; they are declared through
// the `cxx` bridge in `src/proof.rs`, so they are compiled and linked rather than dead, and they
// are available to any future caller that does depend on this crate.
//
// Neither header includes anything: `PrintConfigConstants.hpp` is three `#define`s and
// `Technologies.hpp` eighteen, so neither needs an include path beyond the one
// `crates/slic3r-ffi/build.rs` already sets up for `src/libslic3r`.

#include "PrintConfigConstants.hpp"
#include "Technologies.hpp"

namespace slic3r_ffi {
namespace proof {

double path1_initial_layer_height() {
    const double value = INITIAL_LAYER_HEIGHT;
    return value;
}

int32_t path1_initial_raft_layers() {
    const int value = INITIAL_RAFT_LAYERS;
    return static_cast<int32_t>(value);
}

bool path1_initial_reduce_crossing_wall() {
    const bool value = INITIAL_REDUCE_CROSSING_WALL;
    return value;
}

// The eighteen flags in `Technologies.hpp`'s own declaration order, which is the order
// `crates/slic3r-technologies/src/technologies.rs` lists them in.
rust::Vec<int32_t> path1_technologies_flags() {
    rust::Vec<int32_t> flags;
    flags.push_back(ENABLE_SHOW_CAMERA_TARGET ? 1 : 0);
    flags.push_back(ENABLE_SELECTION_DEBUG_OUTPUT ? 1 : 0);
    flags.push_back(ENABLE_RENDER_SELECTION_CENTER ? 1 : 0);
    flags.push_back(ENABLE_CAMERA_STATISTICS ? 1 : 0);
    flags.push_back(ENABLE_THUMBNAIL_GENERATOR_DEBUG ? 1 : 0);
    flags.push_back(DISABLE_INSTANCES_SYNCH ? 1 : 0);
    flags.push_back(ENABLE_NONCUSTOM_DATA_VIEW_RENDERING ? 1 : 0);
    flags.push_back(ENABLE_PROJECT_DIRTY_STATE_DEBUG_WINDOW ? 1 : 0);
    flags.push_back(ENABLE_ENVIRONMENT_MAP ? 1 : 0);
    flags.push_back(ENABLE_FIXED_SCREEN_SIZE_POINT_MARKERS ? 1 : 0);
    flags.push_back(ENABLE_IMGUI_STYLE_EDITOR ? 1 : 0);
    flags.push_back(ENABLE_RELOAD_FROM_DISK_REWORK ? 1 : 0);
    flags.push_back(ENABLE_MODIFIERS_ALWAYS_TRANSPARENT ? 1 : 0);
    flags.push_back(ENABLE_ENHANCED_IMGUI_SLIDER_FLOAT ? 1 : 0);
    flags.push_back(ENABLE_ENHANCED_PRINT_VOLUME_FIT ? 1 : 0);
    flags.push_back(ENABLE_RAYCAST_PICKING_DEBUG ? 1 : 0);
    flags.push_back(ENABLE_NEW_GCODE_VIEWER_DEBUG ? 1 : 0);
    flags.push_back(ENABLE_ACTUAL_SPEED_DEBUG ? 1 : 0);
    return flags;
}

} // namespace proof
} // namespace slic3r_ffi
