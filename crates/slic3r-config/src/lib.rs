//! `slic3r-config`, one of the ported-libslic3r crates `port/cut.tsv` cuts the vendored
//! tree into (plan step 20, task 34-foundation-spike). `config` is this crate's one
//! hand-designed foundation module (task `59-foundation-config`); any further module Phase B
//! (plan step 23) ports lands its own `mod` line at its alphabetical position, per
//! docs/WORKFLOW.md's "Conflict discipline".

pub mod config;
