//! `slic3r-geometry-dir`, one of the ported-libslic3r crates `port/cut.tsv` cuts the vendored
//! tree into (plan step 20, task 34-foundation-spike). This skeleton carries no `mod`
//! line yet: Phase B (plan step 23) adds one per ported module, in alphabetical order,
//! as docs/WORKFLOW.md states.

pub mod bicubic;
pub mod circle;
pub mod curves;
// Not yet reached by a `mod` line: `voronoi.rs` is an honest Phase A draft
// (task `76-foundation-voronoi-curves`), gated out of the build until
// `77-foundation-voronoi-offset` adds `voronoi_utils.rs` and this crate's `mod voronoi;` line
// together (see `voronoi.rs`'s own module doc).
