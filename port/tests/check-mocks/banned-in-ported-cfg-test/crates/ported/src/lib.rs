//! A crate holding a ported module (it carries a PORT STATUS trailer below), whose own
//! `#[cfg(test)]` module declares a banned identifier.

pub fn real_thing() -> i64 {
    1
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stub_helper() -> i64 {
        real_thing()
    }

    #[test]
    fn it_works() {
        assert_eq!(stub_helper(), 1);
    }
}

// PORT STATUS
// source: Fixture/Ported.cpp
// confidence: high
// tier: exact
// oracle: built
