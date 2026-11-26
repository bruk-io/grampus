// The shape the rule exists to refuse: the committed stage dump is read, so the row's declared
// mechanism appears to run, but nothing the oracle emitted enters a comparison -- the module's
// output is checked against a string literal and against a table the test itself wrote. This is
// PR #54's `LifecycleEvents.hpp` proof, reduced to its shape.

use slic3r_proof::dumps;
use slic3r_status::ported_mod::render;

const EXPECTED: &str = "psWipeTower";

// PROOF: 3
#[test]
fn ported_mod_matches_oracle() {
    let dump = dumps::read_dump_default("20mm_cube-base-fff", "psWipeTower").unwrap();
    assert_eq!(dump.seam, "psWipeTower");
    assert_eq!(render(), EXPECTED);
}
