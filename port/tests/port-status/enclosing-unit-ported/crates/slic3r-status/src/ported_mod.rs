// Fixture (22-port-status-gates): a `mod` declaration reaches this file, it holds no
// todo!()/unimplemented!(), and its proof is present in
// crates/slic3r-status/tests/ported_mod_test.rs -- expected to derive as "ported". Its row's
// `fuzz target` is "required" and
// crates/slic3r-ffi/fuzz/fuzz_targets/slic3r-status_ported_mod.rs is deliberately absent, so
// the required-fuzz-target check must fail this row.

fn compute() -> i32 {
    2
}

// PORT STATUS
// source: src/libslic3r/Geometry.hpp
// confidence: high
// tier: exact
// oracle: built
