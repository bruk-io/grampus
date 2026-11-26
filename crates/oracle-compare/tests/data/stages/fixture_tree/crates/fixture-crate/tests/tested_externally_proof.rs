// Fixture proof under `crates/<crate>/tests/` (task `39-compare-stages`'s `row_is_ported`,
// mirroring `port/port-status`'s `proof_present`): a `#[test]` function whose name contains
// `tested_externally` (the module's own file stem), with a `// PROOF: proof/tested_externally`
// line immediately above it, and -- since docs/PORTING.md's "Decisions recorded on 2026-11-25"
// -- a body comparing a value the oracle produced against a value the module computed.

use fixture_crate::tested_externally::value;
use slic3r_proof::proof_units::tested_externally::path1_value;

// PROOF: proof/tested_externally
#[test]
fn test_tested_externally() {
    let oracle = path1_value();
    assert_eq!(value(), oracle);
}
