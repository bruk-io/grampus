// Fixture: a well-formed PERF(port) marker whose committed perf-backlog.md row names a
// different reason -- the "a row whose marker was ... edited fails" case task 23-perf-backlog's
// Done-when states.

fn slow_but_faithful(points: &[f64]) -> f64 {
    points.iter().sum()
}

// PERF(port): walks the list twice; kept as-is to stay byte-for-byte faithful to the oracle.

// PORT STATUS
// source: src/libslic3r/Geometry/Circle.cpp
// confidence: high
// tier: exact
// oracle: built
