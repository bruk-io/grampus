// The shape the rule must accept, and refused before `let_binding_pattern` cut a `let`'s type
// annotation off its pattern: the oracle's own value is compared against the module's own, but
// both bindings are annotated, so `Mask` read as a name the oracle had bound. The taint then
// reached `ported` through `Mask::ALL`, both arguments of the comparison read as oracle values,
// and the row derived "drafted". This is PR #56's `crates/slic3r-filamentmixermodel` proof,
// reduced to its shape.

use slic3r_proof::proof_units;
use slic3r_status::ported_mod::Mask;

// PROOF: 1
#[test]
fn ported_mod_matches_oracle() {
    let oracle: Mask = proof_units::ported_mod::path1_combined_mask();
    let ported: Mask = Mask::ALL;
    assert_eq!(oracle, ported);
}
