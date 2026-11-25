// Fixture crate for `crates/oracle-compare/tests/stages.rs` (task `39-compare-stages`): exactly
// one module, reached by this `mod` line, so `row_is_ported` derives it `ported` -- the same
// shape `port/port-status`'s own `derive_row_status` scans for on the real tree. Deliberately
// does not declare `does_not_exist`, which the fixture manifest's `myNotPortedStage` row points
// at: that module genuinely has no file on disk, which is this fixture's "not ported" case.
pub mod ported_seam;

// A directory `mod.rs` chain (`lib.rs` -> `mod_chain/mod.rs` -> `mod_chain/leaf.rs`), which no
// other fixture module here exercises: `port/port-status`'s own `compute_mod_reachable_files`
// resolves a `mod leaf;` line two different ways (`leaf.rs` or `leaf/mod.rs`), and
// `ported_seam.rs` alone never drives the directory-chain resolution path at all (`lib.rs`
// itself is the only `mod.rs`-named file `mod_reachable_files` sees otherwise).
pub mod mod_chain;

// Reached, holds no `todo!()`/`unimplemented!()` -- but its proof lives under this crate's own
// `tests/` directory (`tested_externally_proof.rs`) rather than in a `#[cfg(test)]` module
// inside this file, the other half of `proof_present`'s own two search locations that
// `ported_seam.rs` alone never exercises.
pub mod tested_externally;
