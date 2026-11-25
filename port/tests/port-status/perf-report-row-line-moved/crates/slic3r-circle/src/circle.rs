// Fixture: a well-formed PERF(port) marker whose committed perf-backlog.md row names a
// stale line number (code was inserted above it) but the same source and reason -- this must
// still pass: the per-marker assertion is keyed on (module, source, reason), never on line.

fn padding_one() {}
fn padding_two() {}
fn padding_three() {}

fn slow_but_faithful(points: &[f64]) -> f64 {
    points.iter().sum()
}

// PERF(port): walks the list twice; kept as-is to stay byte-for-byte faithful to the oracle.

// PORT STATUS
// source: src/libslic3r/Geometry/Circle.cpp
// confidence: high
// tier: exact
// oracle: built
