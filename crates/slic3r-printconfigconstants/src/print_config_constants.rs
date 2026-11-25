//! Port of `PrintConfigConstants.hpp`
//! (`vendor/OrcaSlicer/src/libslic3r/PrintConfigConstants.hpp`): three `#define`s the upstream
//! header itself says are "here just for unit testing" (`:3`). Each macro's call site
//! (`PrintConfig.cpp`) pins its type: `ConfigOptionFloat` wraps `double`, `ConfigOptionInt`
//! wraps `int`, `ConfigOptionBool` wraps `bool` (`Config.hpp:824,1018,1907`) -- the type map's
//! `double -> f64` and the platform's 32-bit `int -> i32`.

/// `INITIAL_LAYER_HEIGHT` (`PrintConfigConstants.hpp:5`, `#define INITIAL_LAYER_HEIGHT 0.2`).
// PORT SRC: src/libslic3r/PrintConfigConstants.hpp:5
pub const INITIAL_LAYER_HEIGHT: f64 = 0.2;

/// `INITIAL_RAFT_LAYERS` (`:6`, `#define INITIAL_RAFT_LAYERS 0`). `0` is in the numeric-literal
/// rule's exempt set, so no `// PORT SRC:` marker is required for it, but one is recorded anyway
/// (the Done-when line's own wording: "every numeric constant transcribed ... carries" one).
// PORT SRC: src/libslic3r/PrintConfigConstants.hpp:6
pub const INITIAL_RAFT_LAYERS: i32 = 0;

/// `INITIAL_REDUCE_CROSSING_WALL` (`:7`, `#define INITIAL_REDUCE_CROSSING_WALL false`). Not a
/// numeric literal, so the marker rule never applies to it; cited here for completeness.
pub const INITIAL_REDUCE_CROSSING_WALL: bool = false;

// PORT STATUS
// source: src/libslic3r/PrintConfigConstants.hpp
// confidence: high
// tier: exact
// oracle: built
