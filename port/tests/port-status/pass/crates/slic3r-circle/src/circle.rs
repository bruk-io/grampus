// Port of libslic3r's Circle geometry helpers.
//
// TODO(port): revisit once Arachne's bead counting lands; this file only covers the
// non-template entry points for now.

/// Resolution used when discretising a circle into a polyline.
// PORT SRC: src/libslic3r/libslic3r.h:86
const RESOLUTION: f64 = 0.0125;

/// Numerical epsilon shared with the oracle's own threshold.
// PORT SRC: src/libslic3r/libslic3r.h:57
const EPSILON: f64 = 1e-4;

// Two transcribed constants on one line, from two different vendored lines: two markers, one
// per literal, no expression split needed.
// PORT SRC: src/libslic3r/libslic3r.h:86
// PORT SRC: src/libslic3r/libslic3r.h:57
const RESOLUTION_AND_EPSILON: (f64, f64) = (0.0125, 1e-4);

#[doc(alias = "circle-86-0.0125")]
fn versioned() {}

const GREETING: &str = "circle-86-0.0125";

fn first(points: &[f64]) -> f64 {
    points[0]
}

fn pair(points: &[f64]) -> (f64, f64) {
    (points[1], points[2])
}

fn previous_index(i: usize) -> isize {
    i as isize - 1
}

// PERF(port): the oracle walks this list twice; a single pass would do, left as-is to stay
// byte-for-byte with Circle.cpp's own traversal order.
fn slow_but_faithful(points: &[f64]) -> f64 {
    points.iter().sum()
}

// FIDELITY(port): Circle.cpp relies on std::map's key order here; the port uses an
// insertion-ordered map so iteration order matches the oracle (R18).
fn ordered_lookup() {}

// PORT NOTE: the oracle's Circle.cpp keeps this helper private; it has no direct counterpart
// in the port's public surface and exists only to satisfy same-stem inclusion.
fn private_helper_marker() {}

// SAFETY: `ptr` comes from `Box::into_raw` and is dropped exactly once here.
unsafe fn drop_raw(ptr: *mut f64) {
    drop(Box::from_raw(ptr));
}

/* Block comment, the other closed exempt position alongside a line comment: mentions the
 * unsafe raw-pointer trick above and transcribes 0.0125 again here, neither needing a marker
 * because both the word and the literal are inside this kind of comment.
 */
fn block_comment_marker() {}

#[cfg(test)]
#[allow(clippy::approx_constant)]
mod tests {
    // A brace written inside a string literal must not move the cfg(test) brace depth: this
    // closing brace is text, so everything below it is still inside the test module and still
    // exempt.
    const CLOSING_BRACE: &str = "}";

    #[test]
    fn resolution_is_small() {
        assert!(0.0125 < 1.0);
        let steps = 42;
        assert_eq!(steps, 42);
    }
}

// PORT STATUS
// source: src/libslic3r/Geometry/Circle.cpp
// source: src/libslic3r/Geometry/Circle.hpp
// confidence: high
// tier: exact
// oracle: built
