// Fixture crate for 22-port-status-gates' self-test: every in-scope row is "ported", and its
// proof lives in its own module's `#[cfg(test)]` block rather than under
// crates/<crate>/tests/ -- task.md's other sanctioned proof-presence form.

mod ported_mod;
