// Fixture for the differential-comparison rule (docs/PORTING.md, "Decisions recorded on
// 2026-11-25"): everything the derivation's `ported` arm asks for is here except a comparison
// against the oracle, so the row must derive "drafted" and the marker must be a finding.

fn render() -> &'static str {
    "psWipeTower"
}

// PORT STATUS
// source: src/libslic3r/Geometry.hpp
// confidence: high
// tier: exact
// oracle: built
