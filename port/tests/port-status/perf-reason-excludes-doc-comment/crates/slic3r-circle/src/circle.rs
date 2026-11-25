// Fixture: a PERF(port) marker immediately followed by an unrelated /// doc comment --
// task 23-perf-backlog's reason extraction must stop before it, not absorb it into the
// derived row's reason, since a later task that edits only the doc comment must not redden
// a report it is forbidden to touch.

// PERF(port): clones the whole vector; kept to mirror the oracle.
/// Returns the sum of the points.
fn slow_but_faithful(points: &[f64]) -> f64 {
    points.iter().sum()
}

// PORT STATUS
// source: src/libslic3r/Geometry/Circle.cpp
// confidence: high
// tier: exact
// oracle: built
