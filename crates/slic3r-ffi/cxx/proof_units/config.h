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

} // namespace config
} // namespace proof_units
} // namespace slic3r_ffi
