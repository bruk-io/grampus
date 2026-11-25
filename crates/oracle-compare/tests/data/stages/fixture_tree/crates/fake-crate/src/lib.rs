// Fixture crate for `crates/oracle-compare/tests/stages.rs` (task `39-compare-stages`): exactly
// one module, reached by this `mod` line, so `row_is_ported` derives it `ported` -- the same
// shape `port/port-status`'s own `derive_row_status` scans for on the real tree. Deliberately
// does not declare `does_not_exist`, which the fixture manifest's `myNotPortedStage` row points
// at: that module genuinely has no file on disk, which is this fixture's "not ported" case.
pub mod ported_seam;
