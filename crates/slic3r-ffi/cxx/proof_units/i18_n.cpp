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
// **Deliberately exercises only the default path (`translate_fn == nullptr`).**
// `Slic3r::I18N::translate_fn` is a process-global, externally linked `static` (`I18N.hpp:16`,
// defined at `utils.cpp:358` as `nullptr`), shared with every other translation unit the link
// extracts -- the same hazard `object_id.cpp`'s own comment documents at length for
// `ObjectBase::s_last_id`. This shim never calls `Slic3r::I18N::set_translate_callback`, so it
// never mutates that global; the default-`nullptr` identity path is exactly what `I18N.hpp:18-19`
// documents for that state and is real, oracle-driven behaviour, not a stand-in.
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

namespace slic3r_ffi {
namespace proof_units {
namespace i18_n {

rust::String path2_translate(rust::Str s) {
    return rust::String(Slic3r::I18N::translate(std::string(s)));
}

} // namespace i18_n
} // namespace proof_units
} // namespace slic3r_ffi
