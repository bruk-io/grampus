// Fixture module reached by `lib.rs`, holds no `todo!()`/`unimplemented!()`, and carries no
// `#[cfg(test)]` module of its own: its proof lives under this crate's `tests/` directory
// (`tests/tested_externally_proof.rs`) instead -- `proof_present`'s other search location,
// which no other fixture module here exercises (`ported_seam.rs` and `mod_chain/leaf.rs` both
// carry their proof inline).

pub fn value() -> i64 {
    13
}
