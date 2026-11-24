// Fixture: this module once had two PERF(port) markers that happened to share one
// (source, reason), one of which was deleted. A committed row-to-marker check that matches
// by existence, not count, lets the surviving marker satisfy both committed rows and must
// instead fail on the excess row -- task 23-perf-backlog's "a row whose marker was deleted
// fails" rule.

fn slow_but_faithful(points: &[f64]) -> f64 {
    points.iter().sum()
}

// PERF(port): recomputes the index each call instead of caching it.

// PORT STATUS
// source: src/libslic3r/Geometry/Circle.cpp
// confidence: high
// tier: exact
// oracle: built
