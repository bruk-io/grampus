//! `grampus-cli`'s library surface (task `19-port-surface`, plan step 19, R10/R11): the
//! GUI-free five-call slicing sequence (`surface`) and the resolved-configuration reader
//! (`resolved_config`) that `src/main.rs` and `tests/surface.rs` use. This crate holds no
//! ported module — nothing under `crates/slic3r-*/` is referenced here, and no file in this
//! crate ends with the trailer `just port-status` looks for on a module it has ported.

pub mod resolved_config;
pub mod surface;
