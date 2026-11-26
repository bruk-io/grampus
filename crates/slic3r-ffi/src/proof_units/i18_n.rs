//! Proof path 2 (R19) for the `I18N.hpp` unit (task 52-foundation-common-defs,
//! `port/foundation.tsv` row `foundation-01`).
//!
//! `path2_translate` drives the vendored header's own inline code at its default
//! (`translate_fn == nullptr`) state; `path2_translate_with_callback` drives the same code with
//! a callback installed, fixed by adversarial review -- see `cxx/proof_units/i18_n.cpp` for how
//! it saves, sets, calls and restores `translate_fn` inside its own call, and for why only
//! `translate` (not `L`/`L_CONTEXT`/`_u8L`) is exposed here.

#[cxx::bridge(namespace = "slic3r_ffi::proof_units::i18_n")]
mod ffi {
    // SAFETY: this is cxx's own unsafe-extern declaration block, not a hand-written unsafe
    // expression (see src/clipper2.rs's identical note) -- it carries no body, and the only
    // unsafety is in the bindings `cxx` generates from it.
    unsafe extern "C++" {
        include!("cxx/proof_units/i18_n.h");

        /// `Slic3r::I18N::translate` (`I18N.hpp:18-19`) at its default `translate_fn == nullptr`
        /// state: returns `s` unchanged.
        fn path2_translate(s: &str) -> String;

        /// `Slic3r::I18N::translate` (`I18N.hpp:18-19`) with a callback installed: saves the
        /// current `translate_fn`, installs a fixed ASCII-uppercasing callback, calls
        /// `translate`, restores the saved value and returns the result.
        fn path2_translate_with_callback(s: &str) -> String;
    }
}

pub use ffi::{path2_translate, path2_translate_with_callback};
