// Proof-path-1 shim for the `Technologies.hpp` unit (R19), compiled onto
// `crates/slic3r-ffi`'s shared `cxx_build::Build` by this crate's `build.rs` -- the one
// sanctioned route a ported unit reaches the oracle by (`docs/PORTING.md`, "Decisions recorded
// on 2026-11-25"). `crates/slic3r-technologies`'s proof calls this through
// `slic3r_proof::proof_units::technologies`.
//
// `#include`s the vendored header verbatim and unmodified. `Technologies.hpp` includes nothing
// (eighteen `#define`s between an include guard), so this translation unit needs no include path
// beyond `src/libslic3r` itself.
//
// The eighteen flags are returned in the header's own declaration order, which is the order
// `crates/slic3r-technologies/src/technologies.rs`'s proof lists them in and the order its
// `// PORT SRC:` line numbers ascend in. The vector's own length is what the proof compares its
// flag count against, so a flag added to or removed from the header on a future pin makes the
// proof fail loudly instead of silently comparing a short prefix.
#include "technologies.h"

#include "Technologies.hpp"

namespace slic3r_ffi {
namespace proof_units {
namespace technologies {

rust::Vec<int32_t> path1_flags() {
    const int32_t values[] = {
        ENABLE_SHOW_CAMERA_TARGET ? 1 : 0,
        ENABLE_SELECTION_DEBUG_OUTPUT ? 1 : 0,
        ENABLE_RENDER_SELECTION_CENTER ? 1 : 0,
        ENABLE_CAMERA_STATISTICS ? 1 : 0,
        ENABLE_THUMBNAIL_GENERATOR_DEBUG ? 1 : 0,
        DISABLE_INSTANCES_SYNCH ? 1 : 0,
        ENABLE_NONCUSTOM_DATA_VIEW_RENDERING ? 1 : 0,
        ENABLE_PROJECT_DIRTY_STATE_DEBUG_WINDOW ? 1 : 0,
        ENABLE_ENVIRONMENT_MAP ? 1 : 0,
        ENABLE_FIXED_SCREEN_SIZE_POINT_MARKERS ? 1 : 0,
        ENABLE_IMGUI_STYLE_EDITOR ? 1 : 0,
        ENABLE_RELOAD_FROM_DISK_REWORK ? 1 : 0,
        ENABLE_MODIFIERS_ALWAYS_TRANSPARENT ? 1 : 0,
        ENABLE_ENHANCED_IMGUI_SLIDER_FLOAT ? 1 : 0,
        ENABLE_ENHANCED_PRINT_VOLUME_FIT ? 1 : 0,
        ENABLE_RAYCAST_PICKING_DEBUG ? 1 : 0,
        ENABLE_NEW_GCODE_VIEWER_DEBUG ? 1 : 0,
        ENABLE_ACTUAL_SPEED_DEBUG ? 1 : 0,
    };
    rust::Vec<int32_t> out;
    for (const int32_t value : values) {
        out.push_back(value);
    }
    return out;
}

} // namespace technologies
} // namespace proof_units
} // namespace slic3r_ffi
