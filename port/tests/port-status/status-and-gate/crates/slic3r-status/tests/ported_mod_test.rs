// Proof for crates/slic3r-status/src/ported_mod.rs (fixture for 22-port-status-gates).
//
// Updated for the differential-comparison rule (docs/PORTING.md, "Decisions recorded on
// 2026-11-25"): the `// PROOF:` marker counts only over a test that compares a value the oracle
// produced -- here a per-unit bridge the fixture imports through `slic3r-proof`'s re-export --
// against a value the module computed. This tree is never compiled, so the import and the call
// stand for the shape, exactly as the rest of this fixture's Rust does.

use slic3r_proof::proof_units::ported_mod::path3_compute;
use slic3r_status::ported_mod::compute;

// PROOF: 3
#[test]
fn ported_mod_matches_oracle() {
    let oracle = path3_compute();
    assert_eq!(compute(), oracle);
}
