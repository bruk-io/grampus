// Proof-path-1 shim for `Technologies.hpp` (task 54-foundation-objectid-clonable, R19), compiled
// by this crate's own `build.rs` and linked into `crates/slic3r-technologies`'s `#[cfg(test)]`
// proof -- `proof_present`'s "a `#[cfg(test)]` module inside the module file itself" branch.
//
// `#include`s the vendored header verbatim and unmodified. `Technologies.hpp` includes nothing
// (eighteen `#define`s between an include guard), so this translation unit needs no include path
// beyond `src/libslic3r` itself.
//
// The eighteen flags are written into the caller's buffer in the header's own declaration order,
// which is the order `crates/slic3r-technologies/src/technologies.rs`'s proof lists them in and
// the order its `// PORT SRC:` line numbers ascend in. The count is returned rather than assumed,
// so a flag added to or removed from the header on a future pin makes the proof fail loudly
// instead of silently comparing a short prefix.
#include "Technologies.hpp"

#include <cstdint>

extern "C" {

int32_t slic3r_technologies_shim_flags(int32_t *out, int32_t capacity) {
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
    const int32_t count = static_cast<int32_t>(sizeof(values) / sizeof(values[0]));
    if (out == nullptr || capacity < count) {
        return -count;
    }
    for (int32_t i = 0; i < count; ++i) {
        out[i] = values[i];
    }
    return count;
}

} // extern "C"
