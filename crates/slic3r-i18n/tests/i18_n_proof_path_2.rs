//! Proof path 2 (R19) for the `I18N.hpp` manifest row: `Slic3r::I18N::translate`
//! (`I18N.hpp:18-19`) at its default (`translate_fn == nullptr`) state, called here through
//! `crates/slic3r-ffi/cxx/proof_units/i18_n.cpp`'s shim, reached through `slic3r-proof`'s
//! re-export of the FFI crate's per-unit proof bridges -- the one sanctioned route
//! (`docs/PORTING.md`, "Decisions recorded on 2026-11-25").
//!
//! Only `translate` is compared against the oracle here, not `L`/`L_CONTEXT`/`_u8L`: those three
//! are private to `slic3r_i18n` (the idiom map's "a free function in an anonymous namespace" row,
//! kept private the same way the vendored header's own anonymous namespace is), and an
//! integration test under `tests/` links against the crate's public API only -- `_u8l` itself
//! calls `translate` verbatim (`I18N.hpp:33`), so `translate`'s own default-state proof exercises
//! the same vendored code path those three forward into. `src/i18_n.rs`'s own `#[cfg(test)]`
//! module still plain-checks all four, including the identity fallback this file also proves
//! against the oracle.
//!
//! CONSTRAINT ON THIS FILE -- it never calls `slic3r_i18n::set_translate_callback`, deliberately,
//! and lives in its own binary (`crates/<crate>/tests/`, not a `#[cfg(test)]` module beside
//! `slic3r_i18n::tests::i18_n_translate_falls_back_to_identity_then_uses_a_set_callback`) for the
//! same reason that test's own doc comment states: both touch `translate`'s one-and-only
//! identity/callback fork on their own side, and the Rust-side callback that test installs is a
//! process-global `static`. Measured directly: moved into the same binary first, this test failed
//! (`left: "hello", right: "HELLO"`) whenever the two ran in the same process, because `cargo
//! test`'s default parallel runner does not order them. The oracle side (`Slic3r::I18N`'s own
//! `translate_fn`) is left untouched throughout -- this file calls no `set_translate_callback` on
//! either side, so the default-identity state it measures cannot be perturbed by anything else in
//! this binary.

use slic3r_i18n::translate;
use slic3r_proof::proof_units::i18_n::path2_translate;

/// `Slic3r::I18N::translate` at its default state, against this port's own `translate`. Each
/// side is driven independently -- the oracle through the per-unit proof bridge, the port
/// through its own function -- over a spread of inputs including the empty string and
/// non-ASCII text.
// PROOF: 2
#[test]
fn i18_n_translate_matches_the_oracle_at_the_default_state() {
    for input in ["hello", "", "héllo wörld", "blend_color_multi"] {
        let oracle_translate = path2_translate(input);
        let ported_translate = translate(input);
        assert_eq!(oracle_translate, ported_translate);
    }
}
