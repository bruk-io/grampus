// Fixture: a PERF(port) marker with no reason text after the colon -- task
// 23-perf-backlog's well-formedness rule.

fn identity(x: f64) -> f64 {
    x
}

// PERF(port):

// PORT STATUS
// source: src/libslic3r/Geometry/Circle.cpp
// confidence: high
// tier: exact
// oracle: built
