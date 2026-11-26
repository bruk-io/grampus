// An accepted external marker in the file does not exempt the file, the module or any other
// line: the rule still fires on an unmarked literal below.
// PORT SRC: external: ISO/IEC 9899:2018 7.21.6.1 p8 -- %g's fixed/scientific threshold
const PERCENT_G_SCIENTIFIC_EXPONENT_FLOOR: i32 = -4;

const RESOLUTION: f64 = 0.0125;

// PORT STATUS
// source: src/libslic3r/Geometry/Circle.cpp
// confidence: high
// tier: exact
// oracle: built
