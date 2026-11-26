// Fixture module for the `enclosing: <path>` proof-path form (docs/PORTING.md, "Decisions
// recorded on 2026-11-25"): reached by `lib.rs`'s `mod` line, holds no
// `todo!()`/`unimplemented!()`, and carries no `// PROOF:` marker anywhere -- a unit with
// nothing executable to prove of its own, whose row names the in-scope unit whose proof
// exercises it instead. `row_is_ported` must derive it from that named row, never from a marker
// here.

pub type Helper = i64;
