//! Port of `GCode/PreciseSeamInternal.hpp`
//! (`vendor/OrcaSlicer/src/libslic3r/GCode/PreciseSeamInternal.hpp`), one of the 33 tasks cut
//! from plan step 20 (`intent/libslic3r-rust-port/tasks/84-foundation-precise-seam/task.md`):
//! the precise-seam binding intermediates -- a header-only unit (`template-heavy: yes`,
//! `port/manifest.tsv`) with no same-stem `.cpp`, 42 lines.
//!
//! **This module is deliberately incomplete, and stays `drafted`.** The header's third
//! include, `GCode/PreciseSeam.hpp`, is *not* a foundation row (`template-heavy: no`, its own
//! manifest row targeting `crates/slic3r-gcode/src/precise_seam.rs`, written by the Phase A/B
//! fan-out, not this task and not `83-foundation-support-params`). The header's three
//! functions -- `append_projected_fragment`, `bind_fragment`, `append_fragment` -- are
//! declarations only, with bodies in `GCode/PreciseSeam.cpp`, and `append_fragment` takes a
//! `const ExtractionContext &`, a type `GCode/PreciseSeam.hpp:90` declares: a structural
//! dependency on that not-yet-written sibling module, not a call inside a function body, so the
//! shared block's `// TODO(port):` escape does not reach it and there is no function body here
//! for a proof or a fuzz target to call. This module therefore carries the two record types in
//! full and a `// TODO(port):` for each declaration; it holds no `// PROOF:` marker, derives
//! `drafted` (its `mod` line reaches it, it compiles, it holds no live `todo` or
//! `unimplemented` macro, and the proof is the thing still missing), and adds no fuzz target --
//! `check_fuzz_targets` fires only at `ported`. See this task's pull request for what plan
//! step 22's cut is asked to pick up: `GCode/PreciseSeam.hpp`'s three signatures, the
//! `// PROOF: 1` marker, and the `slic3r-gcode_precise_seam_internal` fuzz target.

use slic3r_point::point::Point;

/// `struct ClippedEdgeInterval` (`:13-19`): one perimeter edge's clipped span, carrying the
/// edge's own identity (`edge`) forward to segment assembly -- "Binding intermediates retain
/// source edge identity until segment assembly" (`:12`). No in-class initializer on any field,
/// so this record has no faithful `Default`: a C++ `ClippedEdgeInterval{}` default-constructs
/// every field at *its own* type's default (an uninitialised `size_t`/`double` for the first
/// three, `Point`'s own default for the last two), which this module does not reproduce as a
/// Rust `Default` impl because the vendored struct never calls that constructor either.
#[derive(Debug, Clone, Copy)]
pub struct ClippedEdgeInterval {
    /// `size_t edge;` (`:14`).
    pub edge: usize,
    /// `double begin;` (`:15`).
    pub begin: f64,
    /// `double end;` (`:16`).
    pub end: f64,
    /// `Point first;` (`:17`).
    pub first: Point,
    /// `Point last;` (`:18`).
    pub last: Point,
}

/// `struct FragmentBindingFailure` (`:21-24`): the out-parameter the header's three functions
/// fill in on failure, carrying which pair failed and why.
#[derive(Debug, Clone, Copy)]
pub struct FragmentBindingFailure {
    /// `size_t pair_index = 0;` (`:22`).
    pub pair_index: usize,
    /// `const char *reason = "empty fragment";` (`:23`). `port/ownership.tsv`'s row for
    /// `GCode/PreciseSeamInternal.hpp:23` (`FragmentBindingFailure::reason`, class `borrowed`)
    /// gives this field's Rust type verbatim: `&'static str`. The field is reassigned, never
    /// freed or reallocated, by every in-scope call this header declares, matching a borrow of
    /// a string literal with `'static` lifetime -- the same literal the vendored default below
    /// uses.
    pub reason: &'static str,
}

impl Default for FragmentBindingFailure {
    /// `FragmentBindingFailure() = default`, with the in-class initializers `pair_index = 0`
    /// (`:22`) and `reason = "empty fragment"` (`:23`). `0` is `docs/PORTING.md`'s exempt
    /// structural literal, not a transcribed constant, so it carries no `// PORT SRC:`.
    fn default() -> Self {
        FragmentBindingFailure {
            pair_index: 0,
            reason: "empty fragment",
        }
    }
}

// TODO(port): GCode/PreciseSeam.hpp:90 -- `bool append_projected_fragment(const Polyline
// &fragment, const Polygon &perimeter, std::vector<ClippedEdgeInterval> &intervals,
// FragmentBindingFailure &failure)` (`GCode/PreciseSeamInternal.hpp:27-29`). Its body is in
// `GCode/PreciseSeam.cpp`, not this header, so there is nothing to port here; this signature is
// scope for the Phase A task that writes `crates/slic3r-gcode/src/precise_seam.rs`.
// PORT NOTE: `append_projected_fragment`'s body is in `GCode/PreciseSeam.cpp`, not this header.

// TODO(port): GCode/PreciseSeam.hpp:90 -- `bool bind_fragment(const Polyline &fragment, const
// Polygon &perimeter, std::vector<ClippedEdgeInterval> &intervals, FragmentBindingFailure
// &failure)` (`GCode/PreciseSeamInternal.hpp:33-34`). Its body is in `GCode/PreciseSeam.cpp`,
// not this header, so there is nothing to port here; this signature is scope for the Phase A
// task that writes `crates/slic3r-gcode/src/precise_seam.rs`.
// PORT NOTE: `bind_fragment`'s body is in `GCode/PreciseSeam.cpp`, not this header.

// TODO(port): GCode/PreciseSeam.hpp:90 -- `bool append_fragment(const Polyline &fragment, const
// Polygon &perimeter, std::vector<ClippedEdgeInterval> &intervals, const ExtractionContext
// &context, size_t fragment_index)` (`GCode/PreciseSeamInternal.hpp:38-41`). Its body is in
// `GCode/PreciseSeam.cpp`, not this header; its fourth parameter is `const ExtractionContext &`,
// a type `GCode/PreciseSeam.hpp:90` declares -- a structural dependency on the not-yet-written
// sibling module `crates/slic3r-gcode/src/precise_seam.rs`, which is why this declaration is not
// reachable from this header alone. This signature is scope for the Phase A task that writes
// that module.
// PORT NOTE: `append_fragment`'s body is in `GCode/PreciseSeam.cpp`, not this header, and its
// `ExtractionContext` parameter is declared in `GCode/PreciseSeam.hpp`, not here.

#[cfg(test)]
mod tests {
    use super::*;

    /// Not a `// PROOF:` test -- there is no callable function here to compare against the
    /// oracle, only the `FragmentBindingFailure() = default` in-class initializers
    /// (`GCode/PreciseSeamInternal.hpp:22-23`), checked directly against the header's own
    /// literals.
    #[test]
    fn fragment_binding_failure_default_matches_vendored_initializers() {
        let failure = FragmentBindingFailure::default();
        assert_eq!(failure.pair_index, 0);
        assert_eq!(failure.reason, "empty fragment");
    }
}

// PORT STATUS
// source: src/libslic3r/GCode/PreciseSeamInternal.hpp
// confidence: high
// tier: exact
// oracle: built
