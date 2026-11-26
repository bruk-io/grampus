// Fixture for the type-annotation half of the differential-comparison rule (docs/PORTING.md,
// "Decisions recorded on 2026-11-26"): this row's proof is a genuine differential, so the row
// must derive "ported" and the marker must raise no finding -- even though both of its `let`
// bindings carry a type annotation naming the same type as the value they compare.

pub struct Mask;

impl Mask {
    pub const ALL: Mask = Mask;
}

// PORT STATUS
// source: src/libslic3r/Geometry.hpp
// confidence: high
// tier: exact
// oracle: built
