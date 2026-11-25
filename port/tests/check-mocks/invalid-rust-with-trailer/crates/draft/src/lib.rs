//! A Phase A draft (plan step 14): not valid Rust, carries a PORT STATUS trailer, and reaches
//! this through no `mod` line anywhere (nothing `mod`s this file in). check-mocks must still
//! read it as plain lines and exit clean, since neither its own `#[cfg(test)]` module nor any
//! file under crates/*/tests/ in this fixture declares a banned identifier.

pub fn half_written( {
    let x = ;
    #[cfg(test)]
    mod tests {
        #[test]
        fn it_works() {
            assert_eq!(1 + 1, 2);
        }
    }
}

// PORT STATUS
// source: Fixture/Draft.cpp
// confidence: low
// tier: exact
// oracle: built
