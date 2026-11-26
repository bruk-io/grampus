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
//! function pointer would. **Fixed by adversarial review:** the vendored `translate_fn` is an
//! unsynchronised raw pointer, so `I18N.hpp:18`'s call to it is not reentrant-hostile -- a
//! callback that calls `translate` again just reads the same pointer again, and a throwing
//! callback (`cxx` has no C++ exception to propagate here, but the general case) leaves
//! `translate_fn` exactly as it was for the next call. An earlier version of [`translate`]
//! matched on `*TRANSLATE_FN.lock().unwrap()` directly, so the guard's temporary lived across
//! the call to `f(s)` and the lock was held while the callback ran: a callback that calls
//! [`translate`] again deadlocks (measured: hangs, killed at a 10s timeout), and a callback that
//! panics poisons the mutex permanently, so every later [`translate`] panics too. Neither is
//! reachable from the vendored semantics. [`translate`] now copies the `Option<TranslateFn>`
//! out of the guard (it is `Copy`) and drops the guard before calling it, and both functions
//! recover from a poisoned lock with [`std::sync::PoisonError::into_inner`] rather than
//! unwrapping it, so one panicking callback cannot wedge every later call the way a held lock
//! would.
//!
//! **Fixed by adversarial review: `set_translate_callback` takes `Option<TranslateFn>`, not
//! `TranslateFn`.** `I18N.hpp:15-17`'s `translate_fn_type` is a nullable function pointer, and
//! `set_translate_callback(nullptr)` is how the vendored code restores the identity state. A
//! non-nullable `TranslateFn` parameter could not express that reset; `None` now does.
//!
//! **Proof path 2, reached via `slic3r-proof`'s re-export of `slic3r-ffi`'s per-unit proof
//! bridges (`docs/PORTING.md`, "Decisions recorded on 2026-11-25", decisions 1 and 4).** Exactly
//! as `crates/slic3r-commondefs/src/common_defs.rs`'s doc comment states for `CommonDefs.hpp`:
//! `port/manifest.tsv` seeds this row's `proof path` as `2` from its `pipeline stage` cell
//! (`posSlice`) alone, and no field across the committed stage dump corpus carries a translated
//! string or any other I18N-derived value. Decision 4 names the per-unit proof bridges, not only
//! the dump corpus, as a "2" row's oracle entry point, so this unit's shim
//! (`crates/slic3r-ffi/cxx/proof_units/i18_n.{h,cpp}`, `crates/slic3r-ffi/src/proof_units/i18_n.rs`)
//! drives the vendored header's own inline code directly, both at its default
//! (`translate_fn == nullptr`) state and, since the same adversarial review, with a callback
//! installed and then restored -- see the shim's own comment for how it saves, sets, calls and
//! restores `translate_fn` inside its own translation unit.

use std::sync::Mutex;

/// `Slic3r::I18N::translate_fn_type` (`I18N.hpp:15`, `typedef std::string (*translate_fn_type)(const
/// char*)`), carried as a plain Rust function pointer rather than a `Box<dyn Fn>` -- the vendored
/// type is itself a bare function pointer, never a capturing closure.
pub type TranslateFn = fn(&str) -> String;

/// `Slic3r::I18N::translate_fn` (`I18N.hpp:16`, declared `extern`; defined at `utils.cpp:358` as
/// `nullptr`) -- `None` is that same default.
static TRANSLATE_FN: Mutex<Option<TranslateFn>> = Mutex::new(None);

/// `Slic3r::I18N::set_translate_callback` (`I18N.hpp:17`): `None` is the nullable vendored
/// parameter's `nullptr`, restoring the identity state `translate` falls back to.
pub fn set_translate_callback(f: Option<TranslateFn>) {
    let mut guard = TRANSLATE_FN
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    *guard = f;
}

/// `Slic3r::I18N::translate` (`I18N.hpp:18-19`, both overloads): the callback's result, or `s`
/// unchanged when none is set.
///
/// The `Option<TranslateFn>` is copied out of the lock and the guard is dropped before the
/// callback runs -- see this module's doc comment for why holding the lock across the call
/// would diverge from `I18N.hpp:18`'s unsynchronised raw-pointer read.
pub fn translate(s: &str) -> String {
    let callback = {
        let guard = TRANSLATE_FN
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        *guard
    };
    match callback {
        // PORT NOTE: I18N.hpp:18 hands the callback `s.c_str()`, a `const char*` that truncates
        // at the first embedded NUL byte; this port hands it the whole `&str`, which can contain
        // an interior NUL Rust strings do not treat specially. Divergent only on that callback
        // branch, and unreachable from any in-scope caller: the only vendored caller of
        // `set_translate_callback` is `src/slic3r/GUI/GUI_App.cpp:3715` (R10, the GUI module,
        // out of scope).
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

    /// Not a `// PROOF:` marker (see `crates/slic3r-i18n/tests/i18_n_proof_path_2.rs` for that) --
    /// a plain correctness check of `translate`'s identity fallback (`translate_fn == nullptr`,
    /// `I18N.hpp:18-19`'s own case), of `set_translate_callback(Some(..))` actually installing a
    /// callback `translate` then uses, of a callback that calls `translate` again not
    /// deadlocking now that the lock is not held across the call, and of
    /// `set_translate_callback(None)` restoring the identity state. One test, not several in
    /// this file: [`TRANSLATE_FN`] is one process-global `static`, exactly as the vendored
    /// `translate_fn` is, so a second `#[test]` touching it in this binary could race this one
    /// under `cargo test`'s default multi-threaded runner -- the same reason the
    /// oracle-comparing proof test lives in its own binary under `tests/` rather than beside
    /// this one.
    #[test]
    fn i18_n_translate_falls_back_to_identity_then_uses_a_set_callback() {
        assert_eq!(translate("hello"), "hello");
        assert_eq!(l("hello"), "hello");
        assert_eq!(l_context("hello", "menu"), "hello");
        assert_eq!(_u8l("hello"), "hello");

        fn shout(s: &str) -> String {
            s.to_uppercase()
        }
        set_translate_callback(Some(shout));
        assert_eq!(translate("hello"), "HELLO");
        assert_eq!(_u8l("hello"), "HELLO");

        // A callback that itself calls `translate` (and `set_translate_callback`) must not
        // deadlock: it would, if the outer `translate` still held the lock while this callback
        // ran (see this module's doc comment). Clearing the callback before the nested call
        // keeps the recursion to one level while still proving the lock is free during the
        // callback.
        fn reentrant(s: &str) -> String {
            set_translate_callback(None);
            let inner = translate(s);
            set_translate_callback(Some(reentrant));
            format!("[{inner}]")
        }
        set_translate_callback(Some(reentrant));
        assert_eq!(translate("hi"), "[hi]");

        set_translate_callback(None);
        assert_eq!(translate("hello"), "hello");
        assert_eq!(_u8l("hello"), "hello");
    }
}

// PORT STATUS
// source: src/libslic3r/I18N.hpp
// confidence: high
// tier: exact
// oracle: built
