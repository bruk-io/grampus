// Fixture for port/audit-order --self-test: a marker outside the crate's src/ tree, at
// <crate>/tests/, which the narrower *-src-only glob never reaches. One row expected.

// FIDELITY(port): fixture marker outside src/, for the audit-order self-test.
fn ordered_lookup_in_tests() {}
