// Fixture module reached only through a directory `mod.rs` chain (`lib.rs` -> `mod_chain/mod.rs`
// -> here), holds no `todo!()`/`unimplemented!()`, and carries its own inline proof.

pub fn value() -> i64 {
    11
}

#[cfg(test)]
mod tests {
    use super::value;
    use slic3r_proof::proof_units::mod_chain_leaf::path1_value;

    // PROOF: proof/mod_chain_leaf
    #[test]
    fn test_mod_chain_leaf() {
        let oracle = path1_value();
        assert_eq!(value(), oracle);
    }
}
