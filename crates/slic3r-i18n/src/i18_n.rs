//! Ports `vendor/OrcaSlicer/src/libslic3r/I18N.hpp` (crate-level level 0, hand-designed by plan
//! step 20's task `52-foundation-common-defs`, `port/foundation.tsv` row `foundation-01`): a
//! global, settable translation callback with an identity fallback, plus three anonymous-
//! namespace marker functions. No dependent among this task's own units and no
//! `port/ownership.tsv` row (no raw-pointer field at the pin).
//!
//! **Decision (task was silent): one Rust function instead of two C++ overloads.**
//! `I18N.hpp:18-19` overloads `translate` on `const std::string&` and `const char*`; Rust's
//! `&str` already covers both call shapes with no overload needed, so this module exposes one
//! [`translate`] taking `&str` -- a shape change forced by the language, not a behaviour change
//! (every call site produces the same string either way), so it is not the kind of "workaround"
//! `docs/WORKFLOW.md`'s "The rejection rule" forbids.
//!
//! **Decision (task was silent): the global callback is a `Mutex<Option<fn>>`, not
//! `static mut`.** `I18N.hpp:16-17`'s `translate_fn` is process-global mutable state, default
//! `nullptr` (`utils.cpp:358`). `Mutex::new` is `const fn`, so a plain `static` needs no lazy
//! initialisation and no `unsafe`, which a raw mutable static or an `AtomicPtr` cast back to a
//! function pointer would.
//!
//! **Proof path 2 could not be honoured as a real proof (a finding for this pull request, not a
//! tolerance or a stand-in invented on this branch).** Exactly `crates/slic3r-commondefs/src/common_defs.rs`'s
//! doc comment states for `CommonDefs.hpp`: `port/manifest.tsv` seeds this row's `proof path` as
//! `2` from its `pipeline stage` cell (`posSlice`) alone, and no field across the committed stage
//! dump corpus (`port/dumps/*/*.dump`) carries a translated string or any other I18N-derived
//! value -- the corpus's full field set is `fill_surfaces, fills, height, lslices, object_skirt,
//! perimeters, print_z, raw_slices, skirt, skirt_brim_groups, slice_z, slices, thin_fills,
//! wipe_tower_*`, checked the same way. This module therefore carries no `// PROOF:` marker;
//! `port/port-status` derives it as `drafted`, not `ported`.

use std::sync::Mutex;

/// `Slic3r::I18N::translate_fn_type` (`I18N.hpp:15`, `typedef std::string (*translate_fn_type)(const
/// char*)`), carried as a plain Rust function pointer rather than a `Box<dyn Fn>` -- the vendored
/// type is itself a bare function pointer, never a capturing closure.
pub type TranslateFn = fn(&str) -> String;

/// `Slic3r::I18N::translate_fn` (`I18N.hpp:16`, declared `extern`; defined at `utils.cpp:358` as
/// `nullptr`) -- `None` is that same default.
static TRANSLATE_FN: Mutex<Option<TranslateFn>> = Mutex::new(None);

/// `Slic3r::I18N::set_translate_callback` (`I18N.hpp:17`).
pub fn set_translate_callback(f: TranslateFn) {
    *TRANSLATE_FN.lock().unwrap() = Some(f);
}

/// `Slic3r::I18N::translate` (`I18N.hpp:18-19`, both overloads): the callback's result, or `s`
/// unchanged when none is set.
pub fn translate(s: &str) -> String {
    match *TRANSLATE_FN.lock().unwrap() {
        Some(f) => f(s),
        None => s.to_string(),
    }
}

// PORT NOTE: I18N.hpp:6-10 and :26-29 are C++ preprocessor guards (`#ifdef
// SLIC3R_CURRENTLY_COMPILING_GUI_MODULE` / `#error ...`) enforcing which of two build contexts
// (libslic3r itself vs. the `slic3r` GUI module, R10 out of scope) may define `L`/`L_CONTEXT`/
// `_u8L` below. The port is one crate with no such conditional-compilation axis, so the guard
// itself has nothing to translate; only the three functions it guards are ported.

/// `L` (`I18N.hpp:31`, anonymous namespace): an identity marker a translation-extraction tool
/// greps for, not a real transformation -- private per the idiom map's "a free function in an
/// anonymous namespace" row.
#[allow(dead_code)]
fn l(s: &str) -> &str {
    s
}

/// `L_CONTEXT` (`I18N.hpp:32`, anonymous namespace): same identity shape, with a disambiguating
/// `context` argument the C++ also ignores at runtime.
#[allow(dead_code)]
fn l_context<'a>(s: &'a str, _context: &str) -> &'a str {
    s
}

/// `_u8L` (`I18N.hpp:33`, anonymous namespace): `Slic3r::I18N::translate(s)`, verbatim.
#[allow(dead_code)]
fn _u8l(s: &str) -> String {
    translate(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Not a `// PROOF:` marker (see this module's doc comment: no dump field holds this
    /// unit's output, so proof path 2 cannot be honoured here) -- a plain correctness check of
    /// `translate`'s identity fallback (`translate_fn == nullptr`, `I18N.hpp:18-19`'s own case)
    /// and of `set_translate_callback` actually installing a callback `translate` then uses. One
    /// test, not two: [`TRANSLATE_FN`] is one process-global `static`, exactly as the vendored
    /// `translate_fn` is, so a second `#[test]` touching it could race this one under `cargo
    /// test`'s default multi-threaded runner -- the same reason this file never calls
    /// `set_translate_callback` outside this one function.
    #[test]
    fn i18_n_translate_falls_back_to_identity_then_uses_a_set_callback() {
        assert_eq!(translate("hello"), "hello");
        assert_eq!(l("hello"), "hello");
        assert_eq!(l_context("hello", "menu"), "hello");
        assert_eq!(_u8l("hello"), "hello");

        fn shout(s: &str) -> String {
            s.to_uppercase()
        }
        set_translate_callback(shout);
        assert_eq!(translate("hello"), "HELLO");
        assert_eq!(_u8l("hello"), "HELLO");
    }
}

// PORT STATUS
// source: src/libslic3r/I18N.hpp
// confidence: high
// tier: exact
// oracle: built
