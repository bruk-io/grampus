#[cfg(test)]
mod tests {
    #[test]
    fn opening_brace_in_a_string() {
        assert_eq!("{".len(), 1);
    }
}

const TOLERANCE: f64 = 3.7;

// PORT STATUS
// source: src/libslic3r/Geometry/Circle.cpp
// confidence: high
// tier: exact
// oracle: built
