// The accepted external form (docs/PORTING.md, "Decisions recorded on 2026-11-25"): a constant
// whose origin is outside the vendored tree, so there is no vendored line to cite. The marker
// sits on the contiguous comment run directly above its literal, with no blank line between --
// the same position the vendored form needs.
// PORT SRC: external: ISO/IEC 9899:2018 7.21.6.1 p8 -- %g renders in scientific notation when
// the converted value's exponent is less than -4
const PERCENT_G_SCIENTIFIC_EXPONENT_FLOOR: i32 = -4;

// The same form written as a trailing comment on the literal's own line, which the vendored form
// also accepts.
const QUANTIZE_DIGITS: usize = 6; // PORT SRC: external: C17 7.21.6.1 p8 -- the default precision of %g

// Deliberately the one spelling whose text WOULD parse as a vendored "<path>:<line>" pair if the
// external form were not matched first: no space after "external:", and a digits-colon-digits
// reference right behind it. Read as external, so no lookup is attempted and the marker is
// accepted; this line is the end-to-end proof of that ordering.
// PORT SRC: external:9899:2018 7.21.6.1 p8 -- the same clause, spelled with no space
const PERCENT_G_DEFAULT_PRECISION: usize = 6;

// PORT STATUS
// source: src/libslic3r/Geometry/Circle.cpp
// confidence: high
// tier: exact
// oracle: built
