// Fixture: a unit with nothing executable to prove of its own -- declarations and a type alias
// the enclosing unit's own exported surface exercises. Its row's `proof path` cell reads
// `enclosing: Geometry.hpp`, so it carries no `// PROOF:` marker and derives "ported" exactly
// when `Geometry.hpp`'s row does (R19's own fallback).

pub type Helper = i64;

// PORT STATUS
// source: src/libslic3r/Geometry/Circle.hpp
// confidence: high
// tier: exact
// oracle: built
