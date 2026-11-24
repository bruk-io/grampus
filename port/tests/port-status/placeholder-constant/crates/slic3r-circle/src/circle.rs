// PORT SRC: src/libslic3r/libslic3r.h:57
const RESOLUTION: f64 = 0.0125;

// A placeholder constant whose digits happen to appear as a substring of the cited vendored
// line's real constant (libslic3r.h:86 writes "RESOLUTION = 0.0125", not "12"): must still
// fail, with a boundary on both sides of the search, not pass because "12" is a substring of
// "0.0125".
// PORT SRC: src/libslic3r/libslic3r.h:86
const SEGMENTS: usize = 12;

// PORT STATUS
// source: src/libslic3r/Geometry/Circle.cpp
// confidence: high
// tier: exact
// oracle: built
