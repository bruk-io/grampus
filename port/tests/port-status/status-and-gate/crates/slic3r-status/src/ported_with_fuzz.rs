// Fixture (22-port-status-gates): a `mod` declaration reaches this file, it holds no
// todo!()/unimplemented!(), and its proof is present in
// crates/slic3r-status/tests/ported_with_fuzz_test.rs -- expected to derive as "ported". Its
// row's `fuzz target` is "required" and
// crates/slic3r-ffi/fuzz/fuzz_targets/slic3r-status_ported_with_fuzz.rs exists at the exact
// path the check requires, so this row must pass the required-fuzz-target check.

fn compute() -> i32 {
    2
}

// PORT STATUS
// source: src/libslic3r/Geometry.cpp
// confidence: high
// tier: exact
// oracle: built
