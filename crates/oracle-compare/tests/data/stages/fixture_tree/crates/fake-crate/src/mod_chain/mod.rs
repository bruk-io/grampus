// Fixture: the directory half of a `mod.rs` chain. `leaf` resolves to `mod_chain/leaf.rs`
// (the other candidate `compute_mod_reachable_files` tries, `mod_chain/leaf/mod.rs`, does not
// exist in this fixture).
pub mod leaf;
