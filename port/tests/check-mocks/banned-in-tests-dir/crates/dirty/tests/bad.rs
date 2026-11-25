//! A test file under crates/*/tests/ that declares a banned identifier (a word check-mocks
//! must catch regardless of which crate this is, since there is no allowlist).

fn make_mock_helper() {}

#[test]
fn uses_it() {
    make_mock_helper();
}
