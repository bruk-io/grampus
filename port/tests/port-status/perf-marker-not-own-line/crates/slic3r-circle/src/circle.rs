// Fixture: a PERF(port) marker written as a trailing comment after code, not on its own
// line -- task 23-perf-backlog's well-formedness rule.

fn slow_but_faithful(points: &[f64]) -> f64 { points.iter().sum() } // PERF(port): walks the list twice; left as-is to stay byte-for-byte faithful.

// PORT STATUS
// source: src/libslic3r/Geometry/Circle.cpp
// confidence: high
// tier: exact
// oracle: built
