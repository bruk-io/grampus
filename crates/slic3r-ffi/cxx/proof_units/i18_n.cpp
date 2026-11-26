// Proof shim for the `I18N.hpp` unit (R19, proof path 2, task 52-foundation-common-defs,
// `port/foundation.tsv` row `foundation-01`), compiled onto `crates/slic3r-ffi`'s shared
// `cxx_build::Build` by this crate's `build.rs` -- the one sanctioned route a ported unit
// reaches the oracle by (`docs/PORTING.md`, "Decisions recorded on 2026-11-25").
// `crates/slic3r-i18n`'s proof calls this through `slic3r_proof::proof_units::i18_n`.
//
// No field in the committed stage-dump corpus carries a translated string or any other
// I18N-derived value (`crates/slic3r-commondefs/src/common_defs.rs`'s doc comment states the
// grep that checked this for the same `posSlice`-seeded "2" cell), so this shim drives the
// vendored header's own inline code directly -- decision 4's sanctioned answer for a header row
// the dump corpus doesn't carry, the same one `ObjectID.hpp`'s `path2_header_observations` is
// proven by.
//
// **`path2_translate` exercises the default path (`translate_fn == nullptr`).**
// `Slic3r::I18N::translate_fn` is a process-global, externally linked `static` (`I18N.hpp:16`,
// defined at `utils.cpp:358` as `nullptr`), shared with every other translation unit the link
// extracts -- the same hazard `object_id.cpp`'s own comment documents at length for
// `ObjectBase::s_last_id`. `path2_translate` never calls `Slic3r::I18N::set_translate_callback`,
// so it never mutates that global; the default-`nullptr` identity path is exactly what
// `I18N.hpp:18-19` documents for that state and is real, oracle-driven behaviour, not a
// stand-in.
//
// **`path2_translate_with_callback` exercises the callback branch, fixed by adversarial
// review.** An earlier version of this file proved only the identity path, so the
// `// PROOF: 2` test passed unchanged against a port with no callback mechanism at all --
// `set_translate_callback`, the branch of `I18N.hpp:18-19` where `translate_fn != nullptr`, and
// the `const char*` overload had no oracle comparison. This function saves the current
// `translate_fn`, installs a fixed, deterministic callback (`path2_uppercase_callback` below),
// calls `Slic3r::I18N::translate` through it, and restores the saved value before returning --
// all inside this one call, on the calling thread, with no other code able to run between the
// save and the restore. That keeps the mutation of the process-global confined to this
// function's own duration, the same discipline `object_id.cpp` uses for `s_last_id`.
//
// Only `translate` (`I18N.hpp:18-19`) is exposed here, not `L`/`L_CONTEXT`/`_u8L`
// (`:31-33`): those three are private to `slic3r_i18n` on the Rust side (the idiom map's "a
// free function in an anonymous namespace" row, kept private the same way the vendored
// header's own anonymous namespace is), so an integration test under `tests/` -- which links
// against the crate's public API only -- cannot reach them to compare. `_u8L` forwards into
// `Slic3r::I18N::translate` verbatim, so `translate`'s own proof already exercises the code
// path those three use.
#include "i18_n.h"

#include "I18N.hpp"

#include <cctype>

namespace slic3r_ffi {
namespace proof_units {
namespace i18_n {

rust::String path2_translate(rust::Str s) {
    return rust::String(Slic3r::I18N::translate(std::string(s)));
}

namespace {

// A fixed, deterministic callback matching `Slic3r::I18N::translate_fn_type`
// (`I18N.hpp:15`, `std::string (*)(const char*)`): ASCII-only uppercasing, so the Rust-side
// proof test can reproduce the exact same transform with `str::to_ascii_uppercase` and compare
// outputs without needing a closure to cross the `cxx` bridge.
std::string path2_uppercase_callback(const char *s) {
    std::string result(s);
    for (char &c : result) {
        c = static_cast<char>(std::toupper(static_cast<unsigned char>(c)));
    }
    return result;
}

} // namespace

rust::String path2_translate_with_callback(rust::Str s) {
    Slic3r::I18N::translate_fn_type saved = Slic3r::I18N::translate_fn;
    Slic3r::I18N::set_translate_callback(&path2_uppercase_callback);
    std::string result = Slic3r::I18N::translate(std::string(s));
    Slic3r::I18N::set_translate_callback(saved);
    return rust::String(result);
}

} // namespace i18_n
} // namespace proof_units
} // namespace slic3r_ffi
