#pragma once

// Declarations only (see cxx/clipper2_shim.h's comment for why this crate always splits the
// cxx-generated header from the hand-written declarations file). `cxx_build::bridges` generates
// this header's counterpart at <crate-name>/src/proof_units/config.rs.h.
#include "slic3r-ffi/src/proof_units/config.rs.h"

namespace slic3r_ffi {
namespace proof_units {
namespace config {

// The four character-class helpers (internal linkage: `static` free functions, Config.cpp:1250-1253).
bool path3_is_whitespace(uint8_t c);
bool path3_is_end_of_line(uint8_t c);
bool path3_is_end_of_gcode_line(uint8_t c);
bool path3_is_end_of_word(uint8_t c);

// `normalize_cli_bool_value` (internal linkage: `static`, Config.cpp:1758-1782).
rust::String path3_normalize_cli_bool_value(rust::Str value);

// The externally-linked string-escaping free functions (Config.cpp:76-259), reached through the
// same `#include` as the internal-linkage helpers above.
rust::String path3_escape_string_cstyle(rust::Str value);
rust::String path3_escape_ampersand(rust::Str value);

// `ProofUnescapeResult`/`ProofUnescapeStringsResult` are `cxx`-generated shared structs (declared
// in the bridge module, `src/proof_units/config.rs`); this header's own include of the
// `cxx`-generated header above already defines them, and this file does not redefine them.
ProofUnescapeResult path3_unescape_string_cstyle(rust::Str value);

rust::String path3_escape_strings_cstyle(rust::Vec<rust::String> values);

ProofUnescapeStringsResult path3_unescape_strings_cstyle(rust::Str value);

// `ConfigOptionFloat::serialize`/`::deserialize` (`Config.hpp:840-854`) -- this unit's own
// primary surface, proving the float serialisation/deserialisation format directly rather than
// only the string-escaping fan-in.
rust::String path3_serialize_float(double value);
ProofDeserializeFloatResult path3_deserialize_float(rust::Str value);
ProofDeserializeIntResult path3_deserialize_int(rust::Str value);

// `std::istringstream >> double` on its own, seeded with a sentinel the caller can recognise.
// `ConfigOptionFloat::deserialize` (`Config.hpp:844-854`) extracts into a *local* `double` and
// assigns the option only when `!iss.fail()`, so `path3_deserialize_float` above cannot observe
// what a failing extraction assigned -- and the underflow (`ERANGE`) cases are exactly the ones
// that fail *with* a value. This exposes the extraction itself, which is what the port's
// `cpp_istream_extract_f64` claims to reproduce, in the same translation unit and locale as the
// vendored code above.
ProofIstreamExtractDoubleResult path3_istream_extract_double(rust::Str value);

// `ConfigOptionPoint::deserialize`/`ConfigOptionPoint3::deserialize` (`Config.hpp:1557-1567`
// and `:1686-1696`), whose `sscanf(" %lf , %lf %c", ...)` is `strtod` and not the `istream`
// extraction above -- a different acceptance grammar, measured rather than assumed.
ProofDeserializePointResult path3_deserialize_point(rust::Str value);
ProofDeserializePoint3Result path3_deserialize_point3(rust::Str value);

} // namespace config
} // namespace proof_units
} // namespace slic3r_ffi
