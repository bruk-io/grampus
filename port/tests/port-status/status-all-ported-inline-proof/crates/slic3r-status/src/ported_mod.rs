// Fixture (22-port-status-gates): identical in shape to status-all-ported's ported_mod.rs,
// except its proof lives in this file's own `#[cfg(test)]` module -- task.md's other
// sanctioned proof-presence form, "a `#[cfg(test)]` module in the module file" -- rather than
// under `crates/<crate>/tests/`. This is the fixture that proves the blocker a reviewer found
// is fixed: before the fix, `// PROOF:` written inside a `#[cfg(test)]` block tripped
// `check_marker_whitelist`'s unrecognized-marker finding, failing `--per-commit` on a module
// that conforms to this task's own proof-presence convention. A `mod` declaration reaches this
// file, it holds no `todo!()`/`unimplemented!()`, and `extract_cfg_test_text` finds the proof
// below -- expected to derive as "ported", and to pass --per-commit clean.

fn compute() -> i32 {
    2
}

#[cfg(test)]
mod tests {
    // PROOF: 3
    #[test]
    fn ported_mod_matches_oracle() {
        assert_eq!(1 + 1, 2);
    }
}

// PORT STATUS
// source: src/libslic3r/Geometry.hpp
// confidence: high
// tier: exact
// oracle: built
