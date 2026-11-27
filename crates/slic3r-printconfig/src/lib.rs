//! `slic3r-printconfig`, one of the ported-libslic3r crates `port/cut.tsv` cuts the vendored
//! tree into (plan step 20, task 34-foundation-spike). `mod` lines go here one per ported
//! module, in alphabetical order, as docs/WORKFLOW.md states.
//!
//! `mod print_config;` is landed, the module holds no live `todo!()`/`unimplemented!()`, and
//! its `// PROOF: 3` marker is present over a differential comparison, so both of the unit's
//! manifest rows derive `ported`. That is `port/port-status`'s own rule satisfied and **not** a
//! claim that `PrintConfig.cpp`/`PrintConfig.hpp` are fully ported -- several pieces are still
//! absent, each with a `// TODO(port):` at its own site. See that module's own header for what
//! is in and what is not.

pub mod print_config;
