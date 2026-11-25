// Fixture: the words "FIDELITY(port):" appear only inside a /* ... */ block comment and
// inside a string literal -- never on their own comment line and never trailing after code --
// so task 23-perf-backlog's well-formedness check must not raise fidelity-marker-not-on-own-line
// for either, even though the raw FIDELITY_LINE_RE regex (deliberately unmasked, for count
// parity with port/audit-order) matches both lines.

/* A block comment that mentions // FIDELITY(port): nothing real, just prose.
 */
const MENTION: &str = "// FIDELITY(port): inside a string literal";

fn use_mention() -> &'static str {
    MENTION
}

// PORT STATUS
// source: src/libslic3r/Geometry/Circle.cpp
// confidence: high
// tier: exact
// oracle: built
