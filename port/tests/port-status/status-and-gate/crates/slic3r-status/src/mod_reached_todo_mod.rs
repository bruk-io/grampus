// Fixture (22-port-status-gates): a `mod` declaration reaches this file (see lib.rs) but it
// still holds a live `todo!()` -- task.md's bullet ties `drafted` strictly to "no `mod`
// declaration reaches it", so a mod-reached file with a live `todo!()` must derive as
// "not-started", matching `port/inventory`'s own rule, not "drafted".

fn stub() -> i32 {
    todo!()
}

// PORT STATUS
// source: src/libslic3r/Geometry/Circle.hpp
// confidence: low
// tier: exact
// oracle: built
