//! `slic3r-proof`, the shared proof harness every ported unit's test calls through (plan step
//! 18, task 31-proof-harness, R19). `tiers::compare_unit` is the one API every ported unit's
//! test uses; `dumps` (proof path 2, the committed stage dumps, task 32-proof-dumps-fuzz) reads
//! them into the same per-unit comparison. It is not a `port/cut.tsv` crate row -- no per-unit
//! completion trailer belongs in this crate and it gets no `port/manifest.tsv` row of its own.

pub mod dumps;
pub mod tiers;

pub use tiers::{compare_unit, compare_unit_at, Oracle, TierError};
