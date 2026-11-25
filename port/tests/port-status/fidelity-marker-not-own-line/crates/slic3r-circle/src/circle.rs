// Fixture: a FIDELITY(port) marker written as a trailing comment after code, not on its
// own line -- task 23-perf-backlog's well-formedness rule.

fn ordered_lookup() { } // FIDELITY(port): std::map's key order matters here; the port uses an insertion-ordered map.

// PORT STATUS
// source: src/libslic3r/Geometry/Circle.cpp
// confidence: high
// tier: exact
// oracle: built
