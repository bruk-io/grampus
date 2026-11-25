//! Port of `Technologies.hpp` (`vendor/OrcaSlicer/src/libslic3r/Technologies.hpp`): a set of
//! `#define ENABLE_*`/`DISABLE_*` feature flags, each `0` or `1`, that upstream's own
//! preprocessor reads with `#if`. Every flag transcribed here is in fact read only from
//! `src/slic3r/GUI/*` (debug rendering toggles, selection debug output, the GCode viewer, the
//! ImGui style editor, ...), which R10 puts out of scope; `AppConfig.cpp`'s own include of this
//! header reads none of them directly. The header's own manifest row is in-scope regardless
//! (`template-heavy: yes`), so every flag is ported faithfully as a `bool` constant -- see this
//! task's pull request body for this observation.
//!
//! `0`/`1` are both in the numeric-literal rule's exempt set, so no `// PORT SRC:` marker is
//! required for any of these eighteen constants, but one is recorded for each anyway (the
//! Done-when line's own wording: "every numeric constant transcribed ... carries" one).

/// `ENABLE_SHOW_CAMERA_TARGET` (`Technologies.hpp:8`).
// PORT SRC: src/libslic3r/Technologies.hpp:8
pub const ENABLE_SHOW_CAMERA_TARGET: bool = false;

/// `ENABLE_SELECTION_DEBUG_OUTPUT` (`:10`).
// PORT SRC: src/libslic3r/Technologies.hpp:10
pub const ENABLE_SELECTION_DEBUG_OUTPUT: bool = false;

/// `ENABLE_RENDER_SELECTION_CENTER` (`:12`).
// PORT SRC: src/libslic3r/Technologies.hpp:12
pub const ENABLE_RENDER_SELECTION_CENTER: bool = false;

/// `ENABLE_CAMERA_STATISTICS` (`:14`).
// PORT SRC: src/libslic3r/Technologies.hpp:14
pub const ENABLE_CAMERA_STATISTICS: bool = false;

/// `ENABLE_THUMBNAIL_GENERATOR_DEBUG` (`:16`).
// PORT SRC: src/libslic3r/Technologies.hpp:16
pub const ENABLE_THUMBNAIL_GENERATOR_DEBUG: bool = false;

/// `DISABLE_INSTANCES_SYNCH` (`:18`).
// PORT SRC: src/libslic3r/Technologies.hpp:18
pub const DISABLE_INSTANCES_SYNCH: bool = false;

/// `ENABLE_NONCUSTOM_DATA_VIEW_RENDERING` (`:20`).
// PORT SRC: src/libslic3r/Technologies.hpp:20
pub const ENABLE_NONCUSTOM_DATA_VIEW_RENDERING: bool = false;

/// `ENABLE_PROJECT_DIRTY_STATE_DEBUG_WINDOW` (`:22`).
// PORT SRC: src/libslic3r/Technologies.hpp:22
pub const ENABLE_PROJECT_DIRTY_STATE_DEBUG_WINDOW: bool = false;

/// `ENABLE_ENVIRONMENT_MAP` (`:26`).
// PORT SRC: src/libslic3r/Technologies.hpp:26
pub const ENABLE_ENVIRONMENT_MAP: bool = false;

/// `ENABLE_FIXED_SCREEN_SIZE_POINT_MARKERS` (`:29`).
// PORT SRC: src/libslic3r/Technologies.hpp:29
pub const ENABLE_FIXED_SCREEN_SIZE_POINT_MARKERS: bool = true;

/// `ENABLE_IMGUI_STYLE_EDITOR` (`:32`).
// PORT SRC: src/libslic3r/Technologies.hpp:32
pub const ENABLE_IMGUI_STYLE_EDITOR: bool = false;

/// `ENABLE_RELOAD_FROM_DISK_REWORK` (`:35`).
// PORT SRC: src/libslic3r/Technologies.hpp:35
pub const ENABLE_RELOAD_FROM_DISK_REWORK: bool = true;

/// `ENABLE_MODIFIERS_ALWAYS_TRANSPARENT` (`:38`).
// PORT SRC: src/libslic3r/Technologies.hpp:38
pub const ENABLE_MODIFIERS_ALWAYS_TRANSPARENT: bool = true;

/// `ENABLE_ENHANCED_IMGUI_SLIDER_FLOAT` (`:43`).
// PORT SRC: src/libslic3r/Technologies.hpp:43
pub const ENABLE_ENHANCED_IMGUI_SLIDER_FLOAT: bool = true;

/// `ENABLE_ENHANCED_PRINT_VOLUME_FIT` (`:45`).
// PORT SRC: src/libslic3r/Technologies.hpp:45
pub const ENABLE_ENHANCED_PRINT_VOLUME_FIT: bool = true;

/// `ENABLE_RAYCAST_PICKING_DEBUG` (`:47`).
// PORT SRC: src/libslic3r/Technologies.hpp:47
pub const ENABLE_RAYCAST_PICKING_DEBUG: bool = false;

/// `ENABLE_NEW_GCODE_VIEWER_DEBUG` (`:50`).
// PORT SRC: src/libslic3r/Technologies.hpp:50
pub const ENABLE_NEW_GCODE_VIEWER_DEBUG: bool = false;

/// `ENABLE_ACTUAL_SPEED_DEBUG` (`:52`).
// PORT SRC: src/libslic3r/Technologies.hpp:52
pub const ENABLE_ACTUAL_SPEED_DEBUG: bool = true;

// PORT STATUS
// source: src/libslic3r/Technologies.hpp
// confidence: high
// tier: exact
// oracle: built
