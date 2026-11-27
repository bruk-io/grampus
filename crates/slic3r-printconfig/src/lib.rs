//! `slic3r-printconfig`, one of the ported-libslic3r crates `port/cut.tsv` cuts the vendored
//! tree into (plan step 20, task 34-foundation-spike). `mod` lines go here one per ported
//! module, in alphabetical order, as docs/WORKFLOW.md states.
//!
//! `mod print_config;` is landed: the module compiles and holds no live `todo!()` or
//! `unimplemented!()`, which is the whole of what docs/WORKFLOW.md's rule asks of a `mod`
//! line. It is **not** a completeness claim -- `print_config`'s own option table is partial,
//! and with no `// PROOF: 3` marker beside a test naming `print_config` the unit's two
//! manifest rows derive `drafted`, never `ported`. See that module's own header for what is
//! in and what is not.

pub mod print_config;
