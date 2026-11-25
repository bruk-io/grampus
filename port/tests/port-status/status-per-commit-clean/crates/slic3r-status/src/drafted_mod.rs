// Fixture (22-port-status-gates): a PORT STATUS trailer naming this row's own source path, and
// no `mod` declaration reaching it (see lib.rs) -- expected to derive as "drafted", and to be
// counted as a "module gated out of the build" in the burn-down.

fn placeholder() -> i32 {
    0
}

// PORT STATUS
// source: src/libslic3r/Geometry/Circle.cpp
// confidence: high
// tier: exact
// oracle: built
