// Proof path 3 (R19, plan step 18, task 31-proof-harness): `Config.cpp` has `static` free
// functions (internal linkage) this crate can only reach by `#include`ing the vendored file
// whole, exactly as `clipper_z_utils.cpp`'s own header comment states for the same mechanism --
// see this task's pull request body for the general rule this file's own history is an instance
// of, and the measurement (461 strong defined external `Slic3r::` symbols in this file's own
// object, per the `nm -m` count `clipper_z_utils.cpp` prescribes) that preceded writing it.
//
// ONE GENERAL FINDING THIS FILE'S OWN NAME PRODUCES, RECORDED HERE BECAUSE IT IS A RULE ABOUT THE
// SHIM LAYOUT ITSELF, NOT JUST THIS UNIT'S HISTORY: `docs/PORTING.md`'s naming rule makes this
// shim's own file stem the unit's manifest module stem, verbatim -- "config" for this unit. On a
// case-insensitive filesystem (the default for macOS APFS, which this pin's own CI and every
// local worktree run on), a quote-include (`#include "Config.cpp"`) written inside a file named
// `config.cpp` resolves *first* against the including file's own directory before ever reaching
// `-I`/`-isystem` -- and `"Config.cpp"` case-insensitively matches this file's own name. The
// first version of this file hit exactly that: `#include nested too deeply` and clang's own
// `-Wnonportable-include-path`, pointing at `"config.cpp"` on disk, not the vendored file.
// `clipper_z_utils` (underscored, lower-case) never collides with `ClipperZUtils` this way;
// `config`/`Config` is the first unit whose manifest stem and vendored stem are the identical
// word, differing only in case. THE FIX, BELOW: an angle-bracket `#include <Config.cpp>`, which
// skips the including file's own directory entirely and resolves only against the compiler's
// include search path -- which already carries `vendor/OrcaSlicer/src/libslic3r` as a plain `-I`
// entry (`crates/slic3r-ffi/build.rs`). A future unit whose manifest stem collides with its own
// vendored file's stem this same way needs the same fix.

#include "config.h"

#include <Config.cpp>

namespace slic3r_ffi {
namespace proof_units {
namespace config {

bool path3_is_whitespace(uint8_t c) {
    return Slic3r::is_whitespace(static_cast<char>(c));
}
bool path3_is_end_of_line(uint8_t c) {
    return Slic3r::is_end_of_line(static_cast<char>(c));
}
bool path3_is_end_of_gcode_line(uint8_t c) {
    return Slic3r::is_end_of_gcode_line(static_cast<char>(c));
}
bool path3_is_end_of_word(uint8_t c) {
    return Slic3r::is_end_of_word(static_cast<char>(c));
}

rust::String path3_normalize_cli_bool_value(rust::Str value) {
    std::string cpp_value(value);
    return rust::String(Slic3r::normalize_cli_bool_value(cpp_value));
}

rust::String path3_escape_string_cstyle(rust::Str value) {
    std::string cpp_value(value);
    return rust::String(Slic3r::escape_string_cstyle(cpp_value));
}

rust::String path3_escape_ampersand(rust::Str value) {
    std::string cpp_value(value);
    return rust::String(Slic3r::escape_ampersand(cpp_value));
}

ProofUnescapeResult path3_unescape_string_cstyle(rust::Str value) {
    std::string cpp_value(value);
    std::string out;
    bool ok = Slic3r::unescape_string_cstyle(cpp_value, out);
    return ProofUnescapeResult{ok, rust::String(out)};
}

rust::String path3_escape_strings_cstyle(rust::Vec<rust::String> values) {
    std::vector<std::string> cpp_values;
    cpp_values.reserve(values.size());
    for (const rust::String &v : values)
        cpp_values.emplace_back(std::string(v));
    return rust::String(Slic3r::escape_strings_cstyle(cpp_values));
}

ProofUnescapeStringsResult path3_unescape_strings_cstyle(rust::Str value) {
    std::string cpp_value(value);
    std::vector<std::string> out;
    bool ok = Slic3r::unescape_strings_cstyle(cpp_value, out);
    rust::Vec<rust::String> rust_out;
    for (const std::string &v : out)
        rust_out.push_back(rust::String(v));
    return ProofUnescapeStringsResult{ok, std::move(rust_out)};
}

} // namespace config
} // namespace proof_units
} // namespace slic3r_ffi
