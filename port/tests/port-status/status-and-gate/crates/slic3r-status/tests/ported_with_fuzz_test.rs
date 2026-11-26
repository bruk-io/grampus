// Proof for crates/slic3r-status/src/ported_with_fuzz.rs (fixture for 22-port-status-gates).
// See ported_mod_test.rs for why the comparison is differential.

use slic3r_proof::proof_units::ported_with_fuzz::path4_compute;
use slic3r_status::ported_with_fuzz::compute;

// PROOF: 4
#[test]
fn ported_with_fuzz_matches_oracle() {
    let oracle = path4_compute();
    assert_eq!(compute(), oracle);
}
