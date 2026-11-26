//! Proof path 2 (R19) for the `I18N.hpp` manifest row: `Slic3r::I18N::translate`
//! (`I18N.hpp:18-19`) at both its default (`translate_fn == nullptr`) state and with a callback
//! installed, called here through `crates/slic3r-ffi/cxx/proof_units/i18_n.cpp`'s shim, reached
//! through `slic3r-proof`'s re-export of the FFI crate's per-unit proof bridges -- the one
//! sanctioned route (`docs/PORTING.md`, "Decisions recorded on 2026-11-25").
//!
//! **Fixed by adversarial review: the callback branch is now proven, not just the identity
//! one.** An earlier version of this file proved only `translate_fn == nullptr`, where both
//! sides are the identity and a port with no callback mechanism at all would still pass. The
//! second half of the one test below installs a callback on the port side
//! (`slic3r_i18n::i18_n::set_translate_callback`) and compares against
//! `path2_translate_with_callback`, which does the oracle-side equivalent (save, set, call,
//! restore) confined to its own call, inside `cxx/proof_units/i18_n.cpp`.
//!
//! Only `translate` is compared against the oracle here, not `L`/`L_CONTEXT`/`_u8L`: those three
//! are private to `slic3r_i18n` (the idiom map's "a free function in an anonymous namespace" row,
//! kept private the same way the vendored header's own anonymous namespace is), and an
//! integration test under `tests/` links against the crate's public API only -- `_u8l` itself
//! calls `translate` verbatim (`I18N.hpp:33`), so `translate`'s own proof exercises the same
//! vendored code path those three forward into. `src/i18_n.rs`'s own `#[cfg(test)]` module still
//! plain-checks all four, including both branches this file also proves against the oracle.
//!
//! CONSTRAINT ON THIS FILE -- both branches are driven from **one** `#[test]` function, in
//! sequence, never two: the port's callback slot is one process-global `static`, exactly as the
//! vendored `translate_fn` is, so a second `#[test]` touching it in this binary could race this
//! one under `cargo test`'s default multi-threaded runner -- the same reason
//! `src/i18_n.rs`'s own `i18_n_translate_falls_back_to_identity_then_uses_a_set_callback` is also
//! one test rather than several, and the same reason this file lives in its own binary rather
//! than as a `#[cfg(test)]` module beside that one. Measured directly when this file carried
//! only the identity half: moved into the same binary as that other test first, this test failed
//! (`left: "hello", right: "HELLO"`) whenever the two ran in the same process, because `cargo
//! test`'s default parallel runner does not order them -- which is why they remain two separate
//! binaries.

use slic3r_i18n::i18_n::{set_translate_callback, translate};
use slic3r_proof::proof_units::i18_n::{path2_translate, path2_translate_with_callback};

/// `path2_translate_with_callback`'s fixed oracle-side transform
/// (`crates/slic3r-ffi/cxx/proof_units/i18_n.cpp`'s `path2_uppercase_callback`): ASCII-only
/// uppercasing, reproduced here with `str::to_ascii_uppercase` rather than `str::to_uppercase` so
/// both sides apply the exact same byte-for-byte transform (Unicode case folding can differ from
/// C's `toupper`, which is why every input below stays ASCII).
fn uppercase_callback(s: &str) -> String {
    s.to_ascii_uppercase()
}

/// `Slic3r::I18N::translate` against this port's own `translate`, at both states `I18N.hpp:18-19`
/// can be in. Each side is driven independently -- the oracle through the per-unit proof bridge,
/// the port through its own functions.
// PROOF: 2
#[test]
fn i18_n_translate_matches_the_oracle_at_the_default_state() {
    for input in ["hello", "", "héllo wörld", "blend_color_multi"] {
        let oracle_translate = path2_translate(input);
        let ported_translate = translate(input);
        assert_eq!(oracle_translate, ported_translate);
    }

    // The callback branch (`translate_fn != nullptr`), confined to ASCII inputs since the fixed
    // oracle-side callback is C's `toupper`, not Unicode case folding (see `uppercase_callback`'s
    // own doc comment).
    set_translate_callback(Some(uppercase_callback));
    for input in ["hello", "", "mixed CASE 123", "blend_color_multi"] {
        let oracle_with_callback = path2_translate_with_callback(input);
        let ported_with_callback = translate(input);
        assert_eq!(oracle_with_callback, ported_with_callback);
    }
    set_translate_callback(None);

    // Restoring `None` must bring back the identity fallback this test's first half measured.
    assert_eq!(translate("hello"), "hello");
}
