// Fixture crate for 22-port-status-gates' status/gate self-test. Not real crate metadata --
// Cargo.toml is deliberately omitted, since mod-reachability only ever reads lib.rs/mod.rs
// text, never cargo metadata.
//
// drafted_mod, not_started_mod and outscope_mod are deliberately NOT named here: each must
// stay unreached for its own row's expected status.

mod mod_reached_todo_mod;
mod no_proof_mod;
mod ported_mod;
mod ported_with_fuzz;
