// Fixture module reached only through a directory `mod.rs` chain (`lib.rs` -> `mod_chain/mod.rs`
// -> here), holds no `todo!()`/`unimplemented!()`, and carries its own inline proof.

pub fn stub() {}

#[cfg(test)]
mod tests {
    // PROOF: proof/mod_chain_leaf
    #[test]
    fn test_mod_chain_leaf() {}
}
