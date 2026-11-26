// One external marker names one origin and has no cited line to match a literal against, so it
// may not legitimise two or three literals at once -- that would be a per-line exemption in all
// but name. The externally defined constant goes on a line of its own instead.
// PORT SRC: external: ISO/IEC 9899:2018 7.21.6.1 p8 -- %g's fixed/scientific threshold
const PAIR: (i32, f64) = (-4, 0.0125);

// PORT STATUS
// source: src/libslic3r/Geometry/Circle.cpp
// confidence: high
// tier: exact
// oracle: built
