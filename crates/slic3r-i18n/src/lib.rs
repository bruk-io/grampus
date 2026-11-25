//! `slic3r-i18n`, one of the ported-libslic3r crates `port/cut.tsv` cuts the vendored
//! tree into (plan step 20, task 34-foundation-spike). This skeleton carries no `mod`
//! line yet: Phase B (plan step 23) adds one per ported module, in alphabetical order,
//! as docs/WORKFLOW.md states.

mod i18_n;
pub use i18_n::{set_translate_callback, translate, TranslateFn};
