// An external marker parked beside a vendored marker must not make the vendored marker's own
// failures go away: the cited line below does not exist at the pin, which is R16's own
// placeholder-constant case, and the external marker does not suppress it.
// PORT SRC: src/libslic3r/NoSuchFile.hpp:99999
// PORT SRC: external: ISO/IEC 9899:2018 7.21.6.1 p8 -- %g's fixed/scientific threshold
const PERCENT_G_SCIENTIFIC_EXPONENT_FLOOR: i32 = -4;

// PORT STATUS
// source: src/libslic3r/Geometry/Circle.cpp
// confidence: high
// tier: exact
// oracle: built
